use async_graphql::{EmptySubscription, Schema};
use sqlx::{Pool, Postgres};

use crate::{
    infrastructure::{
        author_repository::PgAuthorRepository, book_repository::PgBookRepository,
        user_repository::PgUserRepository,
    },
    presentational::graphql::{mutation::Mutation, query::Query, schema::build_schema},
    use_case::interactor::{
        author::CreateAuthorInteractor, mutation::MutationInteractor, query::QueryInteractor,
        user::RegisterUserInteractor,
    },
};

pub type QI = QueryInteractor<PgUserRepository, PgBookRepository, PgAuthorRepository>;
pub type MI = MutationInteractor<
    RegisterUserInteractor<PgUserRepository>,
    CreateAuthorInteractor<PgAuthorRepository>,
>;

pub async fn dependency_injection(
    pool: Pool<Postgres>,
) -> Schema<Query<QI>, Mutation<MI>, EmptySubscription> {
    let user_repository = PgUserRepository::new(pool.clone());
    let book_repository = PgBookRepository::new(pool.clone());
    let author_repository = PgAuthorRepository::new(pool.clone());

    let query_use_case = QueryInteractor {
        user_repository: user_repository.clone(),
        book_repository: book_repository.clone(),
        author_repository: author_repository.clone(),
    };
    let register_user_use_case = RegisterUserInteractor::new(user_repository);
    let create_author_use_case = CreateAuthorInteractor::new(author_repository);
    let mutation_use_case = MutationInteractor::new(register_user_use_case, create_author_use_case);

    let query = Query::new(query_use_case);
    let mutation = Mutation::new(mutation_use_case);

    build_schema(query, mutation)
}
