use mockall::automock;

use crate::{
    presentational::error::error::PresentationalError,
    use_case::{
        dto::{author::Author, book::Book},
        use_case::author::ShowAuthorUseCase,
    },
};

#[automock]
pub trait QueryService: Send + Sync + 'static {
    fn find_book_by_id(&self, id: &str) -> Result<Book, PresentationalError>;
    fn find_author_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Author, PresentationalError>;
}

pub struct QueryServiceImpl<SAUC> {
    pub show_author_use_case: SAUC,
}

impl<SAUC> QueryService for QueryServiceImpl<SAUC>
where
    SAUC: ShowAuthorUseCase,
{
    fn find_book_by_id(&self, id: &str) -> Result<Book, PresentationalError> {
        todo!()
    }

    fn find_author_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Author, PresentationalError> {
        todo!()
    }
}
