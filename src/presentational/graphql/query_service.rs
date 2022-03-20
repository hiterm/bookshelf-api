use uuid::Uuid;

use crate::presentational::error::error::PresentationalError;

use super::object::Book;

pub trait QueryService {
    fn find_book_by_id(&self, id: Uuid) -> Result<Book, PresentationalError>;
}

struct QueryServiceImpl {}

impl QueryService for QueryServiceImpl {
    fn find_book_by_id(&self, id: Uuid) -> Result<Book, PresentationalError> {
        todo!()
    }
}
