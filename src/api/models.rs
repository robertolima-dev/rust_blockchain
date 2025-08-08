use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::blockchain::Blockchain;

/// Shared application state with an in-memory blockchain.
pub struct AppState {
    pub blockchain: Mutex<Blockchain>,
}

impl Default for AppState {
    fn default() -> Self {
        use crate::blockchain::DEFAULT_DIFFICULTY;
        Self {
            blockchain: Mutex::new(Blockchain::new(DEFAULT_DIFFICULTY)),
        }
    }
}

/* ---------- Response/Request Models ---------- */

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

#[derive(Deserialize)]
pub struct MineRequest {
    pub data: String,
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
