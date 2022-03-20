use mockall::automock;
use uuid::Uuid;

use crate::{presentational::error::error::PresentationalError, use_case::dto::book::Book};

#[automock]
pub trait QueryService {
    fn find_book_by_id(&self, id: &str) -> Result<Book, PresentationalError>;
}

struct QueryServiceImpl {}

impl QueryService for QueryServiceImpl {
    fn find_book_by_id(&self, id: &str) -> Result<Book, PresentationalError> {
        todo!()
    }
}
