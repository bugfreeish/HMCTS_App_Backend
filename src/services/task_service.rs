use crate::models::task::{Status, Task};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TaskService {
    pool: PgPool,
}

impl TaskService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn insert_new_task(&self, task: &Task) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO tasks (id, title, description, status, due_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(task.id)
        .bind(&task.title)
        .bind(&task.description)
        .bind(&task.status)
        .bind(task.due_date)
        .bind(task.created_at)
        .bind(task.updated_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_task(&self, id: &Uuid) -> Result<Option<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>("SELECT * FROM tasks WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn list_tasks(&self) -> Result<Vec<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>("SELECT * FROM tasks")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn delete_task(&self, id: &Uuid) -> Result<Option<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>("DELETE FROM tasks WHERE id = $1 RETURNING *")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn update_task(
        &self,
        id: &Uuid,
        status: &Option<Status>,
        description: &Option<String>,
        title: &Option<String>,
    ) -> Result<Option<Task>, sqlx::Error> {
        sqlx::query_as::<_, Task>(
            "UPDATE tasks SET
                    status = COALESCE($1, status),
                    description = COALESCE($2, description),
                    title = COALESCE($3, title),
                    updated_at = NOW()
                WHERE id = $4 RETURNING *",
        )
        .bind(status)
        .bind(description)
        .bind(title)
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }
}
