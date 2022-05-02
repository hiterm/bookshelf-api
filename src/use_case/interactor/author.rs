use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{
            author::{Author as DomainAuthor, AuthorId, AuthorName},
            user::UserId,
        },
        repository::author_repository::AuthorRepository,
    },
    use_case::{
        dto::author::{AuthorDto, CreateAuthorDto},
        error::UseCaseError,
        use_case::author::CreateAuthorUseCase,
    },
};

pub struct CreateAuthorInteractor<AR> {
    author_repository: AR,
}

impl<AR> CreateAuthorInteractor<AR> {
    pub fn new(author_repository: AR) -> Self {
        Self { author_repository }
    }
}

#[async_trait]
impl<AR> CreateAuthorUseCase for CreateAuthorInteractor<AR>
where
    AR: AuthorRepository,
{
    async fn create(
        &self,
        user_id: &str,
        author_data: CreateAuthorDto,
    ) -> Result<AuthorDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let uuid = Uuid::new_v4();
        let author_id = AuthorId::new(uuid);
        let author_name = AuthorName::new(author_data.name)?;
        let author = DomainAuthor::new(author_id, author_name)?;
        self.author_repository.create(&user_id, &author).await?;

        Ok(author.into())
    }
}
