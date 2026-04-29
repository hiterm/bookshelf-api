use async_graphql::{EmptySubscription, Schema};
use sqlx::{Pool, Postgres};

use crate::{
    infrastructure::{
        author_history_repository::PgAuthorHistoryRepository,
        author_repository::PgAuthorRepository,
        book_history_repository::PgBookHistoryRepository,
        book_repository::PgBookRepository,
        user_repository::PgUserRepository,
    },
    presentation::graphql::{mutation::Mutation, query::Query, schema::build_schema},
    use_case::interactor::{
        author::{CreateAuthorInteractor, DeleteAuthorInteractor, UpdateAuthorInteractor},
        book::{CreateBookInteractor, DeleteBookInteractor, UpdateBookInteractor},
        history::{RestoreAuthorInteractor, RestoreBookInteractor},
        mutation::MutationInteractor,
        query::QueryInteractor,
        user::RegisterUserInteractor,
    },
};

pub type QI = QueryInteractor<
    PgUserRepository,
    PgBookRepository,
    PgAuthorRepository,
    PgBookHistoryRepository,
    PgAuthorHistoryRepository,
>;

pub type MI = MutationInteractor<
    RegisterUserInteractor<PgUserRepository>,
    CreateBookInteractor<PgBookRepository>,
    UpdateBookInteractor<PgBookRepository>,
    DeleteBookInteractor<PgBookRepository>,
    CreateAuthorInteractor<PgAuthorRepository>,
    UpdateAuthorInteractor<PgAuthorRepository>,
    DeleteAuthorInteractor<PgAuthorRepository>,
    RestoreBookInteractor<PgBookRepository, PgBookHistoryRepository>,
    RestoreAuthorInteractor<PgAuthorRepository, PgAuthorHistoryRepository>,
>;

pub fn dependency_injection(
    pool: Pool<Postgres>,
) -> (QI, Schema<Query<QI>, Mutation<MI>, EmptySubscription>) {
    let user_repository = PgUserRepository::new(pool.clone());
    let book_repository = PgBookRepository::new(pool.clone());
    let author_repository = PgAuthorRepository::new(pool.clone());
    let book_history_repository = PgBookHistoryRepository::new(pool.clone());
    let author_history_repository = PgAuthorHistoryRepository::new(pool);

    let query_use_case = QueryInteractor {
        user_repository: user_repository.clone(),
        book_repository: book_repository.clone(),
        author_repository: author_repository.clone(),
        book_history_repository: book_history_repository.clone(),
        author_history_repository: author_history_repository.clone(),
    };
    let register_user_use_case = RegisterUserInteractor::new(user_repository);
    let create_book_use_case = CreateBookInteractor::new(book_repository.clone());
    let update_book_use_case = UpdateBookInteractor::new(book_repository.clone());
    let delete_book_use_case = DeleteBookInteractor::new(book_repository.clone());
    let create_author_use_case = CreateAuthorInteractor::new(author_repository.clone());
    let update_author_use_case = UpdateAuthorInteractor::new(author_repository.clone());
    let delete_author_use_case = DeleteAuthorInteractor::new(author_repository.clone());
    let restore_book_use_case =
        RestoreBookInteractor::new(book_repository, book_history_repository);
    let restore_author_use_case =
        RestoreAuthorInteractor::new(author_repository, author_history_repository);

    let mutation_use_case = MutationInteractor::new(
        register_user_use_case,
        create_book_use_case,
        update_book_use_case,
        delete_book_use_case,
        create_author_use_case,
        update_author_use_case,
        delete_author_use_case,
        restore_book_use_case,
        restore_author_use_case,
    );

    let query = Query::new(query_use_case.clone());
    let mutation = Mutation::new(mutation_use_case);

    let schema = build_schema(query, mutation);

    (query_use_case, schema)
}
