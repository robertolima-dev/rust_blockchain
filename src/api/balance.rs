use actix_web::{HttpResponse, Responder, get, web};

use super::models::{AppState, BalanceResponse};

#[get("/balance/{address}/")]
pub async fn get_balance(state: web::Data<AppState>, path: web::Path<(String,)>) -> impl Responder {
    let address = path.into_inner().0;

    let (mut sum, mut count) = (0u128, 0usize);
    {
        let utxo = state.utxo_set.lock().expect("mutex poisoned");
        for (_op, out) in utxo.iter() {
            if out.address == address {
                sum += out.amount as u128;
                count += 1;
            }
        }
    }

    HttpResponse::Ok().json(BalanceResponse {
        address,
        balance: sum,
        utxos: count,
    })
}
