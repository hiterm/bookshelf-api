use std::collections::HashMap;

use async_graphql::dataloader::Loader;

use crate::{
    presentation::{error::PresentationalError, extractor::claims::Claims},
    use_case::traits::query::QueryUseCase,
};

use super::object::{Author, Book};

pub struct AuthorLoader<QUC> {
    claims: Claims,
    query_use_case: QUC,
}

pub struct BooksByAuthorLoader<QUC> {
    claims: Claims,
    query_use_case: QUC,
}

impl<QUC> BooksByAuthorLoader<QUC> {
    pub fn new(claims: Claims, query_use_case: QUC) -> Self {
        Self {
            claims,
            query_use_case,
        }
    }
}

impl<QUC> Loader<String> for BooksByAuthorLoader<QUC>
where
    QUC: QueryUseCase,
{
    type Value = Vec<Book>;
    type Error = PresentationalError;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let books_by_author = self
            .query_use_case
            .find_books_by_author_ids_as_hash_map(&self.claims.sub, keys)
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

impl<QUC> AuthorLoader<QUC> {
    pub fn new(claims: Claims, query_use_case: QUC) -> Self {
        Self {
            claims,
            query_use_case,
        }
    }
}

impl<QUC> Loader<String> for AuthorLoader<QUC>
where
    QUC: QueryUseCase,
{
    type Value = Author;
    type Error = PresentationalError;

    async fn load(&self, keys: &[String]) -> Result<HashMap<String, Self::Value>, Self::Error> {
        let authors_map = self
            .query_use_case
            .find_author_by_ids_as_hash_map(&self.claims.sub, keys)
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
        use_case::{dto::book::BookDto, traits::query::MockQueryUseCase},
    };

    use super::BooksByAuthorLoader;

    #[tokio::test]
    async fn books_by_author_loader_batches_keys_and_maps_books() {
        let author_id1 = "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string();
        let author_id2 = "93090e87-b7a1-403c-974c-d74d881e83b9".to_string();
        let expected_keys = vec![author_id1.clone(), author_id2.clone()];
        let mut query_use_case = MockQueryUseCase::new();
        query_use_case
            .expect_find_books_by_author_ids_as_hash_map()
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
            query_use_case,
        );

        let result = loader.load(&expected_keys).await.unwrap();

        assert_eq!(result[&expected_keys[0]].len(), 1);
        assert_eq!(result[&expected_keys[0]][0].title, "Book 1");
        assert!(result[&expected_keys[1]].is_empty());
    }
}
