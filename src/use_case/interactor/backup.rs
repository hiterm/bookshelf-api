use crate::{
    domain::{entity::user::UserId, error::DomainError},
    use_case::{
        dto::backup::{BackupData, BackupScope},
        port::backup::{BackupQueryPort, BackupQueryScope},
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

    pub async fn export(
        &self,
        user_id: &UserId,
        scope: BackupScope,
    ) -> Result<BackupData, DomainError> {
        let query_scope = match scope {
            BackupScope::Snapshot => BackupQueryScope::Snapshot,
            BackupScope::Full => BackupQueryScope::Full,
        };
        Ok(self.query.query(user_id, query_scope).await?.into())
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::eq;

    use crate::{
        domain::entity::user::UserId,
        use_case::{
            dto::backup::BackupScope,
            interactor::backup::BackupInteractor,
            port::backup::{
                BackupAuthorProjection, BackupProjection, BackupQueryScope, MockBackupQueryPort,
            },
        },
    };

    #[tokio::test]
    async fn snapshot_maps_the_query_projection_to_the_external_dto() {
        let user_id = UserId::new("owner".to_owned()).expect("valid user id");
        let mut query = MockBackupQueryPort::new();
        query
            .expect_query()
            .with(eq(user_id.clone()), eq(BackupQueryScope::Snapshot))
            .once()
            .returning(|_, _| {
                Ok(BackupProjection {
                    authors: vec![BackupAuthorProjection {
                        id: "author-id".to_owned(),
                        name: "Author".to_owned(),
                        yomi: "".to_owned(),
                        created_at: "2026-09-11T00:00:00Z".to_owned(),
                        updated_at: "2026-09-11T00:00:00Z".to_owned(),
                    }],
                    books: vec![],
                    history: None,
                })
            });

        let data = BackupInteractor::new(query)
            .export(&user_id, BackupScope::Snapshot)
            .await
            .expect("backup query succeeds");

        assert_eq!(data.authors[0].id, "author-id");
        assert!(data.history.is_none());
    }

    #[tokio::test]
    async fn full_forwards_the_full_query_scope() {
        let user_id = UserId::new("owner".to_owned()).expect("valid user id");
        let mut query = MockBackupQueryPort::new();
        query
            .expect_query()
            .with(eq(user_id.clone()), eq(BackupQueryScope::Full))
            .once()
            .returning(|_, _| {
                Ok(BackupProjection {
                    authors: vec![],
                    books: vec![],
                    history: None,
                })
            });

        BackupInteractor::new(query)
            .export(&user_id, BackupScope::Full)
            .await
            .expect("backup query succeeds");
    }
}
