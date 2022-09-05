use actix_cors::Cors;
use actix_web::http;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use bookshelf_api::dependency_injection::{dependency_injection, MI, QI};
use bookshelf_api::extractors;
use bookshelf_api::presentation::controller::graphql_controller::{graphql, graphql_playground};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    env_logger::init();

    let db_url = fetch_database_url();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Migration failed.");

    let (query_use_case, schema) = dependency_injection(pool);

    let auth0_config = extractors::Auth0Config::default();

    let allowed_origins = fetch_allowed_origins();

    HttpServer::new(move || {
        let mut cors = Cors::default();
        for allowed_origin in allowed_origins.iter() {
            cors = cors.allowed_origin(allowed_origin);
        }
        cors = cors.allowed_methods([http::Method::POST]).allowed_headers([
            http::header::AUTHORIZATION,
            http::header::ACCEPT,
            http::header::CONTENT_TYPE,
        ]);
        App::new()
            .app_data(web::Data::new(query_use_case.clone()))
            .app_data(web::Data::new(schema.clone()))
            .app_data(auth0_config.clone())
            .wrap(Logger::default())
            .wrap(cors)
            .service(hello)
            .service(graphql_playground)
            .route("/graphql", web::post().to(graphql::<QI, MI>))
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

fn fetch_database_url() -> String {
    use std::env::VarError;

    match std::env::var("DATABASE_URL") {
        Ok(s) => s,
        Err(VarError::NotPresent) => panic!("Environment variable DATABASE_URL is required."),
        Err(VarError::NotUnicode(_)) => panic!("Environment variable DATABASE_URL is not unicode."),
    }
}

fn fetch_allowed_origins() -> Vec<String> {
    use std::env::VarError;

    match std::env::var("ALLOWED_ORIGINS") {
        Ok(s) => s.split(',').map(|s| s.to_owned()).collect(),
        Err(VarError::NotPresent) => panic!("Environment variable ALLOWED_ORIGINS is required."),
        Err(VarError::NotUnicode(_)) => {
            panic!("Environment variable ALLOWED_ORIGINS is not unicode.")
        }
    }
}
