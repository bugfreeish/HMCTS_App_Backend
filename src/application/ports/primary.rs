use crate::application::ports::secondary::RepositoryError;
use crate::domain::task::{Status, Task};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[async_trait::async_trait]
pub trait TaskUseCase: Send + Sync {
    async fn create_task(
        &self,
        title: String,
        description: Option<String>,
        due_date: Option<DateTime<Utc>>,
    ) -> Result<Task, RepositoryError>;

    async fn list_tasks(&self) -> Result<Vec<Task>, RepositoryError>;

    async fn get_task(&self, id: &Uuid) -> Result<Option<Task>, RepositoryError>;

    async fn delete_task(&self, id: &Uuid) -> Result<Option<Task>, RepositoryError>;

    async fn update_task(
        &self,
        id: &Uuid,
        title: Option<String>,
        description: Option<String>,
        status: Option<Status>,
    ) -> Result<Option<Task>, RepositoryError>;
}
