use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use bookshelf_api::dependency_injection::{dependency_injection, QSI};
use bookshelf_api::extractors;
use bookshelf_api::presentational::controller::graphql_controller::graphql;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let auth0_config = extractors::Auth0Config::default();

    let schema = dependency_injection().await;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .app_data(auth0_config.clone())
            .wrap(Logger::default())
            .service(hello)
            .route("/graphql", web::post().to(graphql::<QSI>))
    })
    .bind(("0.0.0.0", fetch_port()))?
    .run()
    .await
}

#[get("/hello")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("hello")
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
