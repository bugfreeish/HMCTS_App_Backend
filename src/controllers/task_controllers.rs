use crate::models::task::Status;
use crate::models::task::Task;
use crate::services::task_service::TaskService;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

pub type SharedService = Arc<TaskService>;

#[derive(Serialize, Debug)]
pub struct TaskResponse {
    id: Uuid,
    title: String,
    description: Option<String>,
    status: Status,
    #[serde(rename = "dueDate")]
    due_date: Option<String>,
    #[serde(rename = "createdAt")]
    created_at: String,
    #[serde(rename = "updatedAt")]
    updated_at: String,
}

#[derive(Deserialize, Debug)]
pub struct TaskRequest {
    title: String,
    description: Option<String>,
    #[serde(rename = "dueDate")]
    due_date: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Debug)]
pub struct TaskEditRequest {
    status: String,
}

impl From<Task> for TaskResponse {
    fn from(task: Task) -> Self {
        Self {
            id: task.id,
            title: task.title,
            description: task.description,
            status: task.status,
            due_date: task.due_date.map(|d| d.to_rfc3339()),
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.to_rfc3339(),
        }
    }
}

pub async fn create_task(
    State(service): State<SharedService>,
    Json(payload): Json<TaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), StatusCode> {
    let task = Task::new(payload.title, payload.description, payload.due_date);
    let resp = TaskResponse::from(task.clone());

    service
        .insert_new_task(&task)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn list_tasks(
    State(service): State<SharedService>,
) -> Result<Json<Vec<TaskResponse>>, StatusCode> {
    let tasks = service
        .list_tasks()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let items = tasks.into_iter().map(TaskResponse::from).collect();

    Ok(Json(items))
}

pub async fn get_task(
    State(service): State<SharedService>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Option<TaskResponse>>), StatusCode> {
    let task = service
        .get_task(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match task {
        Some(task) => Ok((StatusCode::OK, Json(Some(TaskResponse::from(task))))),
        None => Ok((StatusCode::OK, Json(None))),
    }
}

pub async fn edit_task(
    State(service): State<SharedService>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TaskEditRequest>,
) -> Result<(StatusCode, Json<Option<TaskResponse>>), StatusCode> {
    let status = match payload.status.as_str() {
        "pending" => Status::Pending,
        "in-progress" => Status::InProgress,
        "completed" => Status::Completed,
        _ => return Err(StatusCode::BAD_REQUEST),
    };

    let task = service
        .update_task_status(&id, &status)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match task {
        Some(task) => Ok((StatusCode::OK, Json(Some(TaskResponse::from(task))))),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn delete_task(
    State(service): State<SharedService>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Option<String>>), StatusCode> {
    let deleted = service
        .delete_task(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match deleted {
        Some(_) => Ok((
            StatusCode::OK,
            Json(Some(format!("Task {} has been deleted", id))),
        )),
        None => Err(StatusCode::NOT_FOUND),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_edit_request(status: &str) -> Json<TaskEditRequest> {
        Json(TaskEditRequest {
            status: status.into(),
        })
    }

    fn make_request(
        title: &str,
        description: Option<String>,
        due_date: Option<DateTime<Utc>>,
    ) -> Json<TaskRequest> {
        Json(TaskRequest {
            title: title.into(),
            description,
            due_date,
        })
    }

    #[sqlx::test]
    async fn create_task_returns_created(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let payload = make_request("test task", None, None);
        let result = create_task(State(service), payload).await;
        assert!(result.is_ok());
        let (status, resp) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(resp.title, "test task");
        assert!(resp.description.is_none());
        assert!(resp.due_date.is_none());
    }

    #[sqlx::test]
    async fn create_task_with_all_fields(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let due = Utc::now();
        let payload = make_request("full task", Some("a description".into()), Some(due));
        let result = create_task(State(service), payload).await;
        assert!(result.is_ok());
        let (_status, resp) = result.unwrap();
        assert_eq!(resp.title, "full task");
        assert_eq!(resp.description.as_deref(), Some("a description"));
        assert_eq!(resp.due_date.as_deref(), Some(due.to_rfc3339()).as_deref());
    }

    #[sqlx::test]
    async fn list_tasks_returns_empty_initially(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let result = list_tasks(State(service)).await;
        assert!(result.is_ok());
        let tasks = result.unwrap().0;
        assert!(tasks.is_empty());
    }

    #[sqlx::test]
    async fn list_tasks_returns_created_tasks(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let payload = make_request("task a", None, None);
        let _ = create_task(State(service.clone()), payload).await;
        let payload = make_request("task b", None, None);
        let _ = create_task(State(service.clone()), payload).await;
        let result = list_tasks(State(service)).await;
        assert!(result.is_ok());
        let tasks = result.unwrap().0;
        assert_eq!(tasks.len(), 2);
    }

    #[sqlx::test]
    async fn get_task_returns_none_for_unknown_id(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let result = get_task(State(service), Path(Uuid::new_v4())).await;
        assert!(result.is_ok());
        let (status, Json(task)) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(task.is_none());
    }

    #[sqlx::test]
    async fn get_task_returns_created_task(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let payload = make_request("my task", Some("desc".into()), None);
        let (_, Json(create_resp)) = create_task(State(service.clone()), payload).await.unwrap();
        let result = get_task(State(service), Path(create_resp.id)).await;
        assert!(result.is_ok());
        let (status, Json(task)) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        let resp = task.unwrap();
        assert_eq!(resp.title, "my task");
        assert_eq!(resp.description.as_deref(), Some("desc"));
        assert!(resp.due_date.is_none());
        assert_eq!(resp.status, Status::Pending);
    }

    #[sqlx::test]
    async fn get_task_returns_correct_fields(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let due = Utc::now();
        let payload = make_request("full task", Some("a description".into()), Some(due));
        let (_, Json(create_resp)) = create_task(State(service.clone()), payload).await.unwrap();
        let result = get_task(State(service), Path(create_resp.id)).await;
        assert!(result.is_ok());
        let (_, Json(task)) = result.unwrap();
        let resp = task.unwrap();
        assert_eq!(resp.title, "full task");
        assert_eq!(resp.description.as_deref(), Some("a description"));
        assert_eq!(resp.due_date.as_deref(), Some(due.to_rfc3339()).as_deref());
        assert_eq!(resp.status, Status::Pending);
    }

    #[sqlx::test]
    async fn edit_task_returns_not_found(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let payload = make_edit_request("pending");
        let result = edit_task(State(service), Path(Uuid::new_v4()), payload).await;
        match result {
            Err(code) => assert_eq!(code, StatusCode::NOT_FOUND),
            _ => panic!("expected error"),
        }
    }

    #[sqlx::test]
    async fn edit_task_returns_bad_request_for_invalid_status(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let payload = make_request("dummy", None, None);
        let (_, Json(created)) = create_task(State(service.clone()), payload).await.unwrap();
        let payload = make_edit_request("invalid-status");
        let result = edit_task(State(service), Path(created.id), payload).await;
        match result {
            Err(code) => assert_eq!(code, StatusCode::BAD_REQUEST),
            _ => panic!("expected error"),
        }
    }

    #[sqlx::test]
    async fn edit_task_updates_status_to_pending(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let payload = make_request("dummy", None, None);
        let (_, Json(created)) = create_task(State(service.clone()), payload).await.unwrap();
        let payload = make_edit_request("pending");
        let _ = edit_task(State(service.clone()), Path(created.id), payload).await;
        let (_, Json(task)) = get_task(State(service), Path(created.id)).await.unwrap();
        assert_eq!(task.unwrap().status, Status::Pending);
    }

    #[sqlx::test]
    async fn edit_task_updates_status_to_in_progress(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let payload = make_request("dummy", None, None);
        let (_, Json(created)) = create_task(State(service.clone()), payload).await.unwrap();
        let payload = make_edit_request("in-progress");
        let _ = edit_task(State(service.clone()), Path(created.id), payload).await;
        let (_, Json(task)) = get_task(State(service), Path(created.id)).await.unwrap();
        assert_eq!(task.unwrap().status, Status::InProgress);
    }

    #[sqlx::test]
    async fn edit_task_updates_status_to_completed(pool: sqlx::PgPool) {
        let service = Arc::new(TaskService::new(pool));
        let payload = make_request("dummy", None, None);
        let (_, Json(created)) = create_task(State(service.clone()), payload).await.unwrap();
        let payload = make_edit_request("completed");
        let _ = edit_task(State(service.clone()), Path(created.id), payload).await;
        let (_, Json(task)) = get_task(State(service), Path(created.id)).await.unwrap();
        assert_eq!(task.unwrap().status, Status::Completed);
    }
}
