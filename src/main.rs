use actix_web::{get, App, HttpResponse, HttpServer, Responder};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(hello))
        .bind(("0.0.0.0", fetch_port()))?
        .run()
        .await
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

fn fetch_port() -> u16 {
    std::env::var("PORT")
        // TODO: エラー処理を細かく
        .unwrap_or("8080".to_string())
        .parse()
        .expect("Failed to parse environment variable PORT.")
}
