use actix_web::{HttpResponse, Responder, post, web};
use log::{debug, info, warn};
use uuid::Uuid;

use super::models::{
    AppState, MiningTemplate, SubmitRequest, SubmitResponse, TemplateRequest, TemplateResponse,
};
use crate::blockchain::{BASE_REWARD, Block, MAX_BLOCK_BYTES, MAX_TXS_PER_BLOCK};
use crate::transaction::{Transaction, TxOutput, UtxoSet};

/// Seleciona transações (mesma lógica greedy por fee-rate do chain.rs).
fn select_transactions(mempool: &[Transaction], utxo: &UtxoSet) -> (Vec<Transaction>, u128) {
    #[derive(Clone)]
    struct Cand {
        idx: usize,
        fee: u128,
        size: usize,
        fee_rate: f64,
    }
    let mut cands: Vec<Cand> = Vec::new();

    for (idx, tx) in mempool.iter().enumerate() {
        if tx.inputs.is_empty() {
            continue;
        }
        let mut input_sum: u128 = 0;
        let mut ok = true;
        for input in &tx.inputs {
            match utxo.get(&input.outpoint) {
                Some(prev) => input_sum += prev.amount as u128,
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if !ok {
            continue;
        }
        let output_sum = tx.total_output_amount();
        if input_sum < output_sum {
            continue;
        }
        let fee = input_sum - output_sum;
        let size = tx.vsize_bytes();
        let fee_rate = if size > 0 {
            fee as f64 / size as f64
        } else {
            0.0
        };
        cands.push(Cand {
            idx,
            fee,
            size,
            fee_rate,
        });
    }

    cands.sort_by(|a, b| {
        b.fee_rate
            .partial_cmp(&a.fee_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.fee.cmp(&a.fee))
            .then_with(|| mempool[a.idx].txid.cmp(&mempool[b.idx].txid))
    });

    let mut total_fees: u128 = 0;
    let mut total_bytes: usize = 0;
    let mut picked: Vec<Transaction> = Vec::new();
    let mut consumed = std::collections::HashSet::<(String, u32)>::new();

    for c in cands {
        if picked.len() >= MAX_TXS_PER_BLOCK {
            break;
        }
        if total_bytes + c.size > MAX_BLOCK_BYTES {
            continue;
        }
        let tx = &mempool[c.idx];

        let mut ok = true;
        for input in &tx.inputs {
            let key = (input.outpoint.txid.clone(), input.outpoint.vout);
            if consumed.contains(&key) {
                ok = false;
                break;
            }
        }
        if !ok {
            continue;
        }

        for input in &tx.inputs {
            consumed.insert((input.outpoint.txid.clone(), input.outpoint.vout));
        }
        total_fees += c.fee;
        total_bytes += c.size;
        picked.push(tx.clone());
    }

    (picked, total_fees)
}

/// Produz um template fixando timestamp e a lista de txs (coinbase primeiro).
#[post("/mining/template/")]
pub async fn get_template(
    state: web::Data<AppState>,
    req: web::Json<TemplateRequest>,
) -> impl Responder {
    let miner_addr = req.miner_address.trim();
    if miner_addr.is_empty() {
        return HttpResponse::BadRequest().body("miner_address required");
    }

    // snapshot da head/difficulty
    let (index, previous_hash, difficulty) = {
        let bc = state.blockchain.lock().expect("mutex");
        (
            bc.len() as u64,
            bc.last_block().hash.clone(),
            bc.difficulty(),
        )
    };

    // snapshot mempool + utxo para seleção e cálculo de fees
    let mempool_snapshot = {
        let mem = state.mempool.lock().expect("mutex");
        mem.clone()
    };
    let (mut selected, total_fees) = {
        let utxo = state.utxo_set.lock().expect("mutex");
        select_transactions(&mempool_snapshot, &utxo)
    };

    // coinbase
    let total_fees_u64 = (total_fees as u128).min(u128::from(u64::MAX - BASE_REWARD)) as u64;
    let coinbase_amount = BASE_REWARD + total_fees_u64;
    let coinbase = Transaction::new(
        vec![],
        vec![TxOutput {
            address: miner_addr.to_string(),
            amount: coinbase_amount,
        }],
    );

    // txs do bloco = coinbase + selecionadas
    let mut txs = Vec::with_capacity(1 + selected.len());
    txs.push(coinbase);
    txs.append(&mut selected);

    // fixar timestamp para o template
    let timestamp = chrono::Utc::now().timestamp();

    // armazenar template
    let template_id = Uuid::new_v4().to_string();
    {
        let mut map = state.mining_templates.lock().expect("mutex");
        map.insert(
            template_id.clone(),
            MiningTemplate {
                template_id: template_id.clone(),
                index,
                previous_hash: previous_hash.clone(),
                timestamp,
                difficulty,
                miner_address: miner_addr.to_string(),
                transactions: txs.clone(),
            },
        );
    }

    debug!(
        "TEMPLATE id={} height={} txs={} diff={}",
        &template_id,
        index,
        txs.len(),
        difficulty
    );

    HttpResponse::Ok().json(TemplateResponse {
        template_id,
        index,
        previous_hash,
        timestamp,
        difficulty,
        transactions: txs,
    })
}

/// Submete uma solução de PoW (nonce/hash) para um template.
/// Revalida head/diff e aplica bloco no UTXO/mempool se aceitar.
#[post("/mining/submit/")]
pub async fn submit_solution(
    state: web::Data<AppState>,
    req: web::Json<SubmitRequest>,
) -> impl Responder {
    // pega e remove o template (consumo único)
    let template = {
        let mut map = state.mining_templates.lock().expect("mutex");
        match map.remove(&req.template_id) {
            Some(t) => t,
            None => {
                return HttpResponse::BadRequest().json(SubmitResponse {
                    accepted: false,
                    mined_index: None,
                    hash: None,
                    difficulty: None,
                });
            }
        }
    };

    // checa head atual
    {
        let bc = state.blockchain.lock().expect("mutex");
        if bc.last_block().hash != template.previous_hash {
            warn!("stale template {}: head moved", template.template_id);
            return HttpResponse::BadRequest().json(SubmitResponse {
                accepted: false,
                mined_index: None,
                hash: None,
                difficulty: None,
            });
        }
    }

    // reconstrói o bloco com o mesmo timestamp/txs e aplica nonce
    let mut block = Block::new_with_timestamp(
        template.index,
        template.previous_hash.clone(),
        template.transactions.clone(),
        template.timestamp,
    );
    block.nonce = req.nonce;
    block.hash = block.compute_hash();

    // valida hash informado
    if block.hash != req.hash {
        return HttpResponse::BadRequest().body("hash mismatch");
    }

    // confere PoW via chain (dif atual)
    {
        let mut bc = state.blockchain.lock().expect("mutex");
        if !block.is_valid(bc.difficulty()) {
            return HttpResponse::BadRequest().body("hash does not meet difficulty");
        }
        // append premined
        if let Err(e) = bc.append_premined_block(block.clone()) {
            return HttpResponse::BadRequest().body(e);
        }
    }

    // aplicar efeitos: gastar inputs, adicionar outputs, limpar mempool das txs incluídas
    {
        let included_txids: std::collections::HashSet<String> = template
            .transactions
            .iter()
            .skip(1)
            .map(|t| t.txid.clone())
            .collect();
        let coinbase_tx = &template.transactions[0];

        {
            let mut utxo = state.utxo_set.lock().expect("mutex");
            for tx in template.transactions.iter().skip(1) {
                for input in &tx.inputs {
                    utxo.spend(&input.outpoint);
                }
            }
            for tx in template.transactions.iter().skip(1) {
                utxo.add_tx_outputs(tx);
            }
            utxo.add_tx_outputs(coinbase_tx);
            debug!(
                "Applied premined block to UTXO ({} txs + coinbase)",
                included_txids.len()
            );
        }
        {
            let mut mem = state.mempool.lock().expect("mutex");
            mem.retain(|t| !included_txids.contains(&t.txid));
        }
    }

    // info final
    let (height, diff) = {
        let bc = state.blockchain.lock().expect("mutex");
        (bc.len(), bc.difficulty())
    };

    info!(
        "ACCEPTED template {} -> block#{} hash={} diff={}",
        template.template_id,
        height - 1,
        req.hash,
        diff
    );

    HttpResponse::Ok().json(SubmitResponse {
        accepted: true,
        mined_index: Some(height as u64 - 1),
        hash: Some(req.hash.clone()),
        difficulty: Some(diff),
    })
}
