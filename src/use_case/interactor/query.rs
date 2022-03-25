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

#[cfg(test)]
mod tests {

    use mockall::predicate::always;

    use crate::{
        domain::{
            self,
            entity::author::{AuthorId, AuthorName},
            repository::{
                author_repository::MockAuthorRepository, user_repository::MockUserRepository,
            },
        },
        use_case::{interactor::query::QueryInteractor, use_case::query::QueryUseCase},
    };

    #[tokio::test]
    async fn find_author_by_id() {
        let mut author_repository = MockAuthorRepository::new();
        let user_repository = MockUserRepository::new();

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

        let query_interactor = QueryInteractor {
            user_repository,
            author_repository,
        };

        let author = query_interactor
            .find_author_by_id(user_id, author_id)
            .await
            .unwrap();

        assert_eq!(author.id, author_id);
        assert_eq!(author.name, author_name);
    }
}
