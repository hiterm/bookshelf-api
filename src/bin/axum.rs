use std::{net::SocketAddr, sync::Arc};

use axum::{
    routing::{get, post},
    Extension, Router,
};
use bookshelf_api::{
    dependency_injection::{dependency_injection, MI, QI},
    presentation::handler::graphql::{graphql_handler, graphql_playground_handler},
    presentation::{app_state::AppState, extractor::claims::Auth0Config},
};
use http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt::init();

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

    let auth0_config = Auth0Config::default();
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
        .route("/", get(|| async { "OK" }))
        .route("/graphql", post(graphql_handler::<QI, MI>))
        .route("/graphql/playground", get(graphql_playground_handler))
        .route("/health", get(|| async { "OK" }))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(Extension(query_use_case))
                .layer(Extension(schema))
                .layer(TraceLayer::new_for_http()),
        )
        .layer(cors_layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], fetch_port()));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
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
