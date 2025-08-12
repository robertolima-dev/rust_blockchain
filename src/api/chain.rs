use actix_web::{HttpResponse, Responder, get, post, web};
use log::{debug, info, warn};
use std::collections::HashSet;

use super::models::{
    AppState, ChainResponse, DifficultyResponse, MineRequest, MineResponse, SetDifficultyRequest,
    ValidateResponse,
};
use crate::blockchain::{BASE_REWARD, Blockchain, DEFAULT_DIFFICULTY};
use crate::transaction::{OutPoint, Transaction, TxInput, TxOutput, UtxoSet};

/// Get the full blockchain.
#[get("/chain/")]
pub async fn get_chain(state: web::Data<AppState>) -> impl Responder {
    let bc = state.blockchain.lock().expect("mutex poisoned");
    let resp = ChainResponse {
        length: bc.len(),
        difficulty: bc.difficulty(),
        chain: &bc.chain,
    };
    HttpResponse::Ok().json(resp)
}

/// Validate the whole chain.
#[get("/validate/")]
pub async fn validate_chain(state: web::Data<AppState>) -> impl Responder {
    let bc = state.blockchain.lock().expect("mutex poisoned");
    let resp = ValidateResponse {
        valid: bc.is_valid_chain(),
        length: bc.len(),
        difficulty: bc.difficulty(),
    };
    HttpResponse::Ok().json(resp)
}

/// Mine a new block from the current mempool:
/// - Select valid txs against current UTXO (prevent double spends inside block)
/// - Create coinbase to `miner_address` with BASE_REWARD + total fees
/// - Mine PoW
/// - Apply block to UTXO (spend inputs, add outputs)
/// - Remove included txs from mempool
#[post("/mine/")]
pub async fn mine_block(state: web::Data<AppState>, req: web::Json<MineRequest>) -> impl Responder {
    let miner_address = req.miner_address.trim().to_string();
    if miner_address.is_empty() {
        return HttpResponse::BadRequest().body("miner_address required");
    }

    // Snapshot mempool (clone) to decide what to include
    let mempool_snapshot = {
        let mempool = state.mempool.lock().expect("mutex poisoned");
        mempool.clone()
    };

    // Lock UTXO to select txs + compute fees; release before PoW
    let (mut selected, total_fees_u128) = {
        let utxo = state.utxo_set.lock().expect("mutex poisoned");
        let (txs, fees) = select_transactions(&mempool_snapshot, &utxo);
        debug!(
            "MINER - selected {} txs from mempool (fees={} sat)",
            txs.len(),
            fees
        );
        (txs, fees)
    };

    // Build coinbase (first tx)
    let total_fees_u64 = (total_fees_u128 as u128).min(u128::from(u64::MAX - BASE_REWARD)) as u64;
    let coinbase_amount = BASE_REWARD + total_fees_u64;
    let coinbase = Transaction::new(
        vec![], // no inputs
        vec![TxOutput {
            address: miner_address.clone(),
            amount: coinbase_amount,
        }],
    );

    // Prepend coinbase to block transactions
    let mut txs_for_block = Vec::with_capacity(1 + selected.len());
    txs_for_block.push(coinbase.clone());
    txs_for_block.append(&mut selected);

    // Mine PoW
    let mined_block_hash;
    let mined_block_index;
    let mined_block_nonce;
    {
        let mut bc = state.blockchain.lock().expect("mutex poisoned");
        let b = bc.mine_block(txs_for_block);
        mined_block_hash = b.hash.clone();
        mined_block_index = b.index;
        mined_block_nonce = b.nonce;
    } // release blockchain lock before heavy apply

    // Apply block effects to UTXO and clean mempool
    {
        // Reconstruct the transactions we just mined to apply:
        // We can fetch last block from chain (safe in single-proc).
        let bc = state.blockchain.lock().expect("mutex poisoned");
        let last_block = bc.last_block();
        let included_txids: HashSet<String> = last_block
            .transactions
            .iter()
            .skip(1)
            .map(|t| t.txid.clone())
            .collect();
        let coinbase_tx = &last_block.transactions[0];

        // Apply to UTXO
        {
            let mut utxo = state.utxo_set.lock().expect("mutex poisoned");

            // Spend inputs of normal txs
            for tx in last_block.transactions.iter().skip(1) {
                for input in &tx.inputs {
                    utxo.spend(&input.outpoint);
                }
            }

            // Add outputs of normal txs
            for tx in last_block.transactions.iter().skip(1) {
                utxo.add_tx_outputs(tx);
            }

            // Add coinbase output(s)
            utxo.add_tx_outputs(coinbase_tx);
            debug!(
                "UTXO applied: +coinbase {}, txs_included={}, utxo_size={}",
                coinbase_tx.txid,
                included_txids.len(),
                utxo.len()
            );
        }

        // Remove included txs from mempool
        {
            let mut mempool = state.mempool.lock().expect("mutex poisoned");
            let before = mempool.len();
            mempool.retain(|t| !included_txids.contains(&t.txid));
            let after = mempool.len();
            debug!(
                "Mempool cleaned: {} -> {} (removed {})",
                before,
                after,
                before.saturating_sub(after)
            );
        }
    }

    let resp = MineResponse {
        mined_index: mined_block_index,
        hash: mined_block_hash,
        nonce: mined_block_nonce,
        difficulty: {
            let bc = state.blockchain.lock().expect("mutex poisoned");
            bc.difficulty()
        },
    };
    info!(
        "MINER - sealed block #{} (hash={}, nonce={})",
        resp.mined_index, resp.hash, resp.nonce
    );
    HttpResponse::Ok().json(resp)
}

/// Get current PoW difficulty.
#[get("/difficulty/")]
pub async fn get_difficulty(state: web::Data<AppState>) -> impl Responder {
    let bc = state.blockchain.lock().expect("mutex poisoned");
    HttpResponse::Ok().json(DifficultyResponse {
        difficulty: bc.difficulty(),
    })
}

/// Update PoW difficulty (affects future blocks only).
#[post("/difficulty/")]
pub async fn set_difficulty(
    state: web::Data<AppState>,
    body: web::Json<SetDifficultyRequest>,
) -> impl Responder {
    if body.difficulty > 6 {
        return HttpResponse::BadRequest().body("difficulty too high for dev mode (max 6)");
    }
    let mut bc = state.blockchain.lock().expect("mutex poisoned");
    bc.set_difficulty(body.difficulty);
    HttpResponse::Ok().json(DifficultyResponse {
        difficulty: bc.difficulty(),
    })
}

/* -------------------- Helpers -------------------- */

/// Select transactions from mempool that are valid against current UTXO.
/// Prevents double-spends inside the same block. Returns (selected_txs, total_fees).
fn select_transactions(mempool: &[Transaction], utxo: &UtxoSet) -> (Vec<Transaction>, u128) {
    let mut selected = Vec::new();
    let mut consumed = HashSet::<(String, u32)>::new(); // outpoints used in this block
    let mut total_fees: u128 = 0;

    'outer: for tx in mempool {
        if tx.inputs.is_empty() {
            // We don't accept coinbase-like from mempool
            continue;
        }

        // Ensure inputs exist and are not already consumed in this block
        let mut input_sum: u128 = 0;
        for input in &tx.inputs {
            let op = &input.outpoint;
            let key = (op.txid.clone(), op.vout);
            if consumed.contains(&key) {
                // would double-spend inside this block
                continue 'outer;
            }
            match utxo.get(op) {
                Some(out) => {
                    input_sum += out.amount as u128;
                }
                None => {
                    // missing UTXO -> skip this tx
                    continue 'outer;
                }
            }
        }

        let output_sum = tx.total_output_amount();
        if input_sum < output_sum {
            // invalid economics -> skip
            continue;
        }

        // Accept this tx: mark inputs consumed and add fee
        for input in &tx.inputs {
            let op = &input.outpoint;
            consumed.insert((op.txid.clone(), op.vout));
        }
        total_fees += input_sum - output_sum;
        selected.push(tx.clone());
    }

    (selected, total_fees)
}
