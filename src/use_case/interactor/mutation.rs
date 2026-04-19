use async_trait::async_trait;

use crate::use_case::{
    dto::{
        author::{AuthorDto, CreateAuthorDto, UpdateAuthorDto},
        book::{BookDto, CreateBookDto, UpdateBookDto},
        user::UserDto,
    },
    error::UseCaseError,
    traits::{
        author::{CreateAuthorUseCase, DeleteAuthorUseCase, UpdateAuthorUseCase},
        book::{CreateBookUseCase, DeleteBookUseCase, UpdateBookUseCase},
        mutation::MutationUseCase,
        user::RegisterUserUseCase,
    },
};

pub struct MutationInteractor<RUUC, CBUC, UBUC, DBUC, CAUC, UAUC, DAUC> {
    register_user_use_case: RUUC,
    create_book_use_case: CBUC,
    update_book_use_case: UBUC,
    delete_book_use_case: DBUC,
    create_author_use_case: CAUC,
    update_author_use_case: UAUC,
    delete_author_use_case: DAUC,
}

impl<RUUC, CBUC, UBUC, DBUC, CAUC, UAUC, DAUC>
    MutationInteractor<RUUC, CBUC, UBUC, DBUC, CAUC, UAUC, DAUC>
{
    pub fn new(
        register_user_use_case: RUUC,
        create_book_use_case: CBUC,
        update_book_use_case: UBUC,
        delete_book_use_case: DBUC,
        create_author_use_case: CAUC,
        update_author_use_case: UAUC,
        delete_author_use_case: DAUC,
    ) -> Self {
        Self {
            register_user_use_case,
            create_book_use_case,
            update_book_use_case,
            delete_book_use_case,
            create_author_use_case,
            update_author_use_case,
            delete_author_use_case,
        }
    }
}

#[async_trait]
impl<RUUC, CBUC, UBUC, DBUC, CAUC, UAUC, DAUC> MutationUseCase
    for MutationInteractor<RUUC, CBUC, UBUC, DBUC, CAUC, UAUC, DAUC>
where
    RUUC: RegisterUserUseCase,
    CBUC: CreateBookUseCase,
    UBUC: UpdateBookUseCase,
    DBUC: DeleteBookUseCase,
    CAUC: CreateAuthorUseCase,
    UAUC: UpdateAuthorUseCase,
    DAUC: DeleteAuthorUseCase,
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

    async fn delete_book(&self, user_id: &str, book_id: &str) -> Result<(), UseCaseError> {
        self.delete_book_use_case.delete(user_id, book_id).await?;
        Ok(())
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

    async fn update_author(
        &self,
        user_id: &str,
        author_data: UpdateAuthorDto,
    ) -> Result<AuthorDto, UseCaseError> {
        let author = self
            .update_author_use_case
            .update(user_id, author_data)
            .await?;
        Ok(author)
    }

    async fn delete_author(&self, user_id: &str, author_id: &str) -> Result<(), UseCaseError> {
        self.delete_author_use_case.delete(user_id, author_id).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::always;

    use crate::common::types::{BookFormat, BookStore};
    use crate::use_case::{
        dto::{
            author::{AuthorDto, CreateAuthorDto},
            book::{BookDto, CreateBookDto, UpdateBookDto},
            user::UserDto,
        },
        interactor::mutation::MutationInteractor,
        traits::{
            author::{MockCreateAuthorUseCase, MockDeleteAuthorUseCase, MockUpdateAuthorUseCase},
            book::{MockCreateBookUseCase, MockDeleteBookUseCase, MockUpdateBookUseCase},
            mutation::MutationUseCase,
            user::MockRegisterUserUseCase,
        },
    };
    use time::OffsetDateTime;
    use uuid::Uuid;

    fn make_book_dto(id: &str) -> BookDto {
        BookDto {
            id: id.to_string(),
            title: "Test Book".to_string(),
            author_ids: vec![],
            isbn: "".to_string(),
            read: false,
            owned: false,
            priority: 0,
            format: BookFormat::Unknown,
            store: BookStore::Unknown,
            created_at: OffsetDateTime::now_utc(),
            updated_at: OffsetDateTime::now_utc(),
        }
    }

    #[tokio::test]
    async fn register_user_delegates_to_sub_use_case() {
        // Given
        let mut mock_register_user = MockRegisterUserUseCase::new();
        mock_register_user
            .expect_register_user()
            .with(always())
            .returning(|id| Ok(UserDto::new(id.to_string())));

        let interactor = MutationInteractor::new(
            mock_register_user,
            MockCreateBookUseCase::new(),
            MockUpdateBookUseCase::new(),
            MockDeleteBookUseCase::new(),
            MockCreateAuthorUseCase::new(),
            MockUpdateAuthorUseCase::new(),
            MockDeleteAuthorUseCase::new(),
        );

        // When
        let result = interactor.register_user("user1").await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, "user1");
    }

    #[tokio::test]
    async fn create_book_delegates_to_sub_use_case() {
        // Given
        let book_id = Uuid::new_v4().hyphenated().to_string();
        let expected_dto = make_book_dto(&book_id);

        let mut mock_create_book = MockCreateBookUseCase::new();
        mock_create_book
            .expect_create()
            .with(always(), always())
            .returning(move |_, _| Ok(make_book_dto(&book_id)));

        let interactor = MutationInteractor::new(
            MockRegisterUserUseCase::new(),
            mock_create_book,
            MockUpdateBookUseCase::new(),
            MockDeleteBookUseCase::new(),
            MockCreateAuthorUseCase::new(),
            MockUpdateAuthorUseCase::new(),
            MockDeleteAuthorUseCase::new(),
        );

        let book_data = CreateBookDto::new(
            "New Book".to_string(),
            vec![],
            "".to_string(),
            false,
            false,
            0,
            BookFormat::Unknown,
            BookStore::Unknown,
        );

        // When
        let result = interactor.create_book("user1", book_data).await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, expected_dto.id);
    }

    #[tokio::test]
    async fn update_book_delegates_to_sub_use_case() {
        // Given
        let book_id = Uuid::new_v4().hyphenated().to_string();
        let expected_dto = make_book_dto(&book_id);

        let mut mock_update_book = MockUpdateBookUseCase::new();
        mock_update_book
            .expect_update()
            .with(always(), always())
            .returning(move |_, _| Ok(make_book_dto(&book_id)));

        let interactor = MutationInteractor::new(
            MockRegisterUserUseCase::new(),
            MockCreateBookUseCase::new(),
            mock_update_book,
            MockDeleteBookUseCase::new(),
            MockCreateAuthorUseCase::new(),
            MockUpdateAuthorUseCase::new(),
            MockDeleteAuthorUseCase::new(),
        );

        let book_data = UpdateBookDto::new(
            expected_dto.id.clone(),
            "Updated Book".to_string(),
            vec![],
            "".to_string(),
            false,
            false,
            0,
            BookFormat::Unknown,
            BookStore::Unknown,
        );

        // When
        let result = interactor.update_book("user1", book_data).await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, expected_dto.id);
    }

    #[tokio::test]
    async fn delete_book_delegates_to_sub_use_case() {
        // Given
        let mut mock_delete_book = MockDeleteBookUseCase::new();
        mock_delete_book
            .expect_delete()
            .with(always(), always())
            .returning(|_, _| Ok(()));

        let interactor = MutationInteractor::new(
            MockRegisterUserUseCase::new(),
            MockCreateBookUseCase::new(),
            MockUpdateBookUseCase::new(),
            mock_delete_book,
            MockCreateAuthorUseCase::new(),
            MockUpdateAuthorUseCase::new(),
            MockDeleteAuthorUseCase::new(),
        );

        // When
        let result = interactor
            .delete_book("user1", "a1b2c3d4-e5f6-4890-abcd-ef1234567890")
            .await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn create_author_delegates_to_sub_use_case() {
        // Given
        let mut mock_create_author = MockCreateAuthorUseCase::new();
        mock_create_author
            .expect_create()
            .with(always(), always())
            .returning(|_, data| {
                Ok(AuthorDto {
                    id: "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
                    name: data.name.clone(),
                })
            });

        let interactor = MutationInteractor::new(
            MockRegisterUserUseCase::new(),
            MockCreateBookUseCase::new(),
            MockUpdateBookUseCase::new(),
            MockDeleteBookUseCase::new(),
            mock_create_author,
            MockUpdateAuthorUseCase::new(),
            MockDeleteAuthorUseCase::new(),
        );

        let author_data = CreateAuthorDto::new("New Author".to_string());

        // When
        let result = interactor.create_author("user1", author_data).await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "New Author");
    }
}
