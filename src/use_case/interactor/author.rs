use async_trait::async_trait;

use crate::{
    domain::{
        entity::{author::AuthorId, user::UserId},
        repository::author_repository::AuthorRepository,
    },
    use_case::{dto::author::Author, error::UseCaseError, use_case::author::ShowAuthorUseCase},
};

pub struct ShowAuthorInteractor<ARepo> {
    author_repository: ARepo,
}

impl<ARepo> ShowAuthorInteractor<ARepo> {
    pub fn new(author_repository: ARepo) -> Self {
        ShowAuthorInteractor { author_repository }
    }
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

#[cfg(test)]
mod tests {

    use mockall::predicate::always;

    use crate::domain::repository::author_repository::tests::MockAuthorRepository;
    use crate::{
        domain::{
            self,
            entity::author::{AuthorId, AuthorName},
        },
        use_case::use_case::author::ShowAuthorUseCase,
    };

    use super::ShowAuthorInteractor;

    #[tokio::test]
    async fn find_by_id() {
        let mut author_repository = MockAuthorRepository::new();

        let user_id = "user1";
        let author_id = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author_name = "author1";

        author_repository
            .expect_find_by_id()
            .with(always(), always())
            .returning(|_, _| {
                Ok(Some(domain::entity::author::Author {
                    id: AuthorId::new(author_id).unwrap(),
                    name: AuthorName::new(author_name.to_string()).unwrap(),
                }))
            });

        let show_author_interactor = ShowAuthorInteractor::new(author_repository);

        let author = show_author_interactor
            .find_by_id(user_id, author_id)
            .await
            .unwrap();

        assert_eq!(author.id, author_id);
        assert_eq!(author.name, author_name);
    }
}
