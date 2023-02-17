use actix_web::http::{header, StatusCode};
use actix_web::{web, HttpResponse, Responder};
use mime_guess::from_path;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "frontend/frontend/"]
struct Asset;

pub async fn redirect_to_frontend() -> impl Responder {
    let target = "/frontend/";

    HttpResponse::Ok()
        .status(StatusCode::PERMANENT_REDIRECT)
        .append_header((header::LOCATION, target))
        .finish()
}

pub async fn frontend_serve(path: web::Path<String>) -> impl Responder {
    let path = if path.as_str().is_empty() {
        "index.html"
    } else {
        path.as_str()
    };
    match Asset::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}
