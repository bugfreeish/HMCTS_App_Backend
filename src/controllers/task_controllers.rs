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
use std::sync::{Arc, RwLock};
use uuid::Uuid;

pub type SharedService = Arc<RwLock<TaskService>>;

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

pub async fn create_task(
    State(service): State<SharedService>,
    Json(payload): Json<TaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), StatusCode> {
    let task = Task::new(payload.title, payload.description, payload.due_date);
    let resp = TaskResponse {
        id: task.id,
        title: task.title.clone(),
        description: task.description.clone(),
        status: task.status.lock().unwrap().clone(),
        due_date: task.due_date.map(|d| d.to_rfc3339()),
        created_at: task.created_at.to_rfc3339(),
        updated_at: task.updated_at.lock().unwrap().to_rfc3339(),
    };
    service
        .write()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .insert_new_task(task);
    Ok((StatusCode::CREATED, Json(resp)))
}

pub async fn list_tasks(
    State(service): State<SharedService>,
) -> Result<Json<Vec<TaskResponse>>, StatusCode> {
    let tasks = service
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let items = tasks
        .list_tasks()
        .iter()
        .map(|task| TaskResponse {
            id: task.id,
            title: task.title.clone(),
            description: task.description.clone(),
            status: task.status.lock().unwrap().clone(),
            due_date: task.due_date.map(|d| d.to_rfc3339()),
            created_at: task.created_at.to_rfc3339(),
            updated_at: task.updated_at.lock().unwrap().to_rfc3339(),
        })
        .collect();

    Ok(Json(items))
}

pub async fn get_task(
    State(service): State<SharedService>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<Option<TaskResponse>>), StatusCode> {
    let tasks = service
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let task = tasks.get_task(&id);

    match task {
        Some(task) => {
            let resp = TaskResponse {
                id: task.id,
                title: task.title.clone(),
                description: task.description.clone(),
                status: task.status.lock().unwrap().clone(),
                due_date: task.due_date.map(|d| d.to_rfc3339()),
                created_at: task.created_at.to_rfc3339(),
                updated_at: task.updated_at.lock().unwrap().to_rfc3339(),
            };
            Ok((StatusCode::OK, Json(Some(resp))))
        }

        None => Ok((StatusCode::OK, Json(None))),
    }
}

pub async fn edit_task(
    State(service): State<SharedService>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TaskEditRequest>,
) -> Result<(StatusCode, Json<Option<TaskResponse>>), StatusCode> {
    let tasks = service
        .read()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let task = tasks.get_task(&id).ok_or(StatusCode::NOT_FOUND)?;

    match payload.status.as_str() {
        "pending" => task.set_status(Status::Pending),
        "in-progress" => task.set_status(Status::InProgress),
        "completed" => task.set_status(Status::Completed),
        _ => return Err(StatusCode::BAD_REQUEST),
    }

    task.set_updated_at();

    let resp = TaskResponse {
        id: task.id,
        title: task.title.clone(),
        description: task.description.clone(),
        status: task.status.lock().unwrap().clone(),
        due_date: task.due_date.map(|d| d.to_rfc3339()),
        created_at: task.created_at.to_rfc3339(),
        updated_at: task.updated_at.lock().unwrap().to_rfc3339(),
    };

    Ok((StatusCode::OK, Json(Some(resp))))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn shared_service() -> SharedService {
        Arc::new(RwLock::new(TaskService::new()))
    }

    fn make_edit_request(status: &str) -> Json<TaskEditRequest> {
        Json(TaskEditRequest {
            status: status.into(),
        })
    }

    async fn create_dummy_task(service: &SharedService) -> Uuid {
        let payload = make_request("dummy", None, None);
        let (_, Json(resp)) = create_task(State(service.clone()), payload).await.unwrap();

        resp.id
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

    #[tokio::test]
    async fn create_task_returns_created() {
        let service = shared_service();
        let payload = make_request("test task", None, None);
        let result = create_task(State(service), payload).await;
        assert!(result.is_ok());
        let (status, resp) = result.unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(resp.title, "test task");
        assert!(resp.description.is_none());
        assert!(resp.due_date.is_none());
    }

    #[tokio::test]
    async fn create_task_with_all_fields() {
        let service = shared_service();
        let due = Utc::now();
        let payload = make_request("full task", Some("a description".into()), Some(due));
        let result = create_task(State(service), payload).await;
        assert!(result.is_ok());
        let (_status, resp) = result.unwrap();
        assert_eq!(resp.title, "full task");
        assert_eq!(resp.description.as_deref(), Some("a description"));
        assert_eq!(resp.due_date.as_deref(), Some(due.to_rfc3339()).as_deref());
    }

    #[tokio::test]
    async fn list_tasks_returns_empty_initially() {
        let service = shared_service();
        let result = list_tasks(State(service)).await;
        assert!(result.is_ok());
        let tasks = result.unwrap().0;
        assert!(tasks.is_empty());
    }

    #[tokio::test]
    async fn list_tasks_returns_created_tasks() {
        let service = shared_service();
        let payload = make_request("task a", None, None);
        let _ = create_task(State(service.clone()), payload).await;
        let payload = make_request("task b", None, None);
        let _ = create_task(State(service.clone()), payload).await;
        let result = list_tasks(State(service)).await;
        assert!(result.is_ok());
        let tasks = result.unwrap().0;
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn get_task_returns_none_for_unknown_id() {
        let service = shared_service();
        let result = get_task(State(service), Path(Uuid::new_v4())).await;
        assert!(result.is_ok());
        let (status, Json(task)) = result.unwrap();
        assert_eq!(status, StatusCode::OK);
        assert!(task.is_none());
    }

    #[tokio::test]
    async fn get_task_returns_created_task() {
        let service = shared_service();
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
        assert_eq!(resp.status, "Pending");
    }

    #[tokio::test]
    async fn get_task_returns_correct_fields() {
        let service = shared_service();
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
        assert_eq!(resp.status, "Pending");
    }

    #[tokio::test]
    async fn edit_task_returns_not_found() {
        let service = shared_service();
        let payload = make_edit_request("pending");
        let result = edit_task(State(service), Path(Uuid::new_v4()), payload).await;
        match result {
            Err(code) => assert_eq!(code, StatusCode::NOT_FOUND),
            _ => panic!("expected error"),
        }
    }

    #[tokio::test]
    async fn edit_task_returns_bad_request_for_invalid_status() {
        let service = shared_service();
        let id = create_dummy_task(&service).await;
        let payload = make_edit_request("invalid-status");
        let result = edit_task(State(service), Path(id), payload).await;
        match result {
            Err(code) => assert_eq!(code, StatusCode::BAD_REQUEST),
            _ => panic!("expected error"),
        }
    }

    #[tokio::test]
    async fn edit_task_updates_status_to_pending() {
        let service = shared_service();
        let id = create_dummy_task(&service).await;
        let payload = make_edit_request("pending");
        let result = edit_task(State(service.clone()), Path(id), payload).await;
        assert!(result.is_ok());
        let (_, Json(get_resp)) = get_task(State(service), Path(id)).await.unwrap();
        assert_eq!(get_resp.unwrap().status, "Pending");
    }

    #[tokio::test]
    async fn edit_task_updates_status_to_in_progress() {
        let service = shared_service();
        let id = create_dummy_task(&service).await;
        let payload = make_edit_request("in-progress");
        let result = edit_task(State(service.clone()), Path(id), payload).await;
        assert!(result.is_ok());
        let (_, Json(get_resp)) = get_task(State(service), Path(id)).await.unwrap();
        assert_eq!(get_resp.unwrap().status, "InProgress");
    }

    #[tokio::test]
    async fn edit_task_updates_status_to_completed() {
        let service = shared_service();
        let id = create_dummy_task(&service).await;
        let payload = make_edit_request("completed");
        let result = edit_task(State(service.clone()), Path(id), payload).await;
        assert!(result.is_ok());
        let (_, Json(get_resp)) = get_task(State(service), Path(id)).await.unwrap();
        assert_eq!(get_resp.unwrap().status, "Completed");
    }
}
