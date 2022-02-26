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
    use std::env::VarError;

    match std::env::var("PORT") {
        Ok(s) => s
            .parse()
            .expect("Failed to parse environment variable PORT."),
        Err(VarError::NotPresent) => panic!("Environment variable PORT is required."),
        Err(VarError::NotUnicode(_)) => panic!("Environment variable PORT is not unicode."),
    }
}
