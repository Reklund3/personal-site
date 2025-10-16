use actix_web::{HttpResponse, Responder, http::header::ContentType};

pub async fn home() -> impl Responder {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .insert_header(("Cache-Control", "max-age=604800, must-revalidate"))
        // .body(include_str!("home.html"))
        .body(include_str!("../../../ui/dist/index.html"))
}
