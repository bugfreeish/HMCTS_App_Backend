use crate::application::ports::primary::TaskUseCase;
use crate::application::ports::secondary::{RepositoryError, TaskRepository};
use crate::domain::task::{Status, Task};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

pub struct TaskService {
    repo: Arc<dyn TaskRepository>,
}

impl TaskService {
    pub fn new(repo: Arc<dyn TaskRepository>) -> Self {
        Self { repo }
    }
}

#[async_trait::async_trait]
impl TaskUseCase for TaskService {
    async fn create_task(
        &self,
        title: String,
        description: Option<String>,
        due_date: Option<DateTime<Utc>>,
    ) -> Result<Task, RepositoryError> {
        let task = Task::new(title, description, due_date)
            .map_err(|e| RepositoryError::Internal(e.to_string()))?;
        self.repo.insert(&task).await?;

        Ok(task)
    }

    async fn list_tasks(&self) -> Result<Vec<Task>, RepositoryError> {
        self.repo.list().await
    }

    async fn get_task(&self, id: &Uuid) -> Result<Option<Task>, RepositoryError> {
        self.repo.find(id).await
    }

    async fn delete_task(&self, id: &Uuid) -> Result<Option<Task>, RepositoryError> {
        self.repo.delete(id).await
    }

    async fn update_task(
        &self,
        id: &Uuid,
        title: Option<String>,
        description: Option<String>,
        status: Option<Status>,
    ) -> Result<Option<Task>, RepositoryError> {
        self.repo.update(id, title, description, status).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct MockRepo {
        tasks: Mutex<HashMap<Uuid, Task>>,
    }

    impl MockRepo {
        fn new() -> Self {
            Self {
                tasks: Mutex::new(HashMap::new()),
            }
        }
    }

    #[async_trait::async_trait]
    impl TaskRepository for MockRepo {
        async fn find(&self, id: &Uuid) -> Result<Option<Task>, RepositoryError> {
            let tasks = self.tasks.lock().unwrap();
            Ok(tasks.get(id).cloned())
        }

        async fn insert(&self, task: &Task) -> Result<(), RepositoryError> {
            let mut tasks = self.tasks.lock().unwrap();
            tasks.insert(task.id, task.clone());
            Ok(())
        }

        async fn list(&self) -> Result<Vec<Task>, RepositoryError> {
            let tasks = self.tasks.lock().unwrap();
            Ok(tasks.values().cloned().collect())
        }

        async fn delete(&self, id: &Uuid) -> Result<Option<Task>, RepositoryError> {
            let mut tasks = self.tasks.lock().unwrap();
            Ok(tasks.remove(id))
        }

        async fn update(
            &self,
            id: &Uuid,
            title: Option<String>,
            description: Option<String>,
            status: Option<Status>,
        ) -> Result<Option<Task>, RepositoryError> {
            let mut tasks = self.tasks.lock().unwrap();
            let task = match tasks.get_mut(id) {
                Some(task) => task,
                None => return Ok(None),
            };
            if let Some(title) = title {
                task.title = title;
            }
            if let Some(description) = description {
                task.description = Some(description);
            }
            if let Some(status) = status {
                task.status = status;
            }
            Ok(Some(task.clone()))
        }
    }

    fn service() -> TaskService {
        let repo = Arc::new(MockRepo::new());
        TaskService::new(repo)
    }

    #[tokio::test]
    async fn create_task_returns_task() {
        let service = service();
        let task = service
            .create_task("test".into(), None, None)
            .await
            .unwrap();
        assert_eq!(task.title, "test");
        assert_eq!(task.status, Status::Pending);
    }

    #[tokio::test]
    async fn create_task_stores_task() {
        let repo = Arc::new(MockRepo::new());
        let service = TaskService::new(repo.clone());
        let task = service
            .create_task("test".into(), None, None)
            .await
            .unwrap();
        let found = repo.find(&task.id).await.unwrap();
        assert_eq!(found.unwrap().id, task.id);
    }

    #[tokio::test]
    async fn list_tasks_returns_empty_initially() {
        let service = service();
        let tasks = service.list_tasks().await.unwrap();
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn list_tasks_returns_created_tasks() {
        let service = service();
        service.create_task("a".into(), None, None).await.unwrap();
        service.create_task("b".into(), None, None).await.unwrap();
        let tasks = service.list_tasks().await.unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn get_task_returns_created_task() {
        let service = service();
        let created = service
            .create_task("test".into(), None, None)
            .await
            .unwrap();
        let found = service.get_task(&created.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, created.id);
    }

    #[tokio::test]
    async fn get_task_returns_none_for_missing_id() {
        let service = service();
        let result = service.get_task(&Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn get_task_does_not_remove_task() {
        let service = service();
        let created = service
            .create_task("test".into(), None, None)
            .await
            .unwrap();
        let _ = service.get_task(&created.id).await.unwrap();
        let remaining = service.list_tasks().await.unwrap();
        assert_eq!(remaining.len(), 1);
    }

    #[tokio::test]
    async fn delete_task_removes_and_returns_task() {
        let service = service();
        let created = service
            .create_task("test".into(), None, None)
            .await
            .unwrap();
        let deleted = service.delete_task(&created.id).await.unwrap();
        assert!(deleted.is_some());
        assert_eq!(deleted.unwrap().id, created.id);
        let tasks = service.list_tasks().await.unwrap();
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn delete_task_returns_none_for_missing_id() {
        let service = service();
        let result = service.delete_task(&Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn update_task_modifies_title() {
        let service = service();
        let created = service
            .create_task("original".into(), None, None)
            .await
            .unwrap();
        let updated = service
            .update_task(&created.id, Some("modified".into()), None, None)
            .await
            .unwrap();
        assert_eq!(updated.unwrap().title, "modified");
    }

    #[tokio::test]
    async fn update_task_modifies_status() {
        let service = service();
        let created = service
            .create_task("test".into(), None, None)
            .await
            .unwrap();
        let updated = service
            .update_task(&created.id, None, None, Some(Status::Completed))
            .await
            .unwrap();
        assert_eq!(updated.unwrap().status, Status::Completed);
    }

    #[tokio::test]
    async fn update_task_modifies_multiple_fields() {
        let service = service();
        let created = service
            .create_task("original".into(), None, None)
            .await
            .unwrap();
        let updated = service
            .update_task(
                &created.id,
                Some("new".into()),
                Some("desc".into()),
                Some(Status::InProgress),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated.title, "new");
        assert_eq!(updated.description.as_deref(), Some("desc"));
        assert_eq!(updated.status, Status::InProgress);
    }

    #[tokio::test]
    async fn update_task_returns_none_for_missing_id() {
        let service = service();
        let result = service
            .update_task(&Uuid::new_v4(), None, None, None)
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
