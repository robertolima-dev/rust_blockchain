use actix_web::{HttpResponse, Responder, get};

/// Health check (trailing slash)
#[get("/health/")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("API is up and running 🦀")
}
