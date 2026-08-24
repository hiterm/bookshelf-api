use std::collections::HashMap;

use async_graphql::dataloader::Loader;

use crate::{
    presentation::{error::PresentationalError, extractor::claims::Claims},
    use_case::traits::{
        author::AuthorQueryUseCase, book::BookQueryUseCase, event::EventQueryUseCase,
    },
};

use super::object::{Author, AuthorEventEntry, Book, BookEventEntry};

pub struct AuthorLoader<AQ> {
    claims: Claims,
    author_query: AQ,
}

pub struct BooksByAuthorLoader<BQ> {
    claims: Claims,
    book_query: BQ,
}

pub struct BookEventsByEventSetLoader<EQ> {
    claims: Claims,
    event_query: EQ,
}

pub struct AuthorEventsByEventSetLoader<EQ> {
    claims: Claims,
    event_query: EQ,
}

impl<EQ> BookEventsByEventSetLoader<EQ> {
    pub fn new(claims: Claims, event_query: EQ) -> Self {
        Self {
            claims,
            event_query,
        }
    }
}

impl<EQ> AuthorEventsByEventSetLoader<EQ> {
    pub fn new(claims: Claims, event_query: EQ) -> Self {
        Self {
            claims,
            event_query,
        }
    }
}

impl<EQ> Loader<String> for BookEventsByEventSetLoader<EQ>
where
    EQ: EventQueryUseCase,
{
    type Value = Vec<BookEventEntry>;
    type Error = PresentationalError;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let events = self
            .event_query
            .list_book_events_by_event_set_ids(&self.claims.sub, keys)
            .await?;
        let mut grouped = keys
            .iter()
            .cloned()
            .map(|key| (key, Vec::new()))
            .collect::<HashMap<_, _>>();
        for event in events {
            grouped
                .entry(event.event_set_id.clone())
                .or_default()
                .push(BookEventEntry::from(event));
        }
        Ok(grouped)
    }
}

impl<EQ> Loader<String> for AuthorEventsByEventSetLoader<EQ>
where
    EQ: EventQueryUseCase,
{
    type Value = Vec<AuthorEventEntry>;
    type Error = PresentationalError;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let events = self
            .event_query
            .list_author_events_by_event_set_ids(&self.claims.sub, keys)
            .await?;
        let mut grouped = keys
            .iter()
            .cloned()
            .map(|key| (key, Vec::new()))
            .collect::<HashMap<_, _>>();
        for event in events {
            grouped
                .entry(event.event_set_id.clone())
                .or_default()
                .push(AuthorEventEntry::from(event));
        }
        Ok(grouped)
    }
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
            dto::{
                author::AuthorDto,
                book::BookDto,
                event::{AuthorEventDto, BookEventDto},
            },
            traits::{
                author::MockAuthorQueryUseCase, book::MockBookQueryUseCase,
                event::MockEventQueryUseCase,
            },
        },
    };

    use super::{
        AuthorEventsByEventSetLoader, AuthorLoader, BookEventsByEventSetLoader, BooksByAuthorLoader,
    };

    fn claims() -> Claims {
        Claims {
            sub: "user1".to_string(),
            _permissions: None,
        }
    }

    fn book_event(event_id: i64, event_set_id: String) -> BookEventDto {
        BookEventDto {
            event_id,
            event_set_id,
            operation: "create".to_string(),
            book_id: format!("book-{event_id}"),
            title: Some(format!("Book {event_id}")),
            author_ids: Vec::new(),
            isbn: None,
            read: None,
            owned: None,
            priority: None,
            format: None,
            store: None,
            book_created_at: None,
            book_updated_at: None,
            changed_at: OffsetDateTime::UNIX_EPOCH,
            extra: None,
        }
    }

    fn author_event(event_id: i64, event_set_id: String) -> AuthorEventDto {
        AuthorEventDto {
            event_id,
            event_set_id,
            operation: "create".to_string(),
            author_id: format!("author-{event_id}"),
            name: Some(format!("Author {event_id}")),
            yomi: None,
            author_created_at: None,
            author_updated_at: None,
            changed_at: OffsetDateTime::UNIX_EPOCH,
            extra: None,
        }
    }

    #[tokio::test]
    async fn book_events_loader_batches_keys_and_includes_empty_values() {
        let keys = vec![
            "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
            "93090e87-b7a1-403c-974c-d74d881e83b9".to_string(),
            "278935cf-ed83-4346-9b35-b84bbdb630c0".to_string(),
        ];
        let expected_keys = keys.clone();
        let returned_keys = keys.clone();
        let mut event_query = MockEventQueryUseCase::new();
        event_query
            .expect_list_book_events_by_event_set_ids()
            .with(predicate::eq("user1"), predicate::eq(expected_keys))
            .times(1)
            .return_once(move |_, _| {
                Ok(vec![
                    book_event(1, returned_keys[0].clone()),
                    book_event(2, returned_keys[1].clone()),
                ])
            });
        let loader = BookEventsByEventSetLoader::new(claims(), event_query);

        let result = loader.load(&keys).await.unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[&keys[0]][0].event_id.as_str(), "1");
        assert_eq!(result[&keys[1]][0].event_id.as_str(), "2");
        assert!(result[&keys[2]].is_empty());
    }

    #[tokio::test]
    async fn author_events_loader_batches_keys_and_includes_empty_values() {
        let keys = vec![
            "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
            "93090e87-b7a1-403c-974c-d74d881e83b9".to_string(),
            "278935cf-ed83-4346-9b35-b84bbdb630c0".to_string(),
        ];
        let expected_keys = keys.clone();
        let returned_keys = keys.clone();
        let mut event_query = MockEventQueryUseCase::new();
        event_query
            .expect_list_author_events_by_event_set_ids()
            .with(predicate::eq("user1"), predicate::eq(expected_keys))
            .times(1)
            .return_once(move |_, _| {
                Ok(vec![
                    author_event(1, returned_keys[0].clone()),
                    author_event(2, returned_keys[1].clone()),
                ])
            });
        let loader = AuthorEventsByEventSetLoader::new(claims(), event_query);

        let result = loader.load(&keys).await.unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[&keys[0]][0].event_id.as_str(), "1");
        assert_eq!(result[&keys[1]][0].event_id.as_str(), "2");
        assert!(result[&keys[2]].is_empty());
    }

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
