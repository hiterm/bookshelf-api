use super::{author::AuthorDto, book::BookDto};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MutationResultDto<T> {
    pub value: T,
    pub event_set_id: String,
}

impl<T> MutationResultDto<T> {
    pub fn new(value: T, event_set_id: String) -> Self {
        Self {
            value,
            event_set_id,
        }
    }
}

impl<T> std::ops::Deref for MutationResultDto<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

pub type BookMutationResultDto = MutationResultDto<BookDto>;
pub type AuthorMutationResultDto = MutationResultDto<AuthorDto>;
pub type DeleteBookResultDto = MutationResultDto<String>;
pub type DeleteAuthorResultDto = MutationResultDto<String>;
pub type ImportBooksResultDto = MutationResultDto<Vec<BookDto>>;
pub type RestoreBookResultDto = MutationResultDto<Option<BookDto>>;
pub type RestoreAuthorResultDto = MutationResultDto<Option<AuthorDto>>;
