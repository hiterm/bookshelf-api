use async_trait::async_trait;
use mockall::automock;

use crate::{
    presentational::error::error::PresentationalError,
    use_case::{
        dto::{author::Author, book::Book, user::User},
        use_case::{author::ShowAuthorUseCase, user::LoginUseCase},
    },
};

#[automock]
#[async_trait]
pub trait QueryService: Send + Sync + 'static {
    async fn find_user_by_id(&self, user_id: &str) -> Result<User, PresentationalError>;
    async fn find_author_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Author, PresentationalError>;

    // TODO: fix
    async fn find_book_by_id(&self, id: &str) -> Result<Book, PresentationalError>;
}

pub struct QueryServiceImpl<LUC, SAUC> {
    pub login_use_case: LUC,
    pub show_author_use_case: SAUC,
}

#[async_trait]
impl<LUC, SAUC> QueryService for QueryServiceImpl<LUC, SAUC>
where
    LUC: LoginUseCase,
    SAUC: ShowAuthorUseCase,
{
    async fn find_user_by_id(&self, user_id: &str) -> Result<User, PresentationalError> {
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

    // TODO: remove attribute
    #[allow(unused)]
    async fn find_book_by_id(&self, id: &str) -> Result<Book, PresentationalError> {
        todo!()
    }
}
