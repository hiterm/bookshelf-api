use async_graphql::{EmptySubscription, Schema};
use sqlx::{Pool, Postgres};

use crate::{
    infrastructure::{
        author_event_repository::PgAuthorEventRepository, author_repository::PgAuthorRepository,
        book_event_repository::PgBookEventRepository, book_repository::PgBookRepository,
        event_set_repository::PgEventSetRepository, transaction::PgTransactionManager,
        user_repository::PgUserRepository,
    },
    presentation::graphql::{mutation::Mutation, query::Query, schema::build_schema},
    use_case::interactor::{
        author::{AuthorCommandInteractor, AuthorQueryInteractor},
        book::{BookCommandInteractor, BookQueryInteractor},
        event::EventQueryInteractor,
        user::{UserCommandInteractor, UserQueryInteractor},
    },
};

pub type UQ = UserQueryInteractor<PgUserRepository>;
pub type UC = UserCommandInteractor<PgUserRepository>;
pub type BQ = BookQueryInteractor<PgBookRepository>;
pub type BC = BookCommandInteractor<
    PgBookRepository,
    PgAuthorRepository,
    PgBookEventRepository,
    PgTransactionManager,
>;
pub type AQ = AuthorQueryInteractor<PgAuthorRepository>;
pub type AC = AuthorCommandInteractor<
    PgAuthorRepository,
    PgBookRepository,
    PgAuthorEventRepository,
    PgTransactionManager,
>;
pub type EQ =
    EventQueryInteractor<PgBookEventRepository, PgAuthorEventRepository, PgEventSetRepository>;

pub type AppQuery = Query<UQ, BQ, AQ, EQ>;
pub type AppMutation = Mutation<UC, BC, AC>;
pub type AppSchema = Schema<AppQuery, AppMutation, EmptySubscription>;

pub fn dependency_injection(pool: Pool<Postgres>) -> (AQ, BQ, AppSchema) {
    let user_repository = PgUserRepository::new(pool.clone());
    let book_repository = PgBookRepository::new(pool.clone());
    let author_repository = PgAuthorRepository::new(pool.clone());
    let book_event_repository = PgBookEventRepository::new(pool.clone());
    let author_event_repository = PgAuthorEventRepository::new(pool.clone());
    let event_set_repository = PgEventSetRepository::new(pool.clone());
    let transaction_manager = PgTransactionManager::new(pool);

    let user_query = UserQueryInteractor::new(user_repository.clone());
    let user_command = UserCommandInteractor::new(user_repository);
    let book_query = BookQueryInteractor::new(book_repository.clone());
    let book_command = BookCommandInteractor::new(
        book_repository.clone(),
        author_repository.clone(),
        book_event_repository.clone(),
        transaction_manager.clone(),
    );
    let author_query = AuthorQueryInteractor::new(author_repository.clone());
    let author_command = AuthorCommandInteractor::new(
        author_repository,
        book_repository,
        author_event_repository.clone(),
        transaction_manager,
    );
    let event_query = EventQueryInteractor::new(
        book_event_repository,
        author_event_repository,
        event_set_repository,
    );

    let query = Query::new(
        user_query,
        book_query.clone(),
        author_query.clone(),
        event_query,
    );
    let mutation = Mutation::new(user_command, book_command, author_command);
    let schema = build_schema(query, mutation);

    (author_query, book_query, schema)
}
