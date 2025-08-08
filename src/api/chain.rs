use actix_web::{HttpResponse, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::blockchain::{Blockchain, DEFAULT_DIFFICULTY};

/// Shared application state with an in-memory blockchain.
pub struct AppState {
    pub blockchain: Mutex<Blockchain>,
}

#[derive(Serialize)]
struct ChainResponse<'a> {
    length: usize,
    difficulty: u32,
    chain: &'a [crate::blockchain::Block],
}

#[derive(Serialize)]
struct ValidateResponse {
    valid: bool,
    length: usize,
    difficulty: u32,
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

/// Get the full blockchain.
#[get("/api/v1/chain/")]
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
#[get("/api/v1/validate/")]
pub async fn validate_chain(state: web::Data<AppState>) -> impl Responder {
    let bc = state.blockchain.lock().expect("mutex poisoned");
    let resp = ValidateResponse {
        valid: bc.is_valid_chain(),
        length: bc.len(),
        difficulty: bc.difficulty(),
    };
    HttpResponse::Ok().json(resp)
}

/// Mine a new block with provided data.
#[post("/api/v1/mine/")]
pub async fn mine_block(state: web::Data<AppState>, req: web::Json<MineRequest>) -> impl Responder {
    let mut bc = state.blockchain.lock().expect("mutex poisoned");
    let b = bc.mine_block(req.into_inner().data);
    let resp = MineResponse {
        mined_index: b.index,
        hash: b.hash.clone(),
        nonce: b.nonce,
        difficulty: bc.difficulty(),
    };
    HttpResponse::Ok().json(resp)
}

/// Get current PoW difficulty.
#[get("/api/v1/difficulty/")]
pub async fn get_difficulty(state: web::Data<AppState>) -> impl Responder {
    let bc = state.blockchain.lock().expect("mutex poisoned");
    HttpResponse::Ok().json(DifficultyResponse {
        difficulty: bc.difficulty(),
    })
}

/// Update PoW difficulty (affects future blocks only).
#[post("/api/v1/difficulty/")]
pub async fn set_difficulty(
    state: web::Data<AppState>,
    body: web::Json<SetDifficultyRequest>,
) -> impl Responder {
    if body.difficulty > 6 {
        // Keep a soft cap for dev to prevent extremely slow mining.
        return HttpResponse::BadRequest().body("difficulty too high for dev mode (max 6)");
    }
    let mut bc = state.blockchain.lock().expect("mutex poisoned");
    bc.set_difficulty(body.difficulty);
    HttpResponse::Ok().json(DifficultyResponse {
        difficulty: bc.difficulty(),
    })
}

/// Helper to construct default AppState (used by main)
impl Default for AppState {
    fn default() -> Self {
        Self {
            blockchain: Mutex::new(Blockchain::new(DEFAULT_DIFFICULTY)),
        }
    }
}
