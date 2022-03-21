mod domain;
mod extractors;
mod infrastructure;
mod presentational;
mod types;
mod use_case;

use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use domain::repository::author_repository::AuthorRepository;
use infrastructure::author_repository::PgAuthorRepository;
use presentational::{
    controller::graphql_controller::graphql,
    graphql::{query::QueryRoot, query_service::QueryServiceImpl, schema::build_schema},
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use use_case::{interactor::author::ShowAuthorInteractor, use_case::author::ShowAuthorUseCase};

use crate::extractors::Claims;

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

    let auth0_config = extractors::Auth0Config::default();

    let author_repository = PgAuthorRepository { pool };
    let show_author_use_case = ShowAuthorInteractor::new(author_repository);
    let query_service = QueryServiceImpl {
        show_author_use_case,
    };
    let query = QueryRoot::new(query_service);
    let schema = build_schema(query);

    type QSI = QueryServiceImpl<ShowAuthorInteractor<PgAuthorRepository>>;

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .app_data(auth0_config.clone())
            .wrap(Logger::default())
            .service(hello)
            .route(
                "/graphql",
                web::post()
                    .to(graphql::<QSI>),
            )
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
