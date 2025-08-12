use actix_web::{HttpResponse, Responder, post};
use serde::Serialize;

use crate::wallet::generate_keypair_hex;

#[derive(Serialize)]
struct NewWalletResponse {
    private_key: String,
    public_key: String,
    address: String,
}

#[post("/wallet/new/")]
pub async fn create_wallet() -> impl Responder {
    let (sk, pk, addr) = generate_keypair_hex();
    HttpResponse::Ok().json(NewWalletResponse {
        private_key: sk,
        public_key: pk,
        address: addr,
    })
}
