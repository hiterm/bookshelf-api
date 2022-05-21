use async_trait::async_trait;

use crate::use_case::{
    dto::{
        author::{AuthorDto, CreateAuthorDto},
        book::{BookDto, CreateBookDto, UpdateBookDto},
        user::UserDto,
    },
    error::UseCaseError,
    use_case::{
        author::CreateAuthorUseCase,
        book::{CreateBookUseCase, UpdateBookUseCase},
        mutation::MutationUseCase,
        user::RegisterUserUseCase,
    },
};

pub struct MutationInteractor<RUUC, CBUC, UBUC, CAUC> {
    register_user_use_case: RUUC,
    create_book_use_case: CBUC,
    update_book_use_case: UBUC,
    create_author_use_case: CAUC,
}

impl<RUUC, CBUC, UBUC, CAUC> MutationInteractor<RUUC, CBUC, UBUC, CAUC> {
    pub fn new(
        register_user_use_case: RUUC,
        create_book_use_case: CBUC,
        update_book_use_case: UBUC,
        create_author_use_case: CAUC,
    ) -> Self {
        Self {
            register_user_use_case,
            create_book_use_case,
            update_book_use_case,
            create_author_use_case,
        }
    }
}

#[async_trait]
impl<RUUC, CBUC, UBUC, CAUC> MutationUseCase for MutationInteractor<RUUC, CBUC, UBUC, CAUC>
where
    RUUC: RegisterUserUseCase,
    CBUC: CreateBookUseCase,
    UBUC: UpdateBookUseCase,
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

    async fn update_book(
        &self,
        user_id: &str,
        book_data: UpdateBookDto,
    ) -> Result<BookDto, UseCaseError> {
        let book = self.update_book_use_case.update(user_id, book_data).await?;
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
