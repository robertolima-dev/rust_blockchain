use crate::wallet::{pubkey_to_address_hex, verify_signature_hex};
use actix_web::{HttpResponse, Responder, get, post, web};
use log::{debug, info, warn};
use std::collections::HashSet;
use std::time::Instant;

use super::models::{
    AppState, FaucetRequest, FaucetResponse, MempoolResponse, NewTxRequest, NewTxResponse,
};
use crate::transaction::{OutPoint, Transaction, TxInput, TxOutput, UtxoSet};

/// DEV Faucet: create spendable UTXOs directly in the UTXO set.
/// This avoids hidden seeds and makes testing straightforward.
#[post("/faucet/")]
pub async fn post_faucet(
    state: web::Data<AppState>,
    body: web::Json<FaucetRequest>,
) -> impl Responder {
    if body.amount == 0 {
        return HttpResponse::BadRequest().body("amount must be > 0");
    }

    // Create a fake coinbase tx with 1 output (address/amount).
    // We don't put it in the mempool or chain; we just expose the UTXO for dev.
    let tx = Transaction::new(
        vec![],
        vec![TxOutput {
            address: body.address.clone(),
            amount: body.amount,
        }],
    );

    let outpoint = OutPoint {
        txid: tx.txid.clone(),
        vout: 0,
    };

    {
        let mut utxo = state.utxo_set.lock().expect("mutex poisoned");
        utxo.insert(outpoint.clone(), tx.outputs[0].clone());
        debug!(
            "FAUCET - inserted UTXO {{ txid: {}, vout: 0 }} -> {{ addr: {}, amount: {} }}; UTXO size now {}",
            tx.txid,
            body.address,
            body.amount,
            utxo.len()
        );
    }

    HttpResponse::Ok().json(FaucetResponse {
        txid: tx.txid,
        outpoints: vec![outpoint],
    })
}

/// Submit a new transaction into the mempool (with UTXO validation).
#[post("/tx/")]
pub async fn post_transaction(
    state: web::Data<AppState>,
    body: web::Json<NewTxRequest>,
) -> impl Responder {
    let t0 = Instant::now();
    debug!(
        "POST /tx/ - received: inputs={}, outputs={}",
        body.inputs.len(),
        body.outputs.len()
    );

    // Basic structure checks
    if body.outputs.is_empty() {
        warn!("POST /tx/ - rejected: no outputs");
        return HttpResponse::BadRequest().body("transaction must have at least one output");
    }
    if body.outputs.iter().any(|o| o.amount == 0) {
        warn!("POST /tx/ - rejected: output with zero amount");
        return HttpResponse::BadRequest().body("output amount must be > 0");
    }

    // Build tx
    let tx = Transaction::new(body.inputs.clone(), body.outputs.clone());
    debug!("POST /tx/ - built txid={}", tx.txid);

    // Snapshot+validation under a single short UTXO lock
    {
        let utxo = state.utxo_set.lock().expect("mutex poisoned");

        // Dump UTXO for debug
        for (i, (op, out)) in utxo.iter().enumerate() {
            debug!(
                "UTXO[{}]: {{ txid: {}, vout: {} }} -> {{ address: {}, amount: {} }}",
                i, op.txid, op.vout, out.address, out.amount
            );
        }

        // Check each input existence
        for (i, input) in tx.inputs.iter().enumerate() {
            let op = &input.outpoint;
            let exists = utxo.get(op).is_some();
            debug!(
                "TX input[{}]: looking for {{ txid: {}, vout: {} }} => {}",
                i,
                op.txid,
                op.vout,
                if exists { "FOUND" } else { "NOT FOUND" }
            );
        }

        if let Err(msg) = validate_transaction(&tx, &utxo) {
            warn!(
                "POST /tx/ - validation failed for txid={}: {}",
                tx.txid, msg
            );
            return HttpResponse::BadRequest().body(msg);
        }
    } // <— soltamos lock do UTXO aqui

    // Push to mempool
    {
        let mut mempool = state.mempool.lock().expect("mutex poisoned");
        let before = mempool.len();
        mempool.push(tx.clone());
        let after = mempool.len();
        debug!(
            "POST /tx/ - txid={} accepted into mempool (size: {} -> {})",
            tx.txid, before, after
        );
    }

    info!(
        "POST /tx/ - txid={} OK ({} ms)",
        tx.txid,
        t0.elapsed().as_millis()
    );

    HttpResponse::Ok().json(NewTxResponse { txid: tx.txid })
}

/// List current mempool (just txids to keep it compact).
#[get("/mempool/")]
pub async fn get_mempool(state: web::Data<AppState>) -> impl Responder {
    let mempool = state.mempool.lock().expect("mutex poisoned");
    let txids = mempool.iter().map(|t| t.txid.clone()).collect::<Vec<_>>();
    HttpResponse::Ok().json(MempoolResponse {
        size: mempool.len(),
        transactions: txids,
    })
}

/// UTXO-level validation (no signatures yet).
fn validate_transaction(tx: &Transaction, utxo: &UtxoSet) -> Result<(), &'static str> {
    if tx.inputs.is_empty() {
        return Err("transactions must have at least one input (use /faucet/ to create UTXOs)");
    }

    // No duplicate inputs
    let mut seen = std::collections::HashSet::<(&str, u32)>::new();
    for input in &tx.inputs {
        let key = (input.outpoint.txid.as_str(), input.outpoint.vout);
        if !seen.insert(key) {
            return Err("duplicate input outpoint in transaction");
        }
    }

    // Sum inputs and check existence + ownership + signature
    let sighash = tx.sighash();
    let mut input_sum: u128 = 0;

    for (i, input) in tx.inputs.iter().enumerate() {
        let op = &input.outpoint;

        // Must exist
        let prev_out = utxo.get(op).ok_or("referenced UTXO not found")?;

        // Ownership: address derived from pubkey must match UTXO's address
        let derived_addr = pubkey_to_address_hex(&input.pubkey)?;
        if prev_out.address != derived_addr {
            return Err("pubkey does not own referenced UTXO (address mismatch)");
        }

        // Signature presence
        if input.signature.is_empty() {
            return Err("missing signature in input");
        }

        // Verify signature
        let ok = verify_signature_hex(&input.pubkey, &input.signature, sighash)?;
        if !ok {
            return Err("invalid signature");
        }

        input_sum += prev_out.amount as u128;
    }

    // Economic: sum(inputs) >= sum(outputs)
    let output_sum: u128 = tx.outputs.iter().map(|o| o.amount as u128).sum();
    if input_sum < output_sum {
        return Err("inputs total is less than outputs total");
    }

    Ok(())
}
