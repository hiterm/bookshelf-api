use async_trait::async_trait;

use crate::{
    domain::{
        entity::{author::AuthorId, user::UserId},
        repository::{author_repository::AuthorRepository, user_repository::UserRepository},
    },
    use_case::{
        dto::{author::Author, user::User},
        error::UseCaseError,
        use_case::query::QueryUseCase,
    },
};

pub struct QueryInteractor<UR, AR> {
    user_repository: UR,
    author_repository: AR,
}

#[async_trait]
impl<UR, AR> QueryUseCase for QueryInteractor<UR, AR>
where
    UR: UserRepository,
    AR: AuthorRepository,
{
    async fn find_user_by_id(&self, raw_user_id: &str) -> Result<User, UseCaseError> {
        let user_id = UserId::new(raw_user_id.to_string())?;
        let user = self.user_repository.find_by_id(&user_id).await?;

        user.ok_or(UseCaseError::NotFound {
            entity_type: "user",
            entity_id: raw_user_id.to_string(),
            user_id: raw_user_id.to_string(),
        })
        .map(|user| User::new(user.id.id))
    }

    async fn find_author_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Author, UseCaseError> {
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
                user_id: raw_user_id.to_string(),
            })
            .map(|author| -> Author { author.into() })
    }
}
