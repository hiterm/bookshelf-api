use mockall::automock;

use crate::{presentational::error::error::PresentationalError, use_case::dto::book::Book};

#[automock]
pub trait QueryService {
    fn find_book_by_id(&self, id: &str) -> Result<Book, PresentationalError>;
}

pub struct QueryServiceImpl {}

impl QueryServiceImpl {
    pub fn new() -> Self {
        QueryServiceImpl {}
    }
}

impl QueryService for QueryServiceImpl {
    fn find_book_by_id(&self, id: &str) -> Result<Book, PresentationalError> {
        todo!()
    }
}
