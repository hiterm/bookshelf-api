use async_graphql::{EmptySubscription, Schema};
use sqlx::postgres::PgPoolOptions;

use crate::{
    infrastructure::{author_repository::PgAuthorRepository, user_repository::PgUserRepository},
    presentational::graphql::{mutation::Mutation, query::Query, schema::build_schema},
    use_case::interactor::{
        author::CreateAuthorInteractor, mutation::MutationInteractor, query::QueryInteractor,
        user::RegisterUserInteractor,
    },
};

pub type QI = QueryInteractor<PgUserRepository, PgAuthorRepository>;
pub type MI = MutationInteractor<
    RegisterUserInteractor<PgUserRepository>,
    CreateAuthorInteractor<PgAuthorRepository>,
>;

pub async fn dependency_injection() -> Schema<Query<QI>, Mutation<MI>, EmptySubscription> {
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
        user_repository: user_repository.clone(),
        author_repository: author_repository.clone(),
    };
    let query = Query::new(query_use_case);

    let register_user_use_case = RegisterUserInteractor::new(user_repository);
    let create_author_use_case = CreateAuthorInteractor::new(author_repository);
    let mutation_use_case = MutationInteractor::new(register_user_use_case, create_author_use_case);
    let mutation = Mutation::new(mutation_use_case);

    build_schema(query, mutation)
}

fn fetch_database_url() -> String {
    use std::env::VarError;

    match std::env::var("DATABASE_URL") {
        Ok(s) => s,
        Err(VarError::NotPresent) => panic!("Environment variable DATABASE_URL is required."),
        Err(VarError::NotUnicode(_)) => panic!("Environment variable DATABASE_URL is not unicode."),
    }
}
