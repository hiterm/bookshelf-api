use std::sync::Arc;

use axum::{
    routing::{get, post},
    Extension, Router,
};
use bookshelf_api::{
    dependency_injection::{dependency_injection, MI, QI},
    extractors::{self, claims::AppState},
    presentation::handler::graphql::{graphql_handler, graphql_playground_handler},
};
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
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
    let state = Arc::new(AppState { auth0_config });

    let allowed_origins: Vec<_> = fetch_allowed_origins()
        .into_iter()
        .map(|origin| origin.parse::<HeaderValue>().unwrap())
        .collect();
    let cors_layer = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::POST])
        .allow_headers(vec![AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/graphql", post(graphql_handler::<QI, MI>))
        .route("/graphql/playground", get(graphql_playground_handler))
        .with_state(state)
        .layer(Extension(query_use_case))
        .layer(Extension(schema))
        .layer(cors_layer);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:8080".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
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
