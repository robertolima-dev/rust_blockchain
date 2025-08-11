use actix_web::{HttpResponse, Responder, get, post, web};

use super::models::{
    AppState, ChainResponse, DifficultyResponse, MineRequest, MineResponse, SetDifficultyRequest,
    ValidateResponse,
};
use crate::transaction::{Transaction, TxInput, TxOutput};

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

/// Mine a new block with provided data (temporary: wraps data into a placeholder transaction).
#[post("/mine/")]
pub async fn mine_block(state: web::Data<AppState>, req: web::Json<MineRequest>) -> impl Responder {
    // Placeholder tx: no inputs, zero-amount output (accepted by validator policy)
    let placeholder_tx = Transaction::new(
        vec![], // no inputs
        vec![TxOutput {
            address: req.data.clone(),
            amount: 0,
        }],
    );

    let mut bc = state.blockchain.lock().expect("mutex poisoned");
    let b = bc.mine_block(vec![placeholder_tx]);
    let resp = MineResponse {
        mined_index: b.index,
        hash: b.hash.clone(),
        nonce: b.nonce,
        difficulty: bc.difficulty(),
    };
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
        // Keep a soft cap for dev to prevent extremely slow mining.
        return HttpResponse::BadRequest().body("difficulty too high for dev mode (max 6)");
    }
    let mut bc = state.blockchain.lock().expect("mutex poisoned");
    bc.set_difficulty(body.difficulty);
    HttpResponse::Ok().json(DifficultyResponse {
        difficulty: bc.difficulty(),
    })
}
