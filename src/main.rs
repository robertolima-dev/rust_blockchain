mod api;
mod blockchain;
mod transaction;
mod wallet;

use actix_web::{App, HttpServer, web};
use dotenvy::dotenv;
use std::env;

use api::AppState;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = dotenv();
    env_logger::init();

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port: u16 = env::var("PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8080);

    println!("⛓️ Starting blockchain API at http://{host}:{port}");

    let state = web::Data::new(AppState::default());

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .configure(api::init_routes)
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}
