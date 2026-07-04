use crate::domain::task::{Status, Task};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(Debug)]
pub enum RepositoryError {
    Internal(String),
}

impl Display for RepositoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RepositoryError::Internal(s) => write!(f, "Internal error: {s}"),
        }
    }
}

impl std::error::Error for RepositoryError {}

#[async_trait::async_trait]
pub trait TaskRepository: Send + Sync {
    async fn find(&self, id: &Uuid) -> Result<Option<Task>, RepositoryError>;
    async fn insert(&self, task: &Task) -> Result<(), RepositoryError>;
    async fn list(&self) -> Result<Vec<Task>, RepositoryError>;
    async fn delete(&self, id: &Uuid) -> Result<Option<Task>, RepositoryError>;
    async fn update(
        &self,
        id: &Uuid,
        title: Option<String>,
        description: Option<String>,
        status: Option<Status>,
    ) -> Result<Option<Task>, RepositoryError>;
}
