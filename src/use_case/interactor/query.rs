use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::{PgPool, Row};
use time::OffsetDateTime;

use crate::{
    domain::{
        entity::{author::AuthorId, book::BookId, event_set::EventSetId, user::UserId},
        error::DomainError,
        repository::{
            author_event_repository::AuthorEventRepository, author_repository::AuthorRepository,
            book_event_repository::BookEventRepository, book_repository::BookRepository,
            user_repository::UserRepository,
        },
    },
    use_case::{
        dto::{
            author::AuthorDto,
            book::BookDto,
            event::{AuthorEventDto, BookEventDto, EventSetDto},
            user::UserDto,
        },
        error::UseCaseError,
        traits::query::QueryUseCase,
    },
};

#[derive(Debug, Clone)]
pub struct QueryInteractor<UR, BR, AR, BER, AER> {
    pub user_repository: UR,
    pub book_repository: BR,
    pub author_repository: AR,
    pub book_event_repository: BER,
    pub author_event_repository: AER,
    pub pool: PgPool,
}

#[async_trait]
impl<UR, BR, AR, BER, AER> QueryUseCase for QueryInteractor<UR, BR, AR, BER, AER>
where
    UR: UserRepository,
    BR: BookRepository,
    AR: AuthorRepository,
    BER: BookEventRepository,
    AER: AuthorEventRepository,
{
    async fn find_user_by_id(&self, raw_user_id: &str) -> Result<Option<UserDto>, UseCaseError> {
        let user_id = UserId::new(raw_user_id.to_string())?;
        let user = self.user_repository.find_by_id(&user_id).await?;

        Ok(user.map(|user| UserDto::new(user.id.into_string())))
    }

    async fn find_book_by_id(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<Option<BookDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_id)?;
        let mut conn = self.pool.acquire().await?;
        let book = self
            .book_repository
            .find_by_id(&mut conn, &user_id, &book_id)
            .await?;
        let book = book.map(BookDto::from);
        Ok(book)
    }

    async fn find_all_books(&self, user_id: &str) -> Result<Vec<BookDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let mut conn = self.pool.acquire().await?;
        let books = self.book_repository.find_all(&mut conn, &user_id).await?;
        let books: Vec<BookDto> = books.into_iter().map(BookDto::from).collect();
        Ok(books)
    }

    async fn find_author_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Option<AuthorDto>, UseCaseError> {
        let raw_user_id = user_id;
        let raw_author_id = author_id;
        let user_id = UserId::new(raw_user_id.to_string())?;
        let author_id = AuthorId::try_from(raw_author_id)?;
        let mut conn = self.pool.acquire().await?;
        let author = self
            .author_repository
            .find_by_id(&mut conn, &user_id, &author_id)
            .await?;

        Ok(author.map(AuthorDto::from))
    }

    async fn find_all_authors(&self, user_id: &str) -> Result<Vec<AuthorDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let mut conn = self.pool.acquire().await?;
        let authors = self.author_repository.find_all(&mut conn, &user_id).await?;
        let authors: Vec<AuthorDto> = authors.into_iter().map(AuthorDto::from).collect();
        Ok(authors)
    }

    async fn find_author_by_ids_as_hash_map(
        &self,
        user_id: &str,
        author_ids: &[String],
    ) -> Result<HashMap<String, AuthorDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_ids: Vec<AuthorId> = author_ids
            .iter()
            .map(|author_id| AuthorId::try_from(author_id.as_str()))
            .collect::<Result<Vec<AuthorId>, DomainError>>()?;
        let mut conn = self.pool.acquire().await?;
        let authors_map = self
            .author_repository
            .find_by_ids_as_hash_map(&mut conn, &user_id, &author_ids)
            .await?;
        let authors_map = authors_map
            .into_iter()
            .map(|(author_id, author)| (author_id.to_string(), author.into()))
            .collect();

        Ok(authors_map)
    }

    async fn list_book_events(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<Vec<BookEventDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let book_id = BookId::try_from(book_id)?;
        let entries = self
            .book_event_repository
            .find_by_book(&user_id, &book_id)
            .await?;
        Ok(entries.into_iter().map(BookEventDto::from).collect())
    }

    async fn list_author_events(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Vec<AuthorEventDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_id)?;
        let entries = self
            .author_event_repository
            .find_by_author(&user_id, &author_id)
            .await?;
        Ok(entries.into_iter().map(AuthorEventDto::from).collect())
    }

    async fn find_event_set_by_id(
        &self,
        user_id: &str,
        event_set_id: &str,
    ) -> Result<Option<EventSetDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let event_set_id = EventSetId::try_from(event_set_id)
            .map_err(|e| UseCaseError::Validation(e.to_string()))?;

        let row = sqlx::query(
            r#"SELECT id, user_id, operation, created_at FROM event_set WHERE id = $1 AND user_id = $2"#,
        )
        .bind(event_set_id.to_uuid())
        .bind(user_id.as_str())
        .fetch_optional(&self.pool)
        .await?;

        let dto = row.map(|r| EventSetDto {
            id: r.try_get("id").unwrap_or_default(),
            user_id: r.try_get("user_id").unwrap_or_default(),
            operation: r.try_get("operation").unwrap_or_default(),
            created_at: r.try_get("created_at").unwrap_or_else(|_| OffsetDateTime::now_utc()),
        });

        Ok(dto)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use mockall::predicate::always;
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::{
        common::types::{BookFormat, BookStore},
        domain::{
            self,
            entity::{
                author::{Author, AuthorId, AuthorName},
                book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
                event::{AuthorEvent, BookEvent, EventOperation},
                event_set::EventSetId,
                user::{User, UserId},
            },
            repository::{
                author_event_repository::MockAuthorEventRepository,
                author_repository::MockAuthorRepository,
                book_event_repository::MockBookEventRepository,
                book_repository::MockBookRepository, user_repository::MockUserRepository,
            },
        },
        use_case::{
            dto::author::AuthorDto, interactor::query::QueryInteractor, traits::query::QueryUseCase,
        },
    };

    fn make_author(id_str: &str, name: &str) -> Author {
        Author::new(
            AuthorId::try_from(id_str).unwrap(),
            AuthorName::new(name.to_string()).unwrap(),
        )
        .unwrap()
    }

    fn dummy_pool() -> sqlx::PgPool {
        let url = std::env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:password@localhost:5432/postgres".to_string());
        sqlx::PgPool::connect_lazy(&url).unwrap()
    }

    fn make_book(uuid_str: &str) -> Book {
        let uuid = Uuid::parse_str(uuid_str).unwrap();
        Book::new(
            BookId::new(uuid).unwrap(),
            BookTitle::new("Test Book".to_string()).unwrap(),
            vec![],
            Isbn::new("".to_string()).unwrap(),
            ReadFlag::new(false),
            OwnedFlag::new(true),
            Priority::new(50).unwrap(),
            BookFormat::Unknown,
            BookStore::Unknown,
            OffsetDateTime::now_utc(),
            OffsetDateTime::now_utc(),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn find_book_by_id_passes_correct_user_id_to_repository() {
        // Given
        let user_repository = MockUserRepository::new();
        let mut book_repository = MockBookRepository::new();
        let author_repository = MockAuthorRepository::new();

        let expected_user_id = "user1";
        let book_id_str = "a1b2c3d4-e5f6-4890-abcd-ef1234567890";
        let book = make_book(book_id_str);

        book_repository
            .expect_find_by_id()
            .withf(move |_, uid, _| uid.as_str() == expected_user_id)
            .returning(move |_, _, _| Ok(Some(book.clone())));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let result = query_interactor.find_book_by_id("user1", book_id_str).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn find_all_books_passes_correct_user_id_to_repository() {
        // Given
        let user_repository = MockUserRepository::new();
        let mut book_repository = MockBookRepository::new();
        let author_repository = MockAuthorRepository::new();

        let expected_user_id = "user1";

        book_repository
            .expect_find_all()
            .withf(move |_, uid| uid.as_str() == expected_user_id)
            .returning(|_, _| Ok(vec![]));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let result = query_interactor.find_all_books("user1").await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn find_author_by_id_passes_correct_user_id_to_repository() {
        // Given
        let user_repository = MockUserRepository::new();
        let book_repository = MockBookRepository::new();
        let mut author_repository = MockAuthorRepository::new();

        let expected_user_id = "user1";
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        author_repository
            .expect_find_by_id()
            .withf(move |_, uid, _| uid.as_str() == expected_user_id)
            .returning(|_, _, _| Ok(None));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let result = query_interactor
            .find_author_by_id("user1", author_id_str)
            .await;

        // Then
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn find_all_authors_passes_correct_user_id_to_repository() {
        // Given
        let user_repository = MockUserRepository::new();
        let book_repository = MockBookRepository::new();
        let mut author_repository = MockAuthorRepository::new();

        let expected_user_id = "user1";

        author_repository
            .expect_find_all()
            .withf(move |_, uid| uid.as_str() == expected_user_id)
            .returning(|_, _| Ok(vec![]));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let result = query_interactor.find_all_authors("user1").await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn find_author_by_ids_as_hash_map_passes_correct_user_id_to_repository() {
        // Given
        let user_repository = MockUserRepository::new();
        let book_repository = MockBookRepository::new();
        let mut author_repository = MockAuthorRepository::new();

        let expected_user_id = "user1";

        author_repository
            .expect_find_by_ids_as_hash_map()
            .withf(move |_, uid, _| uid.as_str() == expected_user_id)
            .returning(|_, _, _| Ok(HashMap::new()));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let result = query_interactor
            .find_author_by_ids_as_hash_map(
                "user1",
                &["006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string()],
            )
            .await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn find_author_by_id() {
        let user_repository = MockUserRepository::new();
        let book_repository = MockBookRepository::new();
        let mut author_repository = MockAuthorRepository::new();

        let user_id = "user1";
        let author_id = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author_name = "author1";

        author_repository
            .expect_find_by_id()
            .with(always(), always(), always())
            .returning(move |_, _, _| {
                Ok(Some(domain::entity::author::Author::new(
                    AuthorId::try_from(author_id).unwrap(),
                    AuthorName::new(author_name.to_string()).unwrap(),
                )?))
            });

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        let actual = query_interactor
            .find_author_by_id(user_id, author_id)
            .await
            .unwrap();

        let expected = Some(AuthorDto {
            id: author_id.to_owned(),
            name: author_name.to_owned(),
        });

        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn find_user_by_id_returns_user_when_found() {
        // Given
        let mut user_repository = MockUserRepository::new();
        let book_repository = MockBookRepository::new();
        let author_repository = MockAuthorRepository::new();

        let user_id_str = "user1";
        user_repository
            .expect_find_by_id()
            .with(always())
            .returning(move |_| {
                let uid = UserId::new(user_id_str.to_string()).unwrap();
                Ok(Some(User::new(uid)))
            });

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let actual = query_interactor.find_user_by_id("user1").await.unwrap();

        // Then
        assert!(actual.is_some());
        assert_eq!(actual.unwrap().id, "user1");
    }

    #[tokio::test]
    async fn find_user_by_id_returns_none_when_not_found() {
        // Given
        let mut user_repository = MockUserRepository::new();
        let book_repository = MockBookRepository::new();
        let author_repository = MockAuthorRepository::new();

        user_repository
            .expect_find_by_id()
            .with(always())
            .returning(|_| Ok(None));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let actual = query_interactor.find_user_by_id("user1").await.unwrap();

        // Then
        assert!(actual.is_none());
    }

    #[tokio::test]
    async fn find_book_by_id_returns_book_when_found() {
        // Given
        let user_repository = MockUserRepository::new();
        let mut book_repository = MockBookRepository::new();
        let author_repository = MockAuthorRepository::new();

        let book_id_str = "a1b2c3d4-e5f6-4890-abcd-ef1234567890";
        let book = make_book(book_id_str);

        book_repository
            .expect_find_by_id()
            .with(always(), always(), always())
            .returning(move |_, _, _| Ok(Some(book.clone())));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let actual = query_interactor
            .find_book_by_id("user1", book_id_str)
            .await
            .unwrap();

        // Then
        assert!(actual.is_some());
        assert_eq!(actual.unwrap().id, book_id_str);
    }

    #[tokio::test]
    async fn find_book_by_id_returns_none_when_not_found() {
        // Given
        let user_repository = MockUserRepository::new();
        let mut book_repository = MockBookRepository::new();
        let author_repository = MockAuthorRepository::new();

        book_repository
            .expect_find_by_id()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(None));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let actual = query_interactor
            .find_book_by_id("user1", "a1b2c3d4-e5f6-4890-abcd-ef1234567890")
            .await
            .unwrap();

        // Then
        assert!(actual.is_none());
    }

    #[tokio::test]
    async fn find_all_books_returns_list() {
        // Given
        let user_repository = MockUserRepository::new();
        let mut book_repository = MockBookRepository::new();
        let author_repository = MockAuthorRepository::new();

        let book = make_book("a1b2c3d4-e5f6-4890-abcd-ef1234567890");

        book_repository
            .expect_find_all()
            .with(always(), always())
            .returning(move |_, _| Ok(vec![book.clone()]));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let actual = query_interactor.find_all_books("user1").await.unwrap();

        // Then
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].title, "Test Book");
    }

    #[tokio::test]
    async fn find_all_authors_returns_list() {
        // Given
        let user_repository = MockUserRepository::new();
        let book_repository = MockBookRepository::new();
        let mut author_repository = MockAuthorRepository::new();

        let author = make_author("006099b4-6c42-4ec4-8645-f6bd5b63eddc", "author1");

        author_repository
            .expect_find_all()
            .with(always(), always())
            .returning(move |_, _| Ok(vec![author.clone()]));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let actual = query_interactor.find_all_authors("user1").await.unwrap();

        // Then
        assert_eq!(actual.len(), 1);
        assert_eq!(
            actual[0],
            AuthorDto {
                id: "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
                name: "author1".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn find_author_by_ids_as_hash_map_returns_map() {
        // Given
        let user_repository = MockUserRepository::new();
        let book_repository = MockBookRepository::new();
        let mut author_repository = MockAuthorRepository::new();

        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author = make_author(author_id_str, "author1");
        let author_id = AuthorId::try_from(author_id_str).unwrap();

        author_repository
            .expect_find_by_ids_as_hash_map()
            .with(always(), always(), always())
            .returning(move |_, _, _| {
                let mut map = HashMap::new();
                map.insert(author_id.clone(), author.clone());
                Ok(map)
            });

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        // When
        let actual = query_interactor
            .find_author_by_ids_as_hash_map("user1", &[author_id_str.to_string()])
            .await
            .unwrap();

        // Then
        assert_eq!(actual.len(), 1);
        assert_eq!(
            actual.get(author_id_str).unwrap(),
            &AuthorDto {
                id: author_id_str.to_string(),
                name: "author1".to_string(),
            }
        );
    }

    fn make_book_event(book_id: Uuid) -> BookEvent {
        BookEvent {
            event_id: 1,
            event_set_id: EventSetId::from(Uuid::new_v4()),
            operation: EventOperation::Update,
            book_id: BookId::new(book_id).unwrap(),
            title: Some(BookTitle::new("Old Title".to_string()).unwrap()),
            author_ids: vec![],
            isbn: Some(Isbn::new("".to_string()).unwrap()),
            read: Some(ReadFlag::new(false)),
            owned: Some(OwnedFlag::new(false)),
            priority: Some(Priority::new(50).unwrap()),
            format: Some(BookFormat::Unknown),
            store: Some(BookStore::Unknown),
            book_created_at: Some(OffsetDateTime::now_utc()),
            book_updated_at: Some(OffsetDateTime::now_utc()),
            changed_at: OffsetDateTime::now_utc(),
            extra: None,
        }
    }

    fn make_author_event(author_id: Uuid) -> AuthorEvent {
        AuthorEvent {
            event_id: 2,
            event_set_id: EventSetId::from(Uuid::new_v4()),
            operation: EventOperation::Update,
            author_id: AuthorId::new(author_id),
            name: Some("Old Name".to_string()),
            yomi: Some("".to_string()),
            author_created_at: Some(OffsetDateTime::now_utc()),
            author_updated_at: Some(OffsetDateTime::now_utc()),
            changed_at: OffsetDateTime::now_utc(),
            extra: None,
        }
    }

    #[tokio::test]
    async fn list_book_events_returns_dto_list() {
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();
        let event = make_book_event(book_uuid);

        let mut book_event_repository = MockBookEventRepository::new();
        book_event_repository
            .expect_find_by_book()
            .with(always(), always())
            .returning(move |_, _| Ok(vec![event.clone()]));

        let query_interactor = QueryInteractor {
            user_repository: MockUserRepository::new(),
            book_repository: MockBookRepository::new(),
            author_repository: MockAuthorRepository::new(),
            book_event_repository,
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        let result = query_interactor
            .list_book_events("user1", &book_id_str)
            .await;

        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].title, Some("Old Title".to_string()));
    }

    #[tokio::test]
    async fn list_book_events_returns_empty() {
        let book_uuid = Uuid::new_v4();
        let book_id_str = book_uuid.hyphenated().to_string();

        let mut book_event_repository = MockBookEventRepository::new();
        book_event_repository
            .expect_find_by_book()
            .with(always(), always())
            .returning(|_, _| Ok(vec![]));

        let query_interactor = QueryInteractor {
            user_repository: MockUserRepository::new(),
            book_repository: MockBookRepository::new(),
            author_repository: MockAuthorRepository::new(),
            book_event_repository,
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        let result = query_interactor
            .list_book_events("user1", &book_id_str)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_book_events_invalid_book_id_returns_error() {
        let query_interactor = QueryInteractor {
            user_repository: MockUserRepository::new(),
            book_repository: MockBookRepository::new(),
            author_repository: MockAuthorRepository::new(),
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        let result = query_interactor
            .list_book_events("user1", "not-a-uuid")
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn list_author_events_returns_dto_list() {
        let author_uuid = Uuid::new_v4();
        let author_id_str = author_uuid.hyphenated().to_string();
        let event = make_author_event(author_uuid);

        let mut author_event_repository = MockAuthorEventRepository::new();
        author_event_repository
            .expect_find_by_author()
            .with(always(), always())
            .returning(move |_, _| Ok(vec![event.clone()]));

        let query_interactor = QueryInteractor {
            user_repository: MockUserRepository::new(),
            book_repository: MockBookRepository::new(),
            author_repository: MockAuthorRepository::new(),
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository,
            pool: dummy_pool(),
        };

        let result = query_interactor
            .list_author_events("user1", &author_id_str)
            .await;

        assert!(result.is_ok());
        let list = result.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name, Some("Old Name".to_string()));
    }

    #[tokio::test]
    async fn list_author_events_returns_empty() {
        let author_uuid = Uuid::new_v4();
        let author_id_str = author_uuid.hyphenated().to_string();

        let mut author_event_repository = MockAuthorEventRepository::new();
        author_event_repository
            .expect_find_by_author()
            .with(always(), always())
            .returning(|_, _| Ok(vec![]));

        let query_interactor = QueryInteractor {
            user_repository: MockUserRepository::new(),
            book_repository: MockBookRepository::new(),
            author_repository: MockAuthorRepository::new(),
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository,
            pool: dummy_pool(),
        };

        let result = query_interactor
            .list_author_events("user1", &author_id_str)
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[tokio::test]
    async fn list_author_events_invalid_author_id_returns_error() {
        let query_interactor = QueryInteractor {
            user_repository: MockUserRepository::new(),
            book_repository: MockBookRepository::new(),
            author_repository: MockAuthorRepository::new(),
            book_event_repository: MockBookEventRepository::new(),
            author_event_repository: MockAuthorEventRepository::new(),
            pool: dummy_pool(),
        };

        let result = query_interactor
            .list_author_events("user1", "not-a-uuid")
            .await;

        assert!(result.is_err());
    }
}
