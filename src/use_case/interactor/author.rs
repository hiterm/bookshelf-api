use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{author::AuthorId, user::UserId},
        repository::author_repository::AuthorRepository,
    },
    use_case::{dto::author::Author, error::UseCaseError, use_case::author::ShowAuthorUseCase},
};

struct ShowAuthorInteractor<ARepo> {
    author_repository: ARepo,
}

#[async_trait]
impl<ARepo> ShowAuthorUseCase for ShowAuthorInteractor<ARepo>
where
    ARepo: AuthorRepository,
{
    async fn find_by_id(&self, user_id: &str, author_id: &str) -> Result<Author, UseCaseError> {
        let raw_user_id = user_id;
        let raw_author_id = author_id;
        let user_id = UserId::new(raw_user_id.to_string())?;
        let author_id = AuthorId::new(raw_author_id)?;
        let author = self
            .author_repository
            .find_by_id(&user_id, &author_id)
            .await?;

        author
            .ok_or(UseCaseError::NotFound {
                entity_type: "author",
                entity_id: raw_author_id.to_string(),
                user_id: raw_author_id.to_string(),
            })
            .map(|author| -> Author { author.into() })
    }
}
