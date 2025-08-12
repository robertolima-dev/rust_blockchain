use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::blockchain::Blockchain;
use crate::transaction::{Transaction, UtxoSet};

/// Shared application state with an in-memory blockchain, mempool and UTXO set.
pub struct AppState {
    pub blockchain: Mutex<Blockchain>,
    pub mempool: Mutex<Vec<Transaction>>,
    pub utxo_set: Mutex<UtxoSet>,
}

impl Default for AppState {
    fn default() -> Self {
        use crate::blockchain::DEFAULT_DIFFICULTY;
        Self {
            blockchain: Mutex::new(Blockchain::new(DEFAULT_DIFFICULTY)),
            mempool: Mutex::new(Vec::new()),
            utxo_set: Mutex::new(UtxoSet::new()),
        }
    }
}

/* ---------- Chain API Models ---------- */

#[derive(Serialize)]
pub struct ChainResponse<'a> {
    pub length: usize,
    pub difficulty: u32,
    pub chain: &'a [crate::blockchain::Block],
}

#[derive(Serialize)]
pub struct ValidateResponse {
    pub valid: bool,
    pub length: usize,
    pub difficulty: u32,
}

#[derive(Serialize)]
pub struct MineResponse {
    pub mined_index: u64,
    pub hash: String,
    pub nonce: u64,
    pub difficulty: u32,
}

#[derive(Serialize)]
pub struct DifficultyResponse {
    pub difficulty: u32,
}

#[derive(Deserialize)]
pub struct SetDifficultyRequest {
    pub difficulty: u32,
}

/* ---------- TX API Models ---------- */

#[derive(Deserialize)]
pub struct NewTxRequest {
    pub inputs: Vec<crate::transaction::TxInput>,
    pub outputs: Vec<crate::transaction::TxOutput>,
}

#[derive(Serialize)]
pub struct NewTxResponse {
    pub txid: String,
}

#[derive(Serialize)]
pub struct MempoolResponse {
    pub size: usize,
    pub transactions: Vec<String>, // list txids for brevity
}

/* ---------- Faucet API Models (dev) ---------- */

#[derive(Deserialize)]
pub struct FaucetRequest {
    pub address: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct FaucetResponse {
    pub txid: String,
    pub outpoints: Vec<crate::transaction::OutPoint>,
}

#[derive(Deserialize)]
pub struct MineRequest {
    pub miner_address: String,
}

#[derive(serde::Serialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u128,
    pub utxos: usize,
}

#[derive(serde::Serialize)]
pub struct StatsResponse {
    pub height: usize,
    pub difficulty: u32,
    pub target_block_time_secs: i64,
    pub adjust_window: usize,
    pub adjust_threshold_pct: f64,
    pub last_interval_secs: Option<i64>,
    pub avg_interval_secs: Option<f64>,
    pub mempool_size: usize,
    pub utxo_size: usize,
}
