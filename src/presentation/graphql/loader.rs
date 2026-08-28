use std::collections::HashMap;

use async_graphql::dataloader::Loader;

use crate::{
    presentation::{error::PresentationalError, extractor::claims::Claims},
    use_case::traits::{
        author::AuthorQueryUseCase, book::BookQueryUseCase, history::HistoryQueryUseCase,
    },
};

use super::object::{Author, AuthorOperationChange, Book, BookOperationChange};

pub struct BookChangesByOperationLoader<HQ> {
    claims: Claims,
    history_query: HQ,
}

pub struct AuthorChangesByOperationLoader<HQ> {
    claims: Claims,
    history_query: HQ,
}

impl<HQ> BookChangesByOperationLoader<HQ> {
    pub fn new(claims: Claims, history_query: HQ) -> Self {
        Self {
            claims,
            history_query,
        }
    }
}

impl<HQ> AuthorChangesByOperationLoader<HQ> {
    pub fn new(claims: Claims, history_query: HQ) -> Self {
        Self {
            claims,
            history_query,
        }
    }
}

impl<HQ: HistoryQueryUseCase> Loader<String> for BookChangesByOperationLoader<HQ> {
    type Value = Vec<BookOperationChange>;
    type Error = PresentationalError;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        Ok(self
            .history_query
            .book_changes(&self.claims.sub, keys)
            .await?
            .into_iter()
            .map(|(id, changes)| (id, changes.into_iter().map(Into::into).collect()))
            .collect())
    }
}

impl<HQ: HistoryQueryUseCase> Loader<String> for AuthorChangesByOperationLoader<HQ> {
    type Value = Vec<AuthorOperationChange>;
    type Error = PresentationalError;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        Ok(self
            .history_query
            .author_changes(&self.claims.sub, keys)
            .await?
            .into_iter()
            .map(|(id, changes)| (id, changes.into_iter().map(Into::into).collect()))
            .collect())
    }
}

pub struct AuthorLoader<AQ> {
    claims: Claims,
    author_query: AQ,
}

pub struct BooksByAuthorLoader<BQ> {
    claims: Claims,
    book_query: BQ,
}

impl<BQ> BooksByAuthorLoader<BQ> {
    pub fn new(claims: Claims, book_query: BQ) -> Self {
        Self { claims, book_query }
    }
}

impl<BQ> Loader<String> for BooksByAuthorLoader<BQ>
where
    BQ: BookQueryUseCase,
{
    type Value = Vec<Book>;
    type Error = PresentationalError;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let books_by_author = self
            .book_query
            .find_by_author_ids(&self.claims.sub, keys)
            .await?;

        Ok(books_by_author
            .into_iter()
            .map(|(author_id, books)| {
                (
                    author_id,
                    books.into_iter().map(Book::from).collect::<Vec<_>>(),
                )
            })
            .collect())
    }
}

impl<AQ> AuthorLoader<AQ> {
    pub fn new(claims: Claims, author_query: AQ) -> Self {
        Self {
            claims,
            author_query,
        }
    }
}

impl<AQ> Loader<String> for AuthorLoader<AQ>
where
    AQ: AuthorQueryUseCase,
{
    type Value = Author;
    type Error = PresentationalError;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let authors_map = self
            .author_query
            .find_by_ids(&self.claims.sub, keys)
            .await?;
        let authors_map: HashMap<String, Author> = authors_map
            .into_iter()
            .map(|(author_id, author)| (author_id, Author::from(author)))
            .collect();

        Ok(authors_map)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_graphql::dataloader::Loader;
    use mockall::predicate;
    use time::OffsetDateTime;

    use crate::{
        common::types::{BookFormat, BookStore},
        presentation::extractor::claims::Claims,
        use_case::{
            dto::{author::AuthorDto, book::BookDto},
            traits::{author::MockAuthorQueryUseCase, book::MockBookQueryUseCase},
        },
    };

    use super::{AuthorLoader, BooksByAuthorLoader};

    #[tokio::test]
    async fn author_loader_batches_keys_and_maps_authors() {
        let author_id1 = "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string();
        let author_id2 = "93090e87-b7a1-403c-974c-d74d881e83b9".to_string();
        let expected_keys = vec![author_id1.clone(), author_id2.clone()];
        let mut author_query = MockAuthorQueryUseCase::new();
        author_query
            .expect_find_by_ids()
            .with(predicate::eq("user1"), predicate::eq(expected_keys.clone()))
            .times(1)
            .returning(move |_, _| {
                Ok(HashMap::from([(
                    author_id1.clone(),
                    AuthorDto {
                        id: author_id1.clone(),
                        name: "Author 1".to_string(),
                        yomi: "おーさーわん".to_string(),
                        created_at: OffsetDateTime::UNIX_EPOCH,
                        updated_at: OffsetDateTime::UNIX_EPOCH,
                    },
                )]))
            });
        let loader = AuthorLoader::new(
            Claims {
                sub: "user1".to_string(),
                _permissions: None,
            },
            author_query,
        );

        let result = loader.load(&expected_keys).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[&expected_keys[0]].name, "Author 1");
        assert_eq!(result[&expected_keys[0]].yomi, "おーさーわん");
        assert!(!result.contains_key(&expected_keys[1]));
    }

    #[tokio::test]
    async fn author_loader_returns_empty_map() {
        let keys = vec!["006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string()];
        let mut author_query = MockAuthorQueryUseCase::new();
        author_query
            .expect_find_by_ids()
            .with(predicate::eq("user1"), predicate::eq(keys.clone()))
            .times(1)
            .returning(|_, _| Ok(HashMap::new()));
        let loader = AuthorLoader::new(
            Claims {
                sub: "user1".to_string(),
                _permissions: None,
            },
            author_query,
        );

        let result = loader.load(&keys).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn books_by_author_loader_batches_keys_and_maps_books() {
        let author_id1 = "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string();
        let author_id2 = "93090e87-b7a1-403c-974c-d74d881e83b9".to_string();
        let expected_keys = vec![author_id1.clone(), author_id2.clone()];
        let mut book_query = MockBookQueryUseCase::new();
        book_query
            .expect_find_by_author_ids()
            .with(predicate::eq("user1"), predicate::eq(expected_keys.clone()))
            .times(1)
            .returning(move |_, _| {
                Ok(HashMap::from([
                    (
                        author_id1.clone(),
                        vec![BookDto {
                            id: "a1b2c3d4-e5f6-4890-abcd-ef1234567890".to_string(),
                            title: "Book 1".to_string(),
                            author_ids: vec![author_id1.clone()],
                            isbn: String::new(),
                            read: false,
                            owned: true,
                            priority: 50,
                            format: BookFormat::Unknown,
                            store: BookStore::Unknown,
                            created_at: OffsetDateTime::UNIX_EPOCH,
                            updated_at: OffsetDateTime::UNIX_EPOCH,
                        }],
                    ),
                    (author_id2.clone(), Vec::new()),
                ]))
            });
        let loader = BooksByAuthorLoader::new(
            Claims {
                sub: "user1".to_string(),
                _permissions: None,
            },
            book_query,
        );

        let result = loader.load(&expected_keys).await.unwrap();

        assert_eq!(result[&expected_keys[0]].len(), 1);
        assert_eq!(result[&expected_keys[0]][0].title, "Book 1");
        assert!(result[&expected_keys[1]].is_empty());
    }
}
