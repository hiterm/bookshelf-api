mod extractors;
mod types;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::extractors::Claims;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let db_url = fetch_database_url();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    let auth0_config = extractors::Auth0Config::default();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .app_data(auth0_config.clone())
            .service(hello)
    })
    .bind(("0.0.0.0", fetch_port()))?
    .run()
    .await
}

#[get("/")]
async fn root(pool: web::Data<PgPool>, _claims: Claims) -> impl Responder {
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(pool.get_ref())
        .await
        .unwrap();
    HttpResponse::Ok().body(row.0.to_string())
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

fn fetch_database_url() -> String {
    use std::env::VarError;

    match std::env::var("DATABASE_URL") {
        Ok(s) => s,
        Err(VarError::NotPresent) => panic!("Environment variable DATABASE_URL is required."),
        Err(VarError::NotUnicode(_)) => panic!("Environment variable DATABASE_URL is not unicode."),
    }
}
