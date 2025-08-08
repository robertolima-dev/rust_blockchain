use actix_web::{HttpResponse, Responder, get};

#[get("/api/v1/health/")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().body("API is up and running 🦀")
}
