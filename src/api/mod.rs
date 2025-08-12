mod balance;
mod chain;
mod health;
pub mod models;
mod stats;
mod tx;
mod wallet; // <- add

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
            .service(tx::post_faucet)
            .service(tx::post_transaction)
            .service(tx::get_mempool)
            .service(balance::get_balance)
            .service(stats::get_stats)
            .service(wallet::create_wallet), // <- here
    );
}
