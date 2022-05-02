use async_trait::async_trait;

use crate::use_case::{
    dto::{
        author::{AuthorDto, CreateAuthorDto},
        user::UserDto,
    },
    error::UseCaseError,
    use_case::{author::CreateAuthorUseCase, mutation::MutationUseCase, user::RegisterUserUseCase},
};

pub struct MutationInteractor<RUUC, CAUC> {
    register_user_use_case: RUUC,
    create_author_use_case: CAUC,
}

impl<RUUC, CAUC> MutationInteractor<RUUC, CAUC> {
    pub fn new(register_user_use_case: RUUC, create_author_use_case: CAUC) -> Self {
        Self {
            register_user_use_case,
            create_author_use_case,
        }
    }
}

#[async_trait]
impl<RUUC, CAUC> MutationUseCase for MutationInteractor<RUUC, CAUC>
where
    RUUC: RegisterUserUseCase,
    CAUC: CreateAuthorUseCase,
{
    async fn register_user(&self, user_id: &str) -> Result<UserDto, UseCaseError> {
        let user = self.register_user_use_case.register_user(user_id).await?;
        Ok(user)
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
