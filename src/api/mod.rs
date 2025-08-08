mod chain;
mod health;

use actix_web::web::ServiceConfig;
pub use chain::AppState;

pub fn init_routes(cfg: &mut ServiceConfig) {
    cfg.service(health::health_check);
    cfg.service(chain::get_chain);
    cfg.service(chain::validate_chain);
    cfg.service(chain::mine_block);
    cfg.service(chain::get_difficulty);
    cfg.service(chain::set_difficulty);
}
