use crate::{
    domain::{entity::user::UserId, error::DomainError},
    use_case::{
        dto::backup::{BackupFullData, BackupSnapshotData},
        port::backup::BackupQueryPort,
    },
};

#[derive(Debug, Clone)]
pub struct BackupInteractor<Q: BackupQueryPort> {
    query: Q,
}

impl<Q: BackupQueryPort> BackupInteractor<Q> {
    pub fn new(query: Q) -> Self {
        Self { query }
    }

    pub async fn snapshot(&self, user_id: &UserId) -> Result<BackupSnapshotData, DomainError> {
        Ok(self.query.snapshot(user_id).await?.into())
    }

    pub async fn full(&self, user_id: &UserId) -> Result<BackupFullData, DomainError> {
        Ok(self.query.full(user_id).await?.into())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;
    use serde_json::json;
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    use crate::{
        common::types::{BookFormat, BookStore},
        domain::entity::user::UserId,
        use_case::{
            interactor::backup::BackupInteractor,
            port::backup::{
                BackupAuthorChangeProjection, BackupAuthorProjection,
                BackupAuthorRevisionProjection, BackupAuthorSnapshotProjection,
                BackupBookChangeProjection, BackupBookProjection, BackupBookRevisionProjection,
                BackupBookSnapshotProjection, BackupChangesProjection, BackupFullProjection,
                BackupHistoryProjection, BackupOperationProjection, BackupSnapshotProjection,
                MockBackupQueryPort,
            },
        },
    };

    fn timestamp(value: &str) -> OffsetDateTime {
        OffsetDateTime::parse(value, &Rfc3339).expect("valid timestamp")
    }

    #[tokio::test]
    async fn snapshot_maps_the_query_projection_to_the_external_dto() {
        let user_id = UserId::new("owner".to_owned()).expect("valid user id");
        let mut query = MockBackupQueryPort::new();
        query
            .expect_snapshot()
            .with(eq(user_id.clone()))
            .once()
            .returning(|_| {
                Ok(BackupSnapshotProjection {
                    authors: vec![BackupAuthorProjection {
                        id: "author-id".to_owned(),
                        name: "Author".to_owned(),
                        yomi: "".to_owned(),
                        created_at: timestamp("2026-09-11T00:00:00Z"),
                        updated_at: timestamp("2026-09-11T00:00:00Z"),
                    }],
                    books: vec![],
                })
            });

        let data = BackupInteractor::new(query)
            .snapshot(&user_id)
            .await
            .expect("backup query succeeds");

        assert_eq!(data.authors[0].id, "author-id");
    }

    #[tokio::test]
    async fn full_calls_the_full_query_and_returns_history() {
        let user_id = UserId::new("owner".to_owned()).expect("valid user id");
        let mut query = MockBackupQueryPort::new();
        query
            .expect_full()
            .with(eq(user_id.clone()))
            .once()
            .returning(|_| {
                Ok(BackupFullProjection {
                    authors: vec![],
                    books: vec![],
                    history: BackupHistoryProjection {
                        operations: vec![],
                        book_revisions: vec![],
                        author_revisions: vec![],
                    },
                })
            });

        let data = BackupInteractor::new(query)
            .full(&user_id)
            .await
            .expect("backup query succeeds");

        assert!(data.history.operations.is_empty());
    }

    #[tokio::test]
    async fn full_maps_the_complete_projection_hierarchy_to_the_dto() {
        let user_id = UserId::new("owner".to_owned()).expect("valid user id");
        let book_snapshot = || BackupBookSnapshotProjection {
            title: "Updated Book".to_owned(),
            author_ids: vec!["author-id".to_owned()],
            isbn: "9780000000000".to_owned(),
            read: true,
            owned: true,
            priority: 7,
            format: BookFormat::Printed,
            store: BookStore::Unknown,
            purchase_date: None,
            created_at: timestamp("2026-09-10T00:00:00Z"),
            updated_at: timestamp("2026-09-11T00:00:00Z"),
        };
        let mut query = MockBackupQueryPort::new();
        query
            .expect_full()
            .with(eq(user_id.clone()))
            .once()
            .return_once(move |_| {
                Ok(BackupFullProjection {
                    authors: vec![],
                    books: vec![BackupBookProjection {
                        id: "book-id".to_owned(),
                        snapshot: book_snapshot(),
                    }],
                    history: BackupHistoryProjection {
                        operations: vec![BackupOperationProjection {
                            id: "operation-id".to_owned(),
                            operation_type: "update_book".to_owned(),
                            detail: Some(json!({"source": "fixture"})),
                            undo_of_operation_id: Some("previous-operation-id".to_owned()),
                            created_at: timestamp("2026-09-11T00:00:01Z"),
                            changes: BackupChangesProjection {
                                books: vec![BackupBookChangeProjection {
                                    book_id: "book-id".to_owned(),
                                    before_revision_number: Some(1),
                                    after_revision_number: Some(2),
                                }],
                                authors: vec![BackupAuthorChangeProjection {
                                    author_id: "author-id".to_owned(),
                                    before_revision_number: None,
                                    after_revision_number: Some(1),
                                }],
                            },
                        }],
                        book_revisions: vec![BackupBookRevisionProjection {
                            book_id: "book-id".to_owned(),
                            revision_number: 2,
                            snapshot: book_snapshot(),
                            recorded_at: timestamp("2026-09-11T00:00:02Z"),
                        }],
                        author_revisions: vec![BackupAuthorRevisionProjection {
                            author_id: "author-id".to_owned(),
                            revision_number: 1,
                            snapshot: BackupAuthorSnapshotProjection {
                                name: "Author".to_owned(),
                                yomi: "".to_owned(),
                                created_at: timestamp("2026-09-10T00:00:00Z"),
                                updated_at: timestamp("2026-09-10T00:00:00Z"),
                            },
                            recorded_at: timestamp("2026-09-10T00:00:01Z"),
                        }],
                    },
                })
            });

        let data = BackupInteractor::new(query)
            .full(&user_id)
            .await
            .expect("backup query succeeds");

        assert_eq!(data.books[0].id, "book-id");
        assert_eq!(data.books[0].snapshot.title, "Updated Book");
        let history = data.history;
        let operation = &history.operations[0];
        assert_eq!(operation.operation_type, "update_book");
        assert_eq!(operation.detail, Some(json!({"source": "fixture"})));
        assert_eq!(
            operation.undo_of_operation_id.as_deref(),
            Some("previous-operation-id")
        );
        assert_eq!(operation.changes.books[0].book_id, "book-id");
        assert_eq!(operation.changes.books[0].before_revision_number, Some(1));
        assert_eq!(operation.changes.books[0].after_revision_number, Some(2));
        assert_eq!(operation.changes.authors[0].author_id, "author-id");
        assert_eq!(history.book_revisions[0].snapshot.author_ids, ["author-id"]);
        assert_eq!(history.author_revisions[0].snapshot.name, "Author");
    }
}
