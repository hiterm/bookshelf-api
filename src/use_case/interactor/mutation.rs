use async_trait::async_trait;

use crate::use_case::{
    dto::{
        author::{AuthorDto, CreateAuthorDto},
        book::{BookDto, CreateBookDto},
        user::UserDto,
    },
    error::UseCaseError,
    use_case::{
        author::CreateAuthorUseCase, book::CreateBookUseCase, mutation::MutationUseCase,
        user::RegisterUserUseCase,
    },
};

pub struct MutationInteractor<RUUC, CBUC, CAUC> {
    register_user_use_case: RUUC,
    create_book_use_case: CBUC,
    create_author_use_case: CAUC,
}

impl<RUUC, CBUC, CAUC> MutationInteractor<RUUC, CBUC, CAUC> {
    pub fn new(
        register_user_use_case: RUUC,
        create_book_use_case: CBUC,
        create_author_use_case: CAUC,
    ) -> Self {
        Self {
            register_user_use_case,
            create_book_use_case,
            create_author_use_case,
        }
    }
}

#[async_trait]
impl<RUUC, CBUC, CAUC> MutationUseCase for MutationInteractor<RUUC, CBUC, CAUC>
where
    RUUC: RegisterUserUseCase,
    CBUC: CreateBookUseCase,
    CAUC: CreateAuthorUseCase,
{
    async fn register_user(&self, user_id: &str) -> Result<UserDto, UseCaseError> {
        let user = self.register_user_use_case.register_user(user_id).await?;
        Ok(user)
    }

    async fn create_book(
        &self,
        user_id: &str,
        book_data: CreateBookDto,
    ) -> Result<BookDto, UseCaseError> {
        let book = self.create_book_use_case.create(user_id, book_data).await?;
        Ok(book)
    }

    async fn create_author(
        &self,
        user_id: &str,
        author_data: CreateAuthorDto,
    ) -> Result<AuthorDto, UseCaseError> {
        let author = self
            .create_author_use_case
            .create(user_id, author_data)
            .await?;
        Ok(author)
    }
}
