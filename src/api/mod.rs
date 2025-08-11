mod chain;
mod health;
pub mod models;
mod tx;

use actix_web::web::{self, ServiceConfig};

pub use models::AppState;

pub fn init_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(health::health_check)
            .service(chain::get_chain)
            .service(chain::validate_chain)
            .service(chain::mine_block)
            .service(chain::get_difficulty)
            .service(chain::set_difficulty)
            .service(tx::post_faucet) // NEW
            .service(tx::post_transaction) // REDONE
            .service(tx::get_mempool),
    );
}
