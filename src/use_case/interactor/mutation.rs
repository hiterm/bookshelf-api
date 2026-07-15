use async_trait::async_trait;

use crate::use_case::{
    dto::{
        author::{CreateAuthorDto, UpdateAuthorDto},
        book::{CreateBookDto, ImportBookEntryDto, UpdateBookDto},
        mutation::{
            AuthorMutationResultDto, BookMutationResultDto, DeleteAuthorResultDto,
            DeleteBookResultDto, ImportBooksResultDto, RestoreAuthorResultDto,
            RestoreBookResultDto,
        },
        user::UserDto,
    },
    error::UseCaseError,
    traits::{
        author::{CreateAuthorUseCase, DeleteAuthorUseCase, UpdateAuthorUseCase},
        book::{CreateBookUseCase, DeleteBookUseCase, ImportBooksUseCase, UpdateBookUseCase},
        event::{RestoreAuthorUseCase, RestoreBookUseCase},
        mutation::MutationUseCase,
        user::RegisterUserUseCase,
    },
};

pub struct MutationInteractor<RUUC, CBUC, UBUC, DBUC, CAUC, UAUC, DAUC, RBUC, RAUC, IBUC> {
    register_user_use_case: RUUC,
    create_book_use_case: CBUC,
    update_book_use_case: UBUC,
    delete_book_use_case: DBUC,
    create_author_use_case: CAUC,
    update_author_use_case: UAUC,
    delete_author_use_case: DAUC,
    restore_book_use_case: RBUC,
    restore_author_use_case: RAUC,
    import_books_use_case: IBUC,
}

impl<RUUC, CBUC, UBUC, DBUC, CAUC, UAUC, DAUC, RBUC, RAUC, IBUC>
    MutationInteractor<RUUC, CBUC, UBUC, DBUC, CAUC, UAUC, DAUC, RBUC, RAUC, IBUC>
{
    // This constructor takes many arguments because MutationInteractor composes all
    // mutation use cases via dependency injection. Splitting it would reduce clarity
    // without reducing actual coupling.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        register_user_use_case: RUUC,
        create_book_use_case: CBUC,
        update_book_use_case: UBUC,
        delete_book_use_case: DBUC,
        create_author_use_case: CAUC,
        update_author_use_case: UAUC,
        delete_author_use_case: DAUC,
        restore_book_use_case: RBUC,
        restore_author_use_case: RAUC,
        import_books_use_case: IBUC,
    ) -> Self {
        Self {
            register_user_use_case,
            create_book_use_case,
            update_book_use_case,
            delete_book_use_case,
            create_author_use_case,
            update_author_use_case,
            delete_author_use_case,
            restore_book_use_case,
            restore_author_use_case,
            import_books_use_case,
        }
    }
}

#[async_trait]
impl<RUUC, CBUC, UBUC, DBUC, CAUC, UAUC, DAUC, RBUC, RAUC, IBUC> MutationUseCase
    for MutationInteractor<RUUC, CBUC, UBUC, DBUC, CAUC, UAUC, DAUC, RBUC, RAUC, IBUC>
where
    RUUC: RegisterUserUseCase,
    CBUC: CreateBookUseCase,
    UBUC: UpdateBookUseCase,
    DBUC: DeleteBookUseCase,
    CAUC: CreateAuthorUseCase,
    UAUC: UpdateAuthorUseCase,
    DAUC: DeleteAuthorUseCase,
    RBUC: RestoreBookUseCase,
    RAUC: RestoreAuthorUseCase,
    IBUC: ImportBooksUseCase,
{
    async fn register_user(&self, user_id: &str) -> Result<UserDto, UseCaseError> {
        let user = self.register_user_use_case.register_user(user_id).await?;
        Ok(user)
    }

    async fn create_book(
        &self,
        user_id: &str,
        book_data: CreateBookDto,
    ) -> Result<BookMutationResultDto, UseCaseError> {
        let book = self.create_book_use_case.create(user_id, book_data).await?;
        Ok(book)
    }

    async fn update_book(
        &self,
        user_id: &str,
        book_data: UpdateBookDto,
    ) -> Result<BookMutationResultDto, UseCaseError> {
        let book = self.update_book_use_case.update(user_id, book_data).await?;
        Ok(book)
    }

    async fn delete_book(
        &self,
        user_id: &str,
        book_id: &str,
    ) -> Result<DeleteBookResultDto, UseCaseError> {
        let result = self.delete_book_use_case.delete(user_id, book_id).await?;
        Ok(result)
    }

    async fn create_author(
        &self,
        user_id: &str,
        author_data: CreateAuthorDto,
    ) -> Result<AuthorMutationResultDto, UseCaseError> {
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
    ) -> Result<AuthorMutationResultDto, UseCaseError> {
        let author = self
            .update_author_use_case
            .update(user_id, author_data)
            .await?;
        Ok(author)
    }

    async fn delete_author(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<DeleteAuthorResultDto, UseCaseError> {
        let result = self
            .delete_author_use_case
            .delete(user_id, author_id)
            .await?;
        Ok(result)
    }

    async fn restore_book(
        &self,
        user_id: &str,
        event_id: i64,
    ) -> Result<RestoreBookResultDto, UseCaseError> {
        self.restore_book_use_case.restore(user_id, event_id).await
    }

    async fn restore_author(
        &self,
        user_id: &str,
        event_id: i64,
    ) -> Result<RestoreAuthorResultDto, UseCaseError> {
        self.restore_author_use_case
            .restore(user_id, event_id)
            .await
    }

    async fn import_books(
        &self,
        user_id: &str,
        books: Vec<ImportBookEntryDto>,
    ) -> Result<ImportBooksResultDto, UseCaseError> {
        self.import_books_use_case.import(user_id, books).await
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::{always, eq};

    use crate::common::types::{BookFormat, BookStore};
    use crate::use_case::dto::mutation::MutationResultDto;
    use crate::use_case::error::UseCaseError;
    use crate::use_case::{
        dto::{
            author::{AuthorDto, CreateAuthorDto, UpdateAuthorDto},
            book::{BookDto, CreateBookDto, ImportBookEntryDto, UpdateBookDto},
            user::UserDto,
        },
        interactor::mutation::MutationInteractor,
        traits::{
            author::{MockCreateAuthorUseCase, MockDeleteAuthorUseCase, MockUpdateAuthorUseCase},
            book::{
                MockCreateBookUseCase, MockDeleteBookUseCase, MockImportBooksUseCase,
                MockUpdateBookUseCase,
            },
            event::{MockRestoreAuthorUseCase, MockRestoreBookUseCase},
            mutation::MutationUseCase,
            user::MockRegisterUserUseCase,
        },
    };
    use time::OffsetDateTime;
    use uuid::Uuid;

    type DefaultInteractor = MutationInteractor<
        MockRegisterUserUseCase,
        MockCreateBookUseCase,
        MockUpdateBookUseCase,
        MockDeleteBookUseCase,
        MockCreateAuthorUseCase,
        MockUpdateAuthorUseCase,
        MockDeleteAuthorUseCase,
        MockRestoreBookUseCase,
        MockRestoreAuthorUseCase,
        MockImportBooksUseCase,
    >;

    struct InteractorBuilder {
        register_user: MockRegisterUserUseCase,
        create_book: MockCreateBookUseCase,
        update_book: MockUpdateBookUseCase,
        delete_book: MockDeleteBookUseCase,
        create_author: MockCreateAuthorUseCase,
        update_author: MockUpdateAuthorUseCase,
        delete_author: MockDeleteAuthorUseCase,
        restore_book: MockRestoreBookUseCase,
        restore_author: MockRestoreAuthorUseCase,
        import_books: MockImportBooksUseCase,
    }

    impl InteractorBuilder {
        fn new() -> Self {
            Self {
                register_user: MockRegisterUserUseCase::new(),
                create_book: MockCreateBookUseCase::new(),
                update_book: MockUpdateBookUseCase::new(),
                delete_book: MockDeleteBookUseCase::new(),
                create_author: MockCreateAuthorUseCase::new(),
                update_author: MockUpdateAuthorUseCase::new(),
                delete_author: MockDeleteAuthorUseCase::new(),
                restore_book: MockRestoreBookUseCase::new(),
                restore_author: MockRestoreAuthorUseCase::new(),
                import_books: MockImportBooksUseCase::new(),
            }
        }

        fn with_register_user(mut self, mock: MockRegisterUserUseCase) -> Self {
            self.register_user = mock;
            self
        }

        fn with_create_book(mut self, mock: MockCreateBookUseCase) -> Self {
            self.create_book = mock;
            self
        }

        fn with_update_book(mut self, mock: MockUpdateBookUseCase) -> Self {
            self.update_book = mock;
            self
        }

        fn with_delete_book(mut self, mock: MockDeleteBookUseCase) -> Self {
            self.delete_book = mock;
            self
        }

        fn with_create_author(mut self, mock: MockCreateAuthorUseCase) -> Self {
            self.create_author = mock;
            self
        }

        fn with_update_author(mut self, mock: MockUpdateAuthorUseCase) -> Self {
            self.update_author = mock;
            self
        }

        fn with_delete_author(mut self, mock: MockDeleteAuthorUseCase) -> Self {
            self.delete_author = mock;
            self
        }

        fn with_restore_book(mut self, mock: MockRestoreBookUseCase) -> Self {
            self.restore_book = mock;
            self
        }

        fn with_restore_author(mut self, mock: MockRestoreAuthorUseCase) -> Self {
            self.restore_author = mock;
            self
        }

        fn with_import_books(mut self, mock: MockImportBooksUseCase) -> Self {
            self.import_books = mock;
            self
        }

        fn build(self) -> DefaultInteractor {
            MutationInteractor::new(
                self.register_user,
                self.create_book,
                self.update_book,
                self.delete_book,
                self.create_author,
                self.update_author,
                self.delete_author,
                self.restore_book,
                self.restore_author,
                self.import_books,
            )
        }
    }

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

        let interactor = InteractorBuilder::new()
            .with_register_user(mock_register_user)
            .build();

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
            .returning(move |_, _| {
                Ok(MutationResultDto::new(
                    make_book_dto(&book_id),
                    "event-set".to_string(),
                ))
            });

        let interactor = InteractorBuilder::new()
            .with_create_book(mock_create_book)
            .build();

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
            .returning(move |_, _| {
                Ok(MutationResultDto::new(
                    make_book_dto(&book_id),
                    "event-set".to_string(),
                ))
            });

        let interactor = InteractorBuilder::new()
            .with_update_book(mock_update_book)
            .build();

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
            .returning(|_, _| {
                Ok(MutationResultDto::new(
                    "deleted-id".to_string(),
                    "event-set".to_string(),
                ))
            });

        let interactor = InteractorBuilder::new()
            .with_delete_book(mock_delete_book)
            .build();

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
                Ok(MutationResultDto::new(
                    AuthorDto {
                        id: "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
                        name: data.name.clone(),
                    },
                    "event-set".to_string(),
                ))
            });

        let interactor = InteractorBuilder::new()
            .with_create_author(mock_create_author)
            .build();

        let author_data = CreateAuthorDto::new("New Author".to_string());

        // When
        let result = interactor.create_author("user1", author_data).await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "New Author");
    }

    #[tokio::test]
    async fn update_author_delegates_to_sub_use_case() {
        // Given
        let mut mock_update_author = MockUpdateAuthorUseCase::new();
        mock_update_author
            .expect_update()
            .with(always(), always())
            .returning(|_, data| {
                Ok(MutationResultDto::new(
                    AuthorDto {
                        id: "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
                        name: data.name.clone(),
                    },
                    "event-set".to_string(),
                ))
            });

        let interactor = InteractorBuilder::new()
            .with_update_author(mock_update_author)
            .build();

        let author_data = UpdateAuthorDto::new(
            "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
            "Updated Author".to_string(),
        );

        // When
        let result = interactor.update_author("user1", author_data).await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "Updated Author");
    }

    #[tokio::test]
    async fn delete_author_delegates_to_sub_use_case() {
        // Given
        let mut mock_delete_author = MockDeleteAuthorUseCase::new();
        mock_delete_author
            .expect_delete()
            .with(always(), always())
            .returning(|_, _| {
                Ok(MutationResultDto::new(
                    "deleted-id".to_string(),
                    "event-set".to_string(),
                ))
            });

        let interactor = InteractorBuilder::new()
            .with_delete_author(mock_delete_author)
            .build();

        // When
        let result = interactor
            .delete_author("user1", "006099b4-6c42-4ec4-8645-f6bd5b63eddc")
            .await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn restore_book_delegates_to_sub_use_case() {
        // Given
        let book_id = Uuid::new_v4().hyphenated().to_string();
        let expected_dto = make_book_dto(&book_id);
        let expected_id = expected_dto.id.clone();

        let mut mock_restore_book = MockRestoreBookUseCase::new();
        mock_restore_book
            .expect_restore()
            .with(always(), always())
            .returning(move |_, _| {
                Ok(MutationResultDto::new(
                    Some(make_book_dto(&book_id)),
                    "event-set".to_string(),
                ))
            });

        let interactor = InteractorBuilder::new()
            .with_restore_book(mock_restore_book)
            .build();

        // When
        let result = interactor.restore_book("user1", 42).await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value.unwrap().id, expected_id);
    }

    #[tokio::test]
    async fn restore_book_delete_event_returns_none() {
        // Given
        let mut mock_restore_book = MockRestoreBookUseCase::new();
        mock_restore_book
            .expect_restore()
            .with(always(), always())
            .returning(|_, _| Ok(MutationResultDto::new(None, "event-set".to_string())));

        let interactor = InteractorBuilder::new()
            .with_restore_book(mock_restore_book)
            .build();

        // When
        let result = interactor.restore_book("user1", 42).await;

        // Then
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn restore_author_delegates_to_sub_use_case() {
        // Given
        let mut mock_restore_author = MockRestoreAuthorUseCase::new();
        mock_restore_author
            .expect_restore()
            .with(always(), always())
            .returning(|_, _| {
                Ok(MutationResultDto::new(
                    Some(AuthorDto {
                        id: "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
                        name: "Test Author".to_string(),
                    }),
                    "event-set".to_string(),
                ))
            });

        let interactor = InteractorBuilder::new()
            .with_restore_author(mock_restore_author)
            .build();

        // When
        let result = interactor.restore_author("user1", 99).await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value.unwrap().name, "Test Author");
    }

    #[tokio::test]
    async fn restore_author_delete_event_returns_none() {
        // Given
        let mut mock_restore_author = MockRestoreAuthorUseCase::new();
        mock_restore_author
            .expect_restore()
            .with(always(), always())
            .returning(|_, _| Ok(MutationResultDto::new(None, "event-set".to_string())));

        let interactor = InteractorBuilder::new()
            .with_restore_author(mock_restore_author)
            .build();

        // When
        let result = interactor.restore_author("user1", 99).await;

        // Then
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn restore_book_forwards_exact_arguments() {
        // Given
        let mut mock_restore_book = MockRestoreBookUseCase::new();
        mock_restore_book
            .expect_restore()
            .with(eq("user1"), eq(42_i64))
            .returning(|_, _| Ok(MutationResultDto::new(None, "event-set".to_string())));

        let interactor = InteractorBuilder::new()
            .with_restore_book(mock_restore_book)
            .build();

        // When
        let result = interactor.restore_book("user1", 42).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn restore_book_propagates_error() {
        // Given
        let mut mock_restore_book = MockRestoreBookUseCase::new();
        mock_restore_book
            .expect_restore()
            .with(always(), always())
            .returning(|_, _| {
                Err(UseCaseError::NotFound {
                    entity_type: "Book",
                    entity_id: "999".to_string(),
                    user_id: "user1".to_string(),
                })
            });

        let interactor = InteractorBuilder::new()
            .with_restore_book(mock_restore_book)
            .build();

        // When
        let result = interactor.restore_book("user1", 999).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn restore_author_forwards_exact_arguments() {
        // Given
        let mut mock_restore_author = MockRestoreAuthorUseCase::new();
        mock_restore_author
            .expect_restore()
            .with(eq("user1"), eq(99_i64))
            .returning(|_, _| Ok(MutationResultDto::new(None, "event-set".to_string())));

        let interactor = InteractorBuilder::new()
            .with_restore_author(mock_restore_author)
            .build();

        // When
        let result = interactor.restore_author("user1", 99).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn restore_author_propagates_error() {
        // Given
        let mut mock_restore_author = MockRestoreAuthorUseCase::new();
        mock_restore_author
            .expect_restore()
            .with(always(), always())
            .returning(|_, _| {
                Err(UseCaseError::NotFound {
                    entity_type: "Author",
                    entity_id: "999".to_string(),
                    user_id: "user1".to_string(),
                })
            });

        let interactor = InteractorBuilder::new()
            .with_restore_author(mock_restore_author)
            .build();

        // When
        let result = interactor.restore_author("user1", 999).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn import_books_delegates_to_sub_use_case() {
        // Given
        let book_id = Uuid::new_v4().hyphenated().to_string();
        let expected_dto = make_book_dto(&book_id);

        let mut mock_import_books = MockImportBooksUseCase::new();
        mock_import_books
            .expect_import()
            .with(eq("user1"), always())
            .returning(move |_, _| {
                Ok(MutationResultDto::new(
                    vec![make_book_dto(&book_id)],
                    "event-set".to_string(),
                ))
            });

        let interactor = InteractorBuilder::new()
            .with_import_books(mock_import_books)
            .build();

        let books = vec![ImportBookEntryDto {
            title: "Imported Book".to_string(),
            author_names: vec!["Author".to_string()],
            isbn: "".to_string(),
            read: false,
            owned: false,
            priority: 50,
            format: BookFormat::Unknown,
            store: BookStore::Unknown,
        }];

        // When
        let result = interactor.import_books("user1", books).await;

        // Then
        assert!(result.is_ok());
        assert_eq!(result.unwrap()[0].id, expected_dto.id);
    }
}
