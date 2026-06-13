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
        author::{CreateAuthorInteractor, DeleteAuthorInteractor, UpdateAuthorInteractor},
        book::{
            CreateBookInteractor, DeleteBookInteractor, ImportBooksInteractor, UpdateBookInteractor,
        },
        event::{RestoreAuthorInteractor, RestoreBookInteractor},
        mutation::MutationInteractor,
        query::QueryInteractor,
        user::RegisterUserInteractor,
    },
};

pub type QI = QueryInteractor<
    PgUserRepository,
    PgBookRepository,
    PgAuthorRepository,
    PgBookEventRepository,
    PgAuthorEventRepository,
    PgEventSetRepository,
>;

pub type MI = MutationInteractor<
    RegisterUserInteractor<PgUserRepository>,
    CreateBookInteractor<PgBookRepository, PgTransactionManager>,
    UpdateBookInteractor<PgBookRepository, PgTransactionManager>,
    DeleteBookInteractor<PgBookRepository, PgTransactionManager>,
    CreateAuthorInteractor<PgAuthorRepository, PgTransactionManager>,
    UpdateAuthorInteractor<PgAuthorRepository, PgTransactionManager>,
    DeleteAuthorInteractor<PgAuthorRepository, PgTransactionManager>,
    RestoreBookInteractor<PgBookRepository, PgBookEventRepository, PgTransactionManager>,
    RestoreAuthorInteractor<PgAuthorRepository, PgAuthorEventRepository, PgTransactionManager>,
    ImportBooksInteractor<PgBookRepository, PgAuthorRepository, PgTransactionManager>,
>;

pub fn dependency_injection(
    pool: Pool<Postgres>,
) -> (QI, Schema<Query<QI>, Mutation<MI>, EmptySubscription>) {
    let user_repository = PgUserRepository::new(pool.clone());
    let book_repository = PgBookRepository::new(pool.clone());
    let author_repository = PgAuthorRepository::new(pool.clone());
    let book_event_repository = PgBookEventRepository::new(pool.clone());
    let author_event_repository = PgAuthorEventRepository::new(pool.clone());
    let event_set_repository = PgEventSetRepository::new(pool.clone());
    let transaction_manager = PgTransactionManager::new(pool);

    let query_use_case = QueryInteractor {
        user_repository: user_repository.clone(),
        book_repository: book_repository.clone(),
        author_repository: author_repository.clone(),
        book_event_repository: book_event_repository.clone(),
        author_event_repository: author_event_repository.clone(),
        event_set_repository,
    };
    let register_user_use_case = RegisterUserInteractor::new(user_repository);
    let create_book_use_case =
        CreateBookInteractor::new(book_repository.clone(), transaction_manager.clone());
    let update_book_use_case =
        UpdateBookInteractor::new(book_repository.clone(), transaction_manager.clone());
    let delete_book_use_case =
        DeleteBookInteractor::new(book_repository.clone(), transaction_manager.clone());
    let create_author_use_case =
        CreateAuthorInteractor::new(author_repository.clone(), transaction_manager.clone());
    let update_author_use_case =
        UpdateAuthorInteractor::new(author_repository.clone(), transaction_manager.clone());
    let delete_author_use_case =
        DeleteAuthorInteractor::new(author_repository.clone(), transaction_manager.clone());
    let import_books_use_case = ImportBooksInteractor::new(
        book_repository.clone(),
        author_repository.clone(),
        transaction_manager.clone(),
    );
    let restore_book_use_case = RestoreBookInteractor::new(
        book_repository,
        book_event_repository,
        transaction_manager.clone(),
    );
    let restore_author_use_case = RestoreAuthorInteractor::new(
        author_repository,
        author_event_repository,
        transaction_manager,
    );

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
        import_books_use_case,
    );

    let query = Query::new(query_use_case.clone());
    let mutation = Mutation::new(mutation_use_case);

    let schema = build_schema(query, mutation);

    (query_use_case, schema)
}
