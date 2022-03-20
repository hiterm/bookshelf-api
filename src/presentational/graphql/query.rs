use async_graphql::Object;
use uuid::Uuid;

use crate::presentational::error::error::PresentationalError;

use super::{object::Book, query_service::QueryService};

pub struct QueryRoot<T> {
    query_service: T,
}

impl<T> QueryRoot<T> {
    pub fn new(query_service: T) -> Self {
        QueryRoot { query_service }
    }
}

#[Object]
impl<T> QueryRoot<T>
where
    T: QueryService + Send + Sync,
{
    async fn book(&self, id: String) -> Result<Book, PresentationalError> {
        let book = self.query_service.find_book_by_id(&id)?;
        Ok(Book::new(book.id, book.title))
    }
}
