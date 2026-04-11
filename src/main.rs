use std::{net::SocketAddr, sync::Arc, time::Duration};

use axum::{
    Extension, Router,
    routing::{get, post},
};
use bookshelf_api::{
    dependency_injection::{MI, QI, dependency_injection},
    presentation::handler::graphql::{graphql_handler, graphql_playground_handler},
    presentation::handler::user::me_handler,
    presentation::{app_state::AppState, extractor::claims::JwtConfig},
};
use http::{
    HeaderValue, Method,
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tower_http::{cors::CorsLayer, trace::DefaultOnRequest};
use tracing::Level;

use anyhow::Context as _;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt::init();

    let db_url = fetch_database_url()?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    let (query_use_case, schema) = dependency_injection(pool);

    let jwt_config = JwtConfig::from_env()?;
    let jwks_cache = moka::future::Cache::builder()
        .max_capacity(1)
        .time_to_live(Duration::from_hours(1))
        .build();
    let state = Arc::new(AppState {
        jwt_config,
        jwks_cache,
    });

    let allowed_origins = fetch_allowed_origins()?
        .into_iter()
        .map(|origin| {
            origin
                .parse::<HeaderValue>()
                .with_context(|| format!("invalid ALLOWED_ORIGINS value: \"{origin}\""))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let cors_layer = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(vec![AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    // build our application with routes
    let app = Router::new()
        .route("/", get(|| async { "OK" }))
        .route("/me", get(me_handler))
        .route("/graphql", post(graphql_handler::<QI, MI>))
        .route("/graphql/playground", get(graphql_playground_handler))
        .route("/health", get(|| async { "OK" }))
        .with_state(state)
        .layer(
            ServiceBuilder::new()
                .layer(Extension(query_use_case))
                .layer(Extension(schema))
                .layer(
                    TraceLayer::new_for_http()
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(cors_layer),
        );

    let port = fetch_port()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server started on port {}", port);
    axum::serve(listener, app).await?;

    Ok(())
}

fn fetch_port() -> Result<u16, anyhow::Error> {
    std::env::var("PORT")
        .context("environment variable PORT is required")?
        .parse()
        .context("failed to parse PORT as a port number")
}

fn fetch_database_url() -> Result<String, anyhow::Error> {
    std::env::var("DATABASE_URL").context("environment variable DATABASE_URL is required")
}

fn fetch_allowed_origins() -> Result<Vec<String>, anyhow::Error> {
    Ok(std::env::var("ALLOWED_ORIGINS")
        .context("environment variable ALLOWED_ORIGINS is required")?
        .split(',')
        .map(|s| s.to_owned())
        .collect())
}
