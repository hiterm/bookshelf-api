use std::collections::HashMap;

use async_trait::async_trait;

use crate::{
    domain::{
        entity::{author::AuthorId, user::UserId},
        error::DomainError,
        repository::{
            author_repository::AuthorRepository, book_repository::BookRepository,
            user_repository::UserRepository,
        },
    },
    use_case::{
        dto::{author::AuthorDto, book::BookDto, user::UserDto},
        error::UseCaseError,
        use_case::query::QueryUseCase,
    },
};

pub struct QueryInteractor<UR, BR, AR> {
    pub user_repository: UR,
    pub book_repository: BR,
    pub author_repository: AR,
}

#[async_trait]
impl<UR, BR, AR> QueryUseCase for QueryInteractor<UR, BR, AR>
where
    UR: UserRepository,
    BR: BookRepository,
    AR: AuthorRepository,
{
    async fn find_user_by_id(&self, raw_user_id: &str) -> Result<Option<UserDto>, UseCaseError> {
        let user_id = UserId::new(raw_user_id.to_string())?;
        let user = self.user_repository.find_by_id(&user_id).await?;

        Ok(user.map(|user| UserDto::new(user.id.into_string())))
    }

    async fn find_all_books(&self, user_id: &str) -> Result<Vec<BookDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let books = self.book_repository.find_all(&user_id).await?;
        let books: Vec<BookDto> = books.into_iter().map(|book| BookDto::from(book)).collect();
        Ok(books)
    }

    async fn find_author_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Option<AuthorDto>, UseCaseError> {
        let raw_user_id = user_id;
        let raw_author_id = author_id;
        let user_id = UserId::new(raw_user_id.to_string())?;
        let author_id = AuthorId::try_from(raw_author_id)?;
        let author = self
            .author_repository
            .find_by_id(&user_id, &author_id)
            .await?;

        Ok(author.map(|author| AuthorDto::from(author)))
    }

    async fn find_all_authors(&self, user_id: &str) -> Result<Vec<AuthorDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let authors = self.author_repository.find_all(&user_id).await?;
        let authors: Vec<AuthorDto> = authors
            .into_iter()
            .map(|author| AuthorDto::from(author))
            .collect();
        Ok(authors)
    }

    async fn find_author_by_ids_as_hash_map(
        &self,
        user_id: &str,
        author_ids: &[String],
    ) -> Result<HashMap<String, AuthorDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_ids: Vec<AuthorId> = author_ids
            .iter()
            .map(|author_id| AuthorId::try_from(author_id.as_str()))
            .collect::<Result<Vec<AuthorId>, DomainError>>()?;
        let authors_map = self
            .author_repository
            .find_by_ids_as_hash_map(&user_id, &author_ids)
            .await?;
        let authors_map = authors_map
            .into_iter()
            .map(|(author_id, author)| (author_id.to_string(), author.into()))
            .collect();

        Ok(authors_map)
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
                author_repository::MockAuthorRepository, book_repository::MockBookRepository,
                user_repository::MockUserRepository,
            },
        },
        use_case::{
            dto::author::AuthorDto, interactor::query::QueryInteractor,
            use_case::query::QueryUseCase,
        },
    };

    #[tokio::test]
    async fn find_author_by_id() {
        let user_repository = MockUserRepository::new();
        let book_repository = MockBookRepository::new();
        let mut author_repository = MockAuthorRepository::new();

        let user_id = "user1";
        let author_id = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author_name = "author1";

        author_repository
            .expect_find_by_id()
            .with(always(), always())
            .returning(move |_, _| {
                Ok(Some(domain::entity::author::Author::new(
                    AuthorId::try_from(author_id).unwrap(),
                    AuthorName::new(author_name.to_string()).unwrap(),
                )?))
            });

        let query_interactor = QueryInteractor {
            user_repository,
            book_repository,
            author_repository,
        };

        let actual = query_interactor
            .find_author_by_id(user_id, author_id)
            .await
            .unwrap();

        let expected = Some(AuthorDto {
            id: author_id.to_owned(),
            name: author_name.to_owned(),
        });

        assert_eq!(actual, expected);
    }
}
