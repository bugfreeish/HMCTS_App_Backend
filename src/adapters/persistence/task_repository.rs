use crate::application::ports::secondary::{RepositoryError, TaskRepository};
use crate::domain::task::{Status, Task};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

fn map_error(e: sqlx::Error) -> RepositoryError {
    RepositoryError::Internal(e.to_string())
}

#[derive(sqlx::FromRow)]
pub struct TaskRow {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<TaskRow> for Task {
    type Error = RepositoryError;
    fn try_from(row: TaskRow) -> Result<Self, Self::Error> {
        let status = Status::from_str(&row.status).map_err(RepositoryError::Internal)?;

        Ok(Task {
            id: row.id,
            title: row.title,
            description: row.description,
            status,
            due_date: row.due_date,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

pub struct PgTaskRepository {
    pool: PgPool,
}

impl PgTaskRepository {
    pub fn new(pool: PgPool) -> Self {
        PgTaskRepository { pool }
    }
}

#[async_trait::async_trait]
impl TaskRepository for PgTaskRepository {
    async fn insert(&self, task: &Task) -> Result<(), RepositoryError> {
        sqlx::query(
            "INSERT INTO tasks (id, title, description, status, due_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
            .bind(task.id)
            .bind(task.title.clone())
            .bind(task.description.clone())
            .bind(task.status.to_string())
            .bind(task.due_date)
            .bind(task.created_at)
            .bind(task.updated_at)
            .execute(&self.pool)
            .await
            .map_err(map_error)?;
        Ok(())
    }

    async fn find(&self, id: &Uuid) -> Result<Option<Task>, RepositoryError> {
        let row: Option<TaskRow> = sqlx::query_as("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_error)?;

        row.map(Task::try_from).transpose()
    }

    async fn list(&self) -> Result<Vec<Task>, RepositoryError> {
        let rows: Vec<TaskRow> = sqlx::query_as("SELECT * FROM tasks ORDER BY due_date ASC")
            .fetch_all(&self.pool)
            .await
            .map_err(map_error)?;

        rows.into_iter().map(Task::try_from).collect()
    }

    async fn delete(&self, id: &Uuid) -> Result<Option<Task>, RepositoryError> {
        let row: Option<TaskRow> = sqlx::query_as("DELETE FROM tasks WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(map_error)?;

        row.map(Task::try_from).transpose()
    }
    async fn update(
        &self,
        id: &Uuid,
        title: Option<String>,
        description: Option<String>,
        status: Option<Status>,
    ) -> Result<Option<Task>, RepositoryError> {
        let status_str = status.as_ref().map(|s| s.to_string());
        let row: Option<TaskRow> = sqlx::query_as(
            "UPDATE tasks SET
                status = COALESCE($1, status),
                description = COALESCE($2, description),
                title = COALESCE($3, title),
                updated_at = NOW()
            WHERE id = $4 RETURNING *",
        )
        .bind(status_str)
        .bind(&description.clone())
        .bind(&title.clone())
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_error)?;

        row.map(Task::try_from).transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    async fn insert_and_find_task(pool: PgPool) {
        let repo = PgTaskRepository::new(pool);
        let task = Task::new("test".into(), None, None).unwrap();
        repo.insert(&task).await.unwrap();
        let found = repo.find(&task.id).await.unwrap().unwrap();
        assert_eq!(found.id, task.id);
        assert_eq!(found.title, task.title);
        assert_eq!(found.description, task.description);
        assert_eq!(found.status, task.status);
        assert_eq!(found.due_date, task.due_date);
    }

    #[sqlx::test]
    async fn find_nonexistent_task(pool: PgPool) {
        let repo = PgTaskRepository::new(pool);
        let found = repo.find(&Uuid::new_v4()).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test]
    async fn list_tasks_empty_initially(pool: PgPool) {
        let repo = PgTaskRepository::new(pool);
        let tasks = repo.list().await.unwrap();
        assert!(tasks.is_empty());
    }

    #[sqlx::test]
    async fn list_tasks_returns_inserted(pool: PgPool) {
        let repo = PgTaskRepository::new(pool);
        let t1 = Task::new("a".into(), None, None).unwrap();
        let t2 = Task::new("b".into(), None, None).unwrap();
        repo.insert(&t1).await.unwrap();
        repo.insert(&t2).await.unwrap();
        let tasks = repo.list().await.unwrap();
        assert_eq!(tasks.len(), 2);
    }

    #[sqlx::test]
    async fn delete_task_removes(pool: PgPool) {
        let repo = PgTaskRepository::new(pool);
        let task = Task::new("to delete".into(), None, None).unwrap();
        repo.insert(&task).await.unwrap();
        let deleted = repo.delete(&task.id).await.unwrap().unwrap();
        assert_eq!(deleted.id, task.id);
        assert_eq!(deleted.title, task.title);
        assert_eq!(deleted.description, task.description);
        assert_eq!(deleted.status, task.status);
        assert_eq!(deleted.due_date, task.due_date);
        let found = repo.find(&task.id).await.unwrap();
        assert!(found.is_none());
    }

    #[sqlx::test]
    async fn delete_nonexistent(pool: PgPool) {
        let repo = PgTaskRepository::new(pool);
        let result = repo.delete(&Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[sqlx::test]
    async fn update_task_fields(pool: PgPool) {
        let repo = PgTaskRepository::new(pool);
        let task = Task::new("original".into(), None, None).unwrap();
        repo.insert(&task).await.unwrap();
        let updated = repo
            .update(
                &task.id,
                Some("new title".into()),
                Some("new desc".into()),
                Some(Status::Completed),
            )
            .await
            .unwrap();
        let t = updated.unwrap();
        assert_eq!(t.title, "new title");
        assert_eq!(t.description.as_deref(), Some("new desc"));
        assert_eq!(t.status, Status::Completed);
    }

    #[sqlx::test]
    async fn update_nonexistent(pool: PgPool) {
        let repo = PgTaskRepository::new(pool);
        let result = repo
            .update(&Uuid::new_v4(), None, None, None)
            .await
            .unwrap();
        assert!(result.is_none());
    }
}
