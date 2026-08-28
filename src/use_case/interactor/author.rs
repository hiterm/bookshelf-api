use std::collections::HashMap;

use async_trait::async_trait;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{
        entity::{
            author::{Author, AuthorId, AuthorName, AuthorUpdate, validate_author_yomi},
            book::BookUpdate,
            event::{EventSetOperation, NewAuthorEvent},
            operation::NewOperation,
            user::UserId,
        },
        repository::{
            author_event_repository::AuthorEventRepository,
            author_repository::{AuthorRepository, DeleteAuthorEventExtra},
            book_repository::BookRepository,
            transaction::{TransactionManager, TransactionOperation},
        },
    },
    use_case::{
        dto::{
            author::{AuthorDto, CreateAuthorDto, MergeAuthorInputDto, UpdateAuthorDto},
            mutation::{
                AuthorMutationResultDto, DeleteAuthorResultDto, MutationResultDto,
                RestoreAuthorResultDto, SingleEventMutationResultDto,
            },
        },
        error::UseCaseError,
        traits::author::{AuthorCommandUseCase, AuthorQueryUseCase},
    },
};

#[derive(Debug, Clone)]
pub struct AuthorQueryInteractor<AR> {
    author_repository: AR,
}

impl<AR> AuthorQueryInteractor<AR> {
    pub fn new(author_repository: AR) -> Self {
        Self { author_repository }
    }
}

#[async_trait]
impl<AR> AuthorQueryUseCase for AuthorQueryInteractor<AR>
where
    AR: AuthorRepository,
{
    async fn find_by_id(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<Option<AuthorDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_id)?;
        Ok(self
            .author_repository
            .find_by_id(&user_id, &author_id)
            .await?
            .map(AuthorDto::from))
    }

    async fn find_all(&self, user_id: &str) -> Result<Vec<AuthorDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        Ok(self
            .author_repository
            .find_all(&user_id)
            .await?
            .into_iter()
            .map(AuthorDto::from)
            .collect())
    }

    async fn find_by_ids(
        &self,
        user_id: &str,
        author_ids: &[String],
    ) -> Result<HashMap<String, AuthorDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_ids: Vec<AuthorId> = author_ids
            .iter()
            .map(|author_id| AuthorId::try_from(author_id.as_str()))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self
            .author_repository
            .find_by_ids_as_hash_map(&user_id, &author_ids)
            .await?
            .into_iter()
            .map(|(author_id, author)| (author_id.to_string(), author.into()))
            .collect())
    }
}

pub struct AuthorCommandInteractor<AR, BR, AER, TM> {
    author_repository: AR,
    book_repository: BR,
    author_event_repository: AER,
    transaction_manager: TM,
}

impl<AR, BR, AER, TM> AuthorCommandInteractor<AR, BR, AER, TM> {
    pub fn new(
        author_repository: AR,
        book_repository: BR,
        author_event_repository: AER,
        transaction_manager: TM,
    ) -> Self {
        Self {
            author_repository,
            book_repository,
            author_event_repository,
            transaction_manager,
        }
    }
}

#[async_trait]
impl<AR, BR, AER, TM> AuthorCommandUseCase for AuthorCommandInteractor<AR, BR, AER, TM>
where
    TM: TransactionManager,
    AR: AuthorRepository<Transaction = TM::Transaction>,
    BR: BookRepository<Transaction = TM::Transaction>,
    AER: AuthorEventRepository<Transaction = TM::Transaction>,
{
    async fn merge(
        &self,
        user_id: &str,
        input: MergeAuthorInputDto,
    ) -> Result<MutationResultDto<AuthorDto>, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let source_id_text = input.source_author_id;
        let destination_id_text = input.destination_author_id;
        let source_id = AuthorId::try_from(source_id_text.as_str())?;
        let destination_id = AuthorId::try_from(destination_id_text.as_str())?;
        if source_id == destination_id {
            return Err(UseCaseError::Validation(
                "source and destination authors must differ".to_string(),
            ));
        }

        let mut tx = self
            .transaction_manager
            .begin_operation(
                &user_id,
                &NewOperation::merge_author(&source_id, &destination_id),
            )
            .await?;

        // Book updates lock their Book row before book_author inserts acquire
        // foreign-key locks on Author rows. Merge uses the same Book -> Author
        // order to avoid a deadlock cycle with a concurrent updateBook.
        let mut books = self
            .book_repository
            .find_by_author_id_with_tx(&mut tx, &user_id, &source_id)
            .await?;

        let (first_id, second_id) = if source_id.to_uuid() < destination_id.to_uuid() {
            (&source_id, &destination_id)
        } else {
            (&destination_id, &source_id)
        };
        let first = self
            .author_repository
            .find_by_id_with_tx(&mut tx, &user_id, first_id)
            .await?;
        let second = self
            .author_repository
            .find_by_id_with_tx(&mut tx, &user_id, second_id)
            .await?;
        let find_or_not_found = |author: Option<Author>, id: &str| {
            author.ok_or_else(|| UseCaseError::NotFound {
                entity_type: "author",
                entity_id: id.to_string(),
                user_id: user_id.clone().into_string(),
            })
        };
        let (source_author, destination_author) = if first_id == &source_id {
            (
                find_or_not_found(first, &source_id_text)?,
                find_or_not_found(second, &destination_id_text)?,
            )
        } else {
            (
                find_or_not_found(second, &source_id_text)?,
                find_or_not_found(first, &destination_id_text)?,
            )
        };

        for book in &mut books {
            let mut author_ids: Vec<_> = book
                .author_ids()
                .iter()
                .filter(|id| *id != &source_id)
                .cloned()
                .collect();
            if !author_ids.contains(&destination_id) {
                author_ids.push(destination_id.clone());
            }
            book.update(
                BookUpdate {
                    title: book.title().clone(),
                    author_ids,
                    isbn: book.isbn().clone(),
                    read: book.read().clone(),
                    owned: book.owned().clone(),
                    priority: book.priority().clone(),
                    format: book.format().clone(),
                    store: book.store().clone(),
                },
                OffsetDateTime::now_utc(),
            );
        }
        self.book_repository.update_all(&mut tx, &books).await?;

        self.author_repository
            .record_unchanged_revision(&mut tx, &destination_author)
            .await?;

        self.author_repository
            .delete(
                &mut tx,
                source_author.id(),
                Some(DeleteAuthorEventExtra::Merge {
                    destination_author_id: destination_id.clone(),
                }),
            )
            .await?;
        self.author_event_repository
            .append(
                &mut tx,
                &NewAuthorEvent::merge_as_destination(destination_id, source_author.id()),
            )
            .await?;
        let operation_id = tx.operation_id().to_string();
        self.transaction_manager.commit(tx).await?;

        Ok(MutationResultDto::new(
            destination_author.into(),
            operation_id,
        ))
    }

    async fn create(
        &self,
        user_id: &str,
        author_data: CreateAuthorDto,
    ) -> Result<AuthorMutationResultDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let uuid = Uuid::new_v4();
        let author_id = AuthorId::new(uuid);
        let author_name = AuthorName::new(author_data.name)?;
        let yomi = validate_author_yomi(author_data.yomi.unwrap_or_default())?;
        let now = OffsetDateTime::now_utc();
        let author = Author::new_with_yomi(author_id, author_name, yomi, now)?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::CreateAuthor)
            .await?;
        let _event_id = self.author_repository.create(&mut tx, &author).await?;
        let operation_id = tx.operation_id().to_string();
        let revision_number = tx.revision_number().ok_or_else(|| {
            UseCaseError::Unexpected("Author mutation did not record a revision".to_string())
        })?;
        self.transaction_manager.commit(tx).await?;

        Ok(SingleEventMutationResultDto::new(
            author.into(),
            operation_id,
            revision_number,
        ))
    }

    async fn update(
        &self,
        user_id: &str,
        author_data: UpdateAuthorDto,
    ) -> Result<AuthorMutationResultDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_data.id.as_str())?;
        let author_name = AuthorName::new(author_data.name)?;
        let yomi = author_data.yomi.map(validate_author_yomi).transpose()?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::UpdateAuthor)
            .await?;
        let author = self
            .author_repository
            .find_by_id_with_tx(&mut tx, &user_id, &author_id)
            .await?;
        let mut author = match author {
            Some(author) => author,
            None => {
                return Err(UseCaseError::NotFound {
                    entity_type: "author",
                    entity_id: author_data.id,
                    user_id: user_id.into_string(),
                });
            }
        };

        author.update(
            AuthorUpdate {
                name: author_name,
                yomi,
            },
            OffsetDateTime::now_utc(),
        );

        let _event_id = self.author_repository.update(&mut tx, &author).await?;
        let operation_id = tx.operation_id().to_string();
        let revision_number = tx.revision_number().ok_or_else(|| {
            UseCaseError::Unexpected("Author mutation did not record a revision".to_string())
        })?;
        self.transaction_manager.commit(tx).await?;

        Ok(SingleEventMutationResultDto::new(
            author.into(),
            operation_id,
            revision_number,
        ))
    }

    async fn delete(
        &self,
        user_id: &str,
        author_id: &str,
    ) -> Result<DeleteAuthorResultDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id_value = author_id.to_string();
        let author_id = AuthorId::try_from(author_id)?;

        let mut tx = self
            .transaction_manager
            .begin(&user_id, EventSetOperation::DeleteAuthor)
            .await?;
        self.author_repository
            .delete(&mut tx, &author_id, None)
            .await?;
        let operation_id = tx.operation_id().to_string();
        self.transaction_manager.commit(tx).await?;

        Ok(MutationResultDto::new(author_id_value, operation_id))
    }

    async fn restore(
        &self,
        user_id: &str,
        author_id: &str,
        revision_number: i32,
    ) -> Result<RestoreAuthorResultDto, UseCaseError> {
        let user_id = UserId::new(user_id.to_string())?;
        let author_id = AuthorId::try_from(author_id)?;
        crate::domain::entity::revision::RevisionNumber::try_from(revision_number)?;
        let operation = NewOperation::restore_author(revision_number);
        let mut tx = self
            .transaction_manager
            .begin_operation(&user_id, &operation)
            .await?;
        let restored = self
            .author_repository
            .restore_revision(&mut tx, &author_id, revision_number)
            .await?;
        let operation_id = tx.operation_id().to_string();
        let restored_revision_number = tx.revision_number().ok_or_else(|| {
            UseCaseError::Unexpected("Author restore did not record a revision".to_string())
        })?;
        self.transaction_manager.commit(tx).await?;
        Ok(SingleEventMutationResultDto::new(
            Some(restored.into()),
            operation_id,
            restored_revision_number,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use mockall::{Sequence, predicate::always};
    use time::OffsetDateTime;
    use uuid::Uuid;

    use crate::{
        common::{
            time::normalize_timestamp_for_persistence,
            types::{BookFormat, BookStore},
        },
        domain::{
            entity::{
                author::{Author, AuthorId, AuthorName},
                book::{Book, BookId, BookTitle, Isbn, OwnedFlag, Priority, ReadFlag},
                event::EventOperation,
            },
            error::DomainError,
            repository::{
                author_event_repository::MockAuthorEventRepository,
                author_repository::{DeleteAuthorEventExtra, MockAuthorRepository},
                book_repository::MockBookRepository,
                transaction::MockTransactionManager,
            },
        },
        use_case::{
            dto::author::{CreateAuthorDto, UpdateAuthorDto},
            error::UseCaseError,
            interactor::author::{AuthorCommandInteractor, AuthorQueryInteractor},
            traits::author::{AuthorCommandUseCase, AuthorQueryUseCase},
        },
    };

    // A MockTransactionManager whose Transaction associated type is () and
    // whose begin/commit succeed, for interactors that reach the repository.
    fn make_transaction_manager() -> MockTransactionManager {
        let mut tm = MockTransactionManager::new();
        tm.expect_begin().returning(|_, _| Ok(()));
        tm.expect_begin_operation().returning(|_, _| Ok(()));
        tm.expect_commit().returning(|_| Ok(()));
        tm
    }

    fn command_interactor(
        author_repository: MockAuthorRepository,
        transaction_manager: MockTransactionManager,
    ) -> AuthorCommandInteractor<
        MockAuthorRepository,
        MockBookRepository,
        MockAuthorEventRepository,
        MockTransactionManager,
    > {
        AuthorCommandInteractor::new(
            author_repository,
            MockBookRepository::new(),
            MockAuthorEventRepository::new(),
            transaction_manager,
        )
    }

    #[tokio::test]
    async fn find_author_by_id_uses_repository() {
        let author_id = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let mut repository = MockAuthorRepository::new();
        repository
            .expect_find_by_id()
            .withf(|user_id, _| user_id.as_str() == "user1")
            .returning(|_, _| Ok(None));

        let result = AuthorQueryInteractor::new(repository)
            .find_by_id("user1", author_id)
            .await
            .unwrap();

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn find_all_authors_uses_repository() {
        let mut repository = MockAuthorRepository::new();
        repository
            .expect_find_all()
            .withf(|user_id| user_id.as_str() == "user1")
            .returning(|_| Ok(vec![]));

        let result = AuthorQueryInteractor::new(repository)
            .find_all("user1")
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn find_authors_by_ids_returns_hash_map() {
        let author_id = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let mut repository = MockAuthorRepository::new();
        repository
            .expect_find_by_ids_as_hash_map()
            .withf(|user_id, ids| user_id.as_str() == "user1" && ids.len() == 1)
            .returning(|_, _| Ok(HashMap::new()));

        let result = AuthorQueryInteractor::new(repository)
            .find_by_ids("user1", &[author_id.to_string()])
            .await
            .unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn restore_author_applies_revision_state() {
        let author_id = AuthorId::new(Uuid::new_v4());
        let restored_author = Author::new_with_timestamps(
            author_id.clone(),
            AuthorName::new("Old Name".to_string()).unwrap(),
            "おーるど".to_string(),
            OffsetDateTime::UNIX_EPOCH,
            OffsetDateTime::now_utc(),
        )
        .unwrap();
        let mut authors = MockAuthorRepository::new();
        let expected_author_id = author_id.clone();
        authors
            .expect_restore_revision()
            .withf(move |_, id, revision| id == &expected_author_id && *revision == 2)
            .return_once(move |_, _, _| Ok(restored_author));
        let interactor = AuthorCommandInteractor::new(
            authors,
            MockBookRepository::new(),
            MockAuthorEventRepository::new(),
            make_transaction_manager(),
        );

        let restored = interactor
            .restore("user1", &author_id.to_string(), 2)
            .await
            .unwrap();

        assert_eq!(restored.value.unwrap().name, "Old Name");
    }

    #[tokio::test]
    async fn restore_author_failure_does_not_commit() {
        let author_id = AuthorId::new(Uuid::new_v4());
        let mut authors = MockAuthorRepository::new();
        authors
            .expect_restore_revision()
            .returning(|_, _, _| Err(DomainError::Unexpected("restore failed".to_string())));
        let mut transaction_manager = MockTransactionManager::new();
        transaction_manager
            .expect_begin_operation()
            .returning(|_, _| Ok(()));
        transaction_manager.expect_commit().times(0);
        let interactor = AuthorCommandInteractor::new(
            authors,
            MockBookRepository::new(),
            MockAuthorEventRepository::new(),
            transaction_manager,
        );

        let result = interactor.restore("user1", &author_id.to_string(), 2).await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn restore_author_rejects_invalid_revision_before_transaction() {
        let author_id = AuthorId::new(Uuid::new_v4());
        let mut authors = MockAuthorRepository::new();
        authors.expect_restore_revision().times(0);
        let interactor = AuthorCommandInteractor::new(
            authors,
            MockBookRepository::new(),
            MockAuthorEventRepository::new(),
            MockTransactionManager::new(),
        );

        let result = interactor.restore("user1", &author_id.to_string(), 0).await;

        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn create_author_success() {
        // Given
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_create()
            .with(always(), always())
            .returning(|_, _| Ok(303.into()));

        let interactor = command_interactor(author_repository, make_transaction_manager());
        let mut author_data = CreateAuthorDto::new("Test Author".to_string());
        author_data.yomi = Some("てすと・おーさー1".to_string());

        // When
        let before = normalize_timestamp_for_persistence(OffsetDateTime::now_utc());
        let result = interactor.create("user1", author_data).await;
        let after = normalize_timestamp_for_persistence(OffsetDateTime::now_utc());

        // Then
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.value.name, "Test Author");
        assert_eq!(dto.value.yomi, "てすと・おーさー1");
        assert_eq!(dto.value.created_at, dto.value.updated_at);
        assert!(dto.value.created_at >= before);
        assert!(dto.value.created_at <= after);
        assert_eq!(dto.revision_number, 1);
    }

    #[tokio::test]
    async fn create_author_repository_failure_returns_no_result() {
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_create()
            .returning(|_, _| Err(DomainError::Unexpected("event insert failed".to_string())));

        let interactor = command_interactor(author_repository, {
            let mut tm = MockTransactionManager::new();
            tm.expect_begin().returning(|_, _| Ok(()));
            tm.expect_commit().times(0);
            tm
        });
        let author_data = CreateAuthorDto::new("Test Author".to_string());

        let result = interactor.create("user1", author_data).await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn create_author_commit_failure_returns_no_result() {
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_create()
            .returning(|_, _| Ok(303.into()));

        let mut tm = MockTransactionManager::new();
        tm.expect_begin().returning(|_, _| Ok(()));
        tm.expect_commit()
            .returning(|_| Err(DomainError::Unexpected("commit failed".to_string())));
        let interactor = command_interactor(author_repository, tm);
        let author_data = CreateAuthorDto::new("Test Author".to_string());

        let result = interactor.create("user1", author_data).await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn create_author_fails_with_empty_name() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor = command_interactor(author_repository, MockTransactionManager::new());
        let author_data = CreateAuthorDto::new("".to_string());

        // When
        let result = interactor.create("user1", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn create_author_fails_with_invalid_user_id() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor = command_interactor(author_repository, MockTransactionManager::new());
        let author_data = CreateAuthorDto::new("Test Author".to_string());

        // When
        let result = interactor.create("", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn create_author_fails_with_invalid_yomi() {
        let author_repository = MockAuthorRepository::new();
        let interactor = command_interactor(author_repository, MockTransactionManager::new());
        let mut author_data = CreateAuthorDto::new("Test Author".to_string());
        author_data.yomi = Some("テスト".to_string());

        let result = interactor.create("user1", author_data).await;

        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn update_author_success() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        let created_at = OffsetDateTime::from_unix_timestamp(1_600_000_000).unwrap();
        let previous_updated_at = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
        let existing_author = Author::new_with_timestamps(
            AuthorId::try_from(author_id_str).unwrap(),
            AuthorName::new("Old Name".to_string()).unwrap(),
            "もとのよみ".to_string(),
            created_at,
            previous_updated_at,
        )
        .unwrap();

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id_with_tx()
            .with(always(), always(), always())
            .returning(move |_, _, _| Ok(Some(existing_author.clone())));
        author_repository
            .expect_update()
            .with(always(), always())
            .returning(|_, _| Ok(404.into()));

        let interactor = command_interactor(author_repository, make_transaction_manager());
        let author_data = UpdateAuthorDto::new(author_id_str.to_string(), "New Name".to_string());

        // When
        let before = normalize_timestamp_for_persistence(OffsetDateTime::now_utc());
        let result = interactor.update("user1", author_data).await;
        let after = normalize_timestamp_for_persistence(OffsetDateTime::now_utc());

        // Then
        assert!(result.is_ok());
        let updated = result.unwrap();
        assert_eq!(updated.value.name, "New Name");
        assert_eq!(updated.value.yomi, "もとのよみ");
        assert_eq!(updated.value.created_at, created_at);
        assert!(updated.value.updated_at >= previous_updated_at);
        assert!(updated.value.updated_at >= before);
        assert!(updated.value.updated_at <= after);
        assert_eq!(updated.revision_number, 1);
    }

    #[tokio::test]
    async fn update_author_commit_failure_returns_no_result() {
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author = Author::new(
            AuthorId::try_from(author_id_str).unwrap(),
            AuthorName::new("Old Name".to_string()).unwrap(),
            OffsetDateTime::UNIX_EPOCH,
        )
        .unwrap();
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id_with_tx()
            .return_once(move |_, _, _| Ok(Some(author)));
        author_repository
            .expect_update()
            .returning(|_, _| Ok(404.into()));

        let mut tm = MockTransactionManager::new();
        tm.expect_begin().returning(|_, _| Ok(()));
        tm.expect_commit()
            .returning(|_| Err(DomainError::Unexpected("commit failed".to_string())));
        let interactor = command_interactor(author_repository, tm);
        let author_data = UpdateAuthorDto::new(author_id_str.to_string(), "New Name".to_string());

        let result = interactor.update("user1", author_data).await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn update_author_repository_failure_returns_no_result() {
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author = Author::new(
            AuthorId::try_from(author_id_str).unwrap(),
            AuthorName::new("Old Name".to_string()).unwrap(),
            OffsetDateTime::UNIX_EPOCH,
        )
        .unwrap();
        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id_with_tx()
            .return_once(move |_, _, _| Ok(Some(author)));
        author_repository
            .expect_update()
            .returning(|_, _| Err(DomainError::Unexpected("event insert failed".to_string())));

        let mut tm = MockTransactionManager::new();
        tm.expect_begin().returning(|_, _| Ok(()));
        tm.expect_commit().times(0);
        let interactor = command_interactor(author_repository, tm);
        let author_data = UpdateAuthorDto::new(author_id_str.to_string(), "New Name".to_string());

        let result = interactor.update("user1", author_data).await;

        assert!(matches!(result, Err(UseCaseError::Unexpected(_))));
    }

    #[tokio::test]
    async fn update_author_changes_yomi_when_provided() {
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let existing_author = Author::new_with_yomi(
            AuthorId::try_from(author_id_str).unwrap(),
            AuthorName::new("Old Name".to_string()).unwrap(),
            "もとのよみ".to_string(),
            OffsetDateTime::UNIX_EPOCH,
        )
        .unwrap();

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id_with_tx()
            .return_once(move |_, _, _| Ok(Some(existing_author)));
        author_repository
            .expect_update()
            .withf(|_, author| author.yomi() == "")
            .returning(|_, _| Ok(405.into()));

        let interactor = command_interactor(author_repository, make_transaction_manager());
        let mut author_data =
            UpdateAuthorDto::new(author_id_str.to_string(), "New Name".to_string());
        author_data.yomi = Some(String::new());

        let result = interactor.update("user1", author_data).await.unwrap();

        assert_eq!(result.value.yomi, "");
    }

    #[tokio::test]
    async fn update_author_fails_with_invalid_yomi() {
        let author_repository = MockAuthorRepository::new();
        let interactor = command_interactor(author_repository, MockTransactionManager::new());
        let mut author_data = UpdateAuthorDto::new(
            "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
            "New Name".to_string(),
        );
        author_data.yomi = Some("New Name".to_string());

        let result = interactor.update("user1", author_data).await;

        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn update_author_not_found() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_find_by_id_with_tx()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(None));

        let interactor = command_interactor(author_repository, {
            let mut tm = MockTransactionManager::new();
            tm.expect_begin().returning(|_, _| Ok(()));
            tm
        });
        let author_data = UpdateAuthorDto::new(author_id_str.to_string(), "New Name".to_string());

        // When
        let result = interactor.update("user1", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn update_author_fails_with_invalid_author_id() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor = command_interactor(author_repository, MockTransactionManager::new());
        let author_data = UpdateAuthorDto::new("not-a-uuid".to_string(), "New Name".to_string());

        // When
        let result = interactor.update("user1", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn update_author_fails_with_empty_name() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author_repository = MockAuthorRepository::new();
        let interactor = command_interactor(author_repository, MockTransactionManager::new());
        let author_data = UpdateAuthorDto::new(author_id_str.to_string(), "".to_string());

        // When
        let result = interactor.update("user1", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn update_author_fails_with_invalid_user_id() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author_repository = MockAuthorRepository::new();
        let interactor = command_interactor(author_repository, MockTransactionManager::new());
        let author_data = UpdateAuthorDto::new(author_id_str.to_string(), "New Name".to_string());

        // When
        let result = interactor.update("", author_data).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn delete_author_success() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_delete()
            .with(always(), always(), always())
            .returning(|_, _, _| Ok(()));

        let interactor = command_interactor(author_repository, make_transaction_manager());

        // When
        let result = interactor.delete("user1", author_id_str).await;

        // Then
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_author_propagates_not_found() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_delete()
            .with(always(), always(), always())
            .returning(|_, _, _| {
                Err(DomainError::NotFound {
                    entity_type: "author",
                    entity_id: "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
                    user_id: "user1".to_string(),
                })
            });

        let interactor = command_interactor(author_repository, make_transaction_manager());

        // When
        let result = interactor.delete("user1", author_id_str).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::NotFound { .. })));
    }

    #[tokio::test]
    async fn delete_author_propagates_has_associated_books() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";

        let mut author_repository = MockAuthorRepository::new();
        author_repository
            .expect_delete()
            .with(always(), always(), always())
            .returning(|_, _, _| {
                Err(DomainError::HasAssociatedBooks {
                    author_id: "006099b4-6c42-4ec4-8645-f6bd5b63eddc".to_string(),
                    user_id: "user1".to_string(),
                })
            });

        let interactor = command_interactor(author_repository, make_transaction_manager());

        // When
        let result = interactor.delete("user1", author_id_str).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Conflict(_))));
    }

    #[tokio::test]
    async fn delete_author_fails_with_invalid_author_id() {
        // Given
        let author_repository = MockAuthorRepository::new();
        let interactor = command_interactor(author_repository, MockTransactionManager::new());

        // When
        let result = interactor.delete("user1", "not-a-uuid").await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn delete_author_fails_with_invalid_user_id() {
        // Given
        let author_id_str = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let author_repository = MockAuthorRepository::new();
        let interactor = command_interactor(author_repository, MockTransactionManager::new());

        // When
        let result = interactor.delete("", author_id_str).await;

        // Then
        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn merge_author_rejects_identical_ids_before_transaction() {
        let id = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let interactor = AuthorCommandInteractor::new(
            MockAuthorRepository::new(),
            MockBookRepository::new(),
            MockAuthorEventRepository::new(),
            MockTransactionManager::new(),
        );

        let result = interactor
            .merge(
                "user1",
                crate::use_case::dto::author::MergeAuthorInputDto {
                    source_author_id: id.to_string(),
                    destination_author_id: id.to_string(),
                },
            )
            .await;

        assert!(matches!(result, Err(UseCaseError::Validation(_))));
    }

    #[tokio::test]
    async fn merge_author_without_books_deletes_source_and_records_destination() {
        let source_id = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let destination_id = "106099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let timestamp = OffsetDateTime::UNIX_EPOCH;
        let source = Author::new(
            AuthorId::try_from(source_id).unwrap(),
            AuthorName::new("Source".to_string()).unwrap(),
            timestamp,
        )
        .unwrap();
        let destination = Author::new(
            AuthorId::try_from(destination_id).unwrap(),
            AuthorName::new("Destination".to_string()).unwrap(),
            timestamp,
        )
        .unwrap();
        let mut sequence = Sequence::new();
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_find_by_author_id_with_tx()
            .in_sequence(&mut sequence)
            .returning(|_, _, _| Ok(vec![]));
        let mut author_repository = MockAuthorRepository::new();
        let mut locked = vec![source.clone(), destination.clone()].into_iter();
        author_repository
            .expect_find_by_id_with_tx()
            .times(2)
            .in_sequence(&mut sequence)
            .returning(move |_, _, _| Ok(Some(locked.next().unwrap())));
        book_repository
            .expect_update_all()
            .withf(|_, books| books.is_empty())
            .times(1)
            .returning(|_, _| Ok(()));
        author_repository
            .expect_record_unchanged_revision()
            .withf(move |_, author| author.id().to_string() == destination_id)
            .times(1)
            .returning(|_, _| Ok(()));
        author_repository
            .expect_delete()
            .withf(move |_, author_id, extra| {
                author_id.to_string() == source_id
                    && matches!(
                        extra,
                        Some(DeleteAuthorEventExtra::Merge {
                            destination_author_id
                        }) if destination_author_id.to_string() == destination_id
                    )
            })
            .returning(|_, _, _| Ok(()));
        let mut event_repository = MockAuthorEventRepository::new();
        event_repository
            .expect_append()
            .withf(move |_, event| {
                event.operation == EventOperation::MergeAsDestination
                    && event.author_id.to_string() == destination_id
                    && event.name.is_none()
                    && event.yomi.is_none()
                    && event.author_created_at.is_none()
                    && event.author_updated_at.is_none()
                    && event.extra
                        == Some(serde_json::json!({
                            "version": 1,
                            "source_author_id": source_id,
                        }))
            })
            .returning(|_, _| Ok(1.into()));
        let interactor = AuthorCommandInteractor::new(
            author_repository,
            book_repository,
            event_repository,
            make_transaction_manager(),
        );

        let result = interactor
            .merge(
                "user1",
                crate::use_case::dto::author::MergeAuthorInputDto {
                    source_author_id: source_id.to_string(),
                    destination_author_id: destination_id.to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(result.value.id, destination_id);
        assert_eq!(result.value.name, "Destination");
    }

    #[tokio::test]
    async fn merge_author_moves_multiple_books_without_duplicate_destination() {
        let source_id = "006099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let destination_id = "106099b4-6c42-4ec4-8645-f6bd5b63eddc";
        let other_id = AuthorId::try_from("206099b4-6c42-4ec4-8645-f6bd5b63eddc").unwrap();
        let source_author_id = AuthorId::try_from(source_id).unwrap();
        let destination_author_id = AuthorId::try_from(destination_id).unwrap();
        let timestamp = OffsetDateTime::UNIX_EPOCH;
        let source = Author::new(
            source_author_id.clone(),
            AuthorName::new("Source".to_string()).unwrap(),
            timestamp,
        )
        .unwrap();
        let destination = Author::new(
            destination_author_id.clone(),
            AuthorName::new("Destination".to_string()).unwrap(),
            timestamp,
        )
        .unwrap();
        let make_book = |title: &str, author_ids: Vec<AuthorId>| {
            Book::new(
                BookId::new(Uuid::new_v4()).unwrap(),
                BookTitle::new(title.to_string()).unwrap(),
                author_ids,
                Isbn::new(String::new()).unwrap(),
                ReadFlag::new(false),
                OwnedFlag::new(false),
                Priority::new(50).unwrap(),
                BookFormat::Unknown,
                BookStore::Unknown,
                timestamp,
                timestamp,
            )
            .unwrap()
        };
        let books = vec![
            make_book(
                "Needs destination",
                vec![source_author_id.clone(), other_id.clone()],
            ),
            make_book(
                "Already has destination",
                vec![
                    source_author_id.clone(),
                    destination_author_id.clone(),
                    other_id.clone(),
                ],
            ),
        ];

        let mut author_repository = MockAuthorRepository::new();
        let mut locked = vec![source, destination].into_iter();
        author_repository
            .expect_find_by_id_with_tx()
            .times(2)
            .returning(move |_, _, _| Ok(Some(locked.next().unwrap())));
        author_repository
            .expect_record_unchanged_revision()
            .withf(move |_, author| author.id().to_string() == destination_id)
            .times(1)
            .returning(|_, _| Ok(()));
        author_repository
            .expect_delete()
            .withf(move |_, author_id, extra| {
                author_id.to_string() == source_id
                    && matches!(
                        extra,
                        Some(DeleteAuthorEventExtra::Merge {
                            destination_author_id
                        }) if destination_author_id.to_string() == destination_id
                    )
            })
            .returning(|_, _, _| Ok(()));
        let mut book_repository = MockBookRepository::new();
        book_repository
            .expect_find_by_author_id_with_tx()
            .return_once(move |_, _, _| Ok(books));
        let expected_source = source_author_id.clone();
        let expected_destination = destination_author_id.clone();
        let expected_other = other_id.clone();
        book_repository
            .expect_update_all()
            .times(1)
            .withf(move |_, books| {
                books.len() == 2
                    && books.iter().all(|book| {
                        !book.author_ids().contains(&expected_source)
                            && book
                                .author_ids()
                                .iter()
                                .filter(|id| *id == &expected_destination)
                                .count()
                                == 1
                            && book.author_ids().contains(&expected_other)
                    })
                    && books
                        .iter()
                        .any(|book| book.title().as_str() == "Needs destination")
                    && books
                        .iter()
                        .any(|book| book.title().as_str() == "Already has destination")
            })
            .returning(|_, _| Ok(()));
        let mut event_repository = MockAuthorEventRepository::new();
        event_repository
            .expect_append()
            .withf(move |_, event| {
                event.operation == EventOperation::MergeAsDestination
                    && event.author_id.to_string() == destination_id
                    && event.name.is_none()
                    && event.yomi.is_none()
                    && event.author_created_at.is_none()
                    && event.author_updated_at.is_none()
                    && event.extra
                        == Some(serde_json::json!({
                            "version": 1,
                            "source_author_id": source_id,
                        }))
            })
            .returning(|_, _| Ok(2.into()));
        let interactor = AuthorCommandInteractor::new(
            author_repository,
            book_repository,
            event_repository,
            make_transaction_manager(),
        );

        let result = interactor
            .merge(
                "user1",
                crate::use_case::dto::author::MergeAuthorInputDto {
                    source_author_id: source_id.to_string(),
                    destination_author_id: destination_id.to_string(),
                },
            )
            .await;

        assert!(result.is_ok());
    }
}
