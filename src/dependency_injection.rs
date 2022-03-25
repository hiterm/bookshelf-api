use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use sqlx::postgres::PgPoolOptions;

use crate::{
    infrastructure::{author_repository::PgAuthorRepository, user_repository::PgUserRepository},
    presentational::graphql::{query::Query, schema::build_schema},
    use_case::interactor::query::QueryInteractor,
};

pub type QI = QueryInteractor<PgUserRepository, PgAuthorRepository>;

pub async fn dependency_injection() -> Schema<Query<QI>, EmptyMutation, EmptySubscription> {
    let db_url = fetch_database_url();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await
        .unwrap();

    // TODO: newを使う
    let user_repository = PgUserRepository { pool: pool.clone() };
    let author_repository = PgAuthorRepository { pool: pool.clone() };
    let query_use_case = QueryInteractor {
        user_repository,
        author_repository,
    };
    let query = Query::new(query_use_case);

    build_schema(query)
}

fn fetch_database_url() -> String {
    use std::env::VarError;

    match std::env::var("DATABASE_URL") {
        Ok(s) => s,
        Err(VarError::NotPresent) => panic!("Environment variable DATABASE_URL is required."),
        Err(VarError::NotUnicode(_)) => panic!("Environment variable DATABASE_URL is not unicode."),
    }
}
