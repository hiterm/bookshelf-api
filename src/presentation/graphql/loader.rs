use std::collections::HashMap;

use async_graphql::dataloader::Loader;
use async_trait::async_trait;

use crate::{
    extractors::Claims, presentation::error::PresentationalError,
    use_case::traits::query::QueryUseCase,
};

use super::object::Author;

pub struct AuthorLoader<QUC> {
    claims: Claims,
    query_use_case: QUC,
}

impl<QUC> AuthorLoader<QUC> {
    pub fn new(claims: Claims, query_use_case: QUC) -> Self {
        Self {
            claims,
            query_use_case,
        }
    }
}

#[async_trait]
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
