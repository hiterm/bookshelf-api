use async_trait::async_trait;

use crate::{
    presentational::error::error::PresentationalError,
    use_case::{
        dto::{author::Author, book::Book},
        use_case::author::ShowAuthorUseCase,
    },
};

#[async_trait]
pub trait QueryService: Send + Sync + 'static {
    async fn find_book_by_id(&self, id: &str) -> Result<Book, PresentationalError>;
    async fn find_author_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Author, PresentationalError>;
}

pub struct QueryServiceImpl<SAUC> {
    pub show_author_use_case: SAUC,
}

#[async_trait]
impl<SAUC> QueryService for QueryServiceImpl<SAUC>
where
    SAUC: ShowAuthorUseCase,
{
    async fn find_book_by_id(&self, id: &str) -> Result<Book, PresentationalError> {
        todo!()
    }

    async fn find_author_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Author, PresentationalError> {
        Ok(self
            .show_author_use_case
            .find_by_id(user_id, author_id)
            .await?)
    }
}

#[cfg(test)]
pub mod tests {
    use async_trait::async_trait;
    use mockall::mock;

    use crate::{presentational::error::error::PresentationalError, use_case::dto::author::Author};

    use super::QueryService;

    mock! {
        pub QueryService {}

        #[async_trait]
        impl QueryService for QueryService {
            async fn find_book_by_id(&self, id: &str) -> Result<crate::use_case::dto::book::Book, PresentationalError>;
            async fn find_author_by_id(
                &self,
                user_id: &str,
                author_id: &str,
            ) -> Result<Author, PresentationalError>;
        }
    }
}
