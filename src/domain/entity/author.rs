use std::fmt::Display;
use std::sync::LazyLock;

use getset::Getters;
use regex::Regex;
use time::OffsetDateTime;
use uuid::Uuid;
use validator::Validate;

use crate::{
    common::time::normalize_timestamp_for_persistence, domain::error::DomainError,
    impl_string_value_object,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AuthorId {
    id: Uuid,
}

impl AuthorId {
    pub fn new(id: Uuid) -> Self {
        Self { id }
    }

    pub fn to_uuid(&self) -> Uuid {
        self.id
    }
}

impl Display for AuthorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id.hyphenated())
    }
}

impl TryFrom<&str> for AuthorId {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let id = Uuid::parse_str(value).map_err(|err| {
            DomainError::Validation(format!(
                r#"Failed to parse id "{}" as uuid. Message from uuid crate: {}"#,
                value, err
            ))
        })?;
        Ok(AuthorId { id })
    }
}

impl From<Uuid> for AuthorId {
    fn from(uuid: Uuid) -> Self {
        AuthorId { id: uuid }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Validate)]
pub struct AuthorName {
    #[validate(length(min = 1))]
    value: String,
}

impl_string_value_object!(AuthorName);

static AUTHOR_YOMI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[\p{Hiragana}0-9０-９ー・ 　-]*\z")
        .expect("AUTHOR_YOMI_REGEX is a hardcoded valid pattern")
});

pub fn validate_author_yomi(yomi: String) -> Result<String, DomainError> {
    if AUTHOR_YOMI_REGEX.is_match(&yomi) {
        Ok(yomi)
    } else {
        Err(DomainError::Validation(
            "author yomi contains unsupported characters".to_string(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters)]
pub struct Author {
    #[getset(get = "pub")]
    id: AuthorId,
    #[getset(get = "pub")]
    name: AuthorName,
    #[getset(get = "pub")]
    yomi: String,
    #[getset(get = "pub")]
    created_at: OffsetDateTime,
    #[getset(get = "pub")]
    updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestructureAuthor {
    pub id: AuthorId,
    pub name: AuthorName,
    pub yomi: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorUpdate {
    pub name: AuthorName,
    pub yomi: Option<String>,
}

impl Author {
    pub fn new(id: AuthorId, name: AuthorName) -> Result<Author, DomainError> {
        Self::new_with_yomi(id, name, String::new())
    }

    pub fn new_with_yomi(
        id: AuthorId,
        name: AuthorName,
        yomi: String,
    ) -> Result<Author, DomainError> {
        let now = OffsetDateTime::now_utc();
        Self::new_with_timestamps(id, name, yomi, now, now)
    }

    pub fn new_with_timestamps(
        id: AuthorId,
        name: AuthorName,
        yomi: String,
        created_at: OffsetDateTime,
        updated_at: OffsetDateTime,
    ) -> Result<Author, DomainError> {
        Ok(Author {
            id,
            name,
            yomi,
            created_at: normalize_timestamp_for_persistence(created_at),
            updated_at: normalize_timestamp_for_persistence(updated_at),
        })
    }

    pub fn update(&mut self, update: AuthorUpdate, updated_at: OffsetDateTime) {
        self.name = update.name;
        if let Some(yomi) = update.yomi {
            self.yomi = yomi;
        }
        self.updated_at = normalize_timestamp_for_persistence(updated_at);
    }

    pub fn destructure(self) -> DestructureAuthor {
        DestructureAuthor {
            id: self.id,
            name: self.name,
            yomi: self.yomi,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use time::OffsetDateTime;

    use crate::domain::{
        entity::author::{Author, AuthorId, AuthorName, AuthorUpdate, validate_author_yomi},
        error::DomainError,
    };

    #[test]
    fn author_id_to_string() {
        let uuid_str = "c6ea22c8-7b70-470c-a713-c7aade5693bd";
        let author_id = AuthorId::try_from(uuid_str).unwrap();
        assert_eq!(author_id.to_string(), uuid_str);
    }

    #[test]
    fn update_changes_name() {
        let mut author = Author::new(
            AuthorId::try_from("c6ea22c8-7b70-470c-a713-c7aade5693bd").unwrap(),
            AuthorName::new(String::from("author1")).unwrap(),
        )
        .unwrap();

        let updated_at = OffsetDateTime::from_unix_timestamp(1_700_000_000).unwrap();
        author.update(
            AuthorUpdate {
                name: AuthorName::new(String::from("author2")).unwrap(),
                yomi: None,
            },
            updated_at,
        );

        assert_eq!(author.name().as_str(), "author2");
        assert_eq!(author.updated_at(), &updated_at);
    }

    #[test]
    fn validation_success() {
        assert!(AuthorName::new(String::from("author1")).is_ok());
    }

    #[test]
    fn validation_failure() {
        assert!(matches!(
            AuthorName::new(String::from("")),
            Err(DomainError::Validation(_))
        ));
    }

    #[test]
    fn author_yomi_accepts_supported_characters_and_empty_string() {
        for yomi in ["", "やまだ たろう", "じぇーん・どー", "だい２-3"] {
            assert!(validate_author_yomi(yomi.to_string()).is_ok(), "{yomi}");
        }
    }

    #[test]
    fn author_yomi_rejects_unsupported_characters() {
        for yomi in ["山田太郎", "ヤマダ", "yamada", "やまだ!", "やまだ\n"] {
            assert!(validate_author_yomi(yomi.to_string()).is_err(), "{yomi}");
        }
    }
}
