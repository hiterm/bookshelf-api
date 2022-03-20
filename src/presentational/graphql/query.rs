use async_graphql::Object;
use uuid::Uuid;

use crate::presentational::error::error::PresentationalError;

use super::{object::Book, query_service::QueryService};

pub struct QueryRoot<T> {
    query_service: T,
}

#[Object]
impl<T> QueryRoot<T>
where
    T: QueryService + Send + Sync,
{
    async fn book(&self, id: String) -> Result<Book, PresentationalError> {
        let id = Uuid::parse_str(&id)?;
        self.query_service.find_book_by_id(id)
    }
}

