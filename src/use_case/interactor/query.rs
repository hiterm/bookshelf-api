use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    domain::{
        entity::{author::AuthorId, book::BookId, user::UserId},
        error::DomainError,
        repository::{
            author_repository::AuthorRepository, book_repository::BookRepository,
            user_repository::UserRepository,
        },
    },
    use_case::{
        dto::{author::AuthorDto, book::BookDto, user::UserDto},
        error::UseCaseError,
        traits::query::QueryUseCase,
    },
};

#[derive(Debug, Clone)]
pub struct QueryInteractor<UR, BR, AR> {
    pub user_repository: UR,
    pub book_repository: BR,
    pub author_repository: AR,
}

#[async_trait]
impl<UR, BR, AR> QueryUseCase for QueryInteractor<UR, BR, AR>
where
    UR: UserRepository,
    BR: BookRepository,
    AR: AuthorRepository,
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
        let book = self.book_repository.find_by_id(&user_id, &book_id).await?;
        let book = book.map(BookDto::from);
        Ok(book)
    }

    async fn find_all_books(&self, user_id: &str) -> Result<Vec<BookDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let books = self.book_repository.find_all(&user_id).await?;
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
        let author = self
            .author_repository
            .find_by_id(&user_id, &author_id)
            .await?;

        Ok(author.map(AuthorDto::from))
    }

    async fn find_all_authors(&self, user_id: &str) -> Result<Vec<AuthorDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let authors = self.author_repository.find_all(&user_id).await?;
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
        let authors_map = self
            .author_repository
            .find_by_ids_as_hash_map(&user_id, &author_ids)
            .await?;
        let authors_map = authors_map
            .into_iter()
            .map(|(author_id, author)| (author_id.to_string(), author.into()))
            .collect();

        Ok(authors_map)
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
                user::{User, UserId},
            },
            repository::{
                author_repository::MockAuthorRepository, book_repository::MockBookRepository,
                user_repository::MockUserRepository,
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
    async fn find_author_by_id() {
        let user_repository = MockUserRepository::new();
        let book_repository = MockBookRepository::new();
        let mut author_repository = MockAuthorRepository::new();

        let user_id = "user1";
        let author_id = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author_name = "author1";

        author_repository
            .expect_find_by_id()
            .with(always(), always())
            .returning(move |_, _| {
                Ok(Some(domain::entity::author::Author::new(
                    AuthorId::try_from(author_id).unwrap(),
                    AuthorName::new(author_name.to_string()).unwrap(),
                )?))
            });

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
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
            .with(always(), always())
            .returning(move |_, _| Ok(Some(book.clone())));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
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
            .with(always(), always())
            .returning(|_, _| Ok(None));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
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
            .with(always())
            .returning(move |_| Ok(vec![book.clone()]));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
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
            .with(always())
            .returning(move |_| Ok(vec![author.clone()]));

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
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
            .with(always(), always())
            .returning(move |_, _| {
                let mut map = HashMap::new();
                map.insert(author_id.clone(), author.clone());
                Ok(map)
            });

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
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
}
