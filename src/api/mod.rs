mod chain;
mod health;
pub mod models;

use actix_web::web::{self, ServiceConfig};

pub use models::AppState;

/// Register all routes under `/api/v1` (with trailing slash for each endpoint).
pub fn init_routes(cfg: &mut ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(health::health_check)
            .service(chain::get_chain)
            .service(chain::validate_chain)
            .service(chain::mine_block)
            .service(chain::get_difficulty)
            .service(chain::set_difficulty),
    );
}
