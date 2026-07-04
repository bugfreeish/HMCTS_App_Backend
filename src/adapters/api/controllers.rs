use crate::adapters::api::error::AppError;
use crate::application::ports::primary::TaskUseCase;
use crate::domain::task::{Status, Task};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::Arc;
use uuid::Uuid;

pub type AppState = Arc<dyn TaskUseCase>;

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
    title: Option<String>,
    status: Option<String>,
    description: Option<String>,
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
    State(service): State<AppState>,
    Json(payload): Json<TaskRequest>,
) -> Result<(StatusCode, Json<TaskResponse>), AppError> {
    let task = service
        .create_task(payload.title, payload.description, payload.due_date)
        .await?;

    Ok((StatusCode::CREATED, Json(TaskResponse::from(task))))
}

pub async fn list_tasks(
    State(service): State<AppState>,
) -> Result<Json<Vec<TaskResponse>>, AppError> {
    let tasks = service.list_tasks().await?;

    Ok(Json(tasks.into_iter().map(TaskResponse::from).collect()))
}

pub async fn get_task(
    State(service): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Option<TaskResponse>>, AppError> {
    let task = service.get_task(&id).await?;

    Ok(Json(task.map(TaskResponse::from)))
}

pub async fn edit_task(
    State(service): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<TaskEditRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let status = payload
        .status
        .map(|str| Status::from_str(str.as_str()))
        .transpose()?;

    let task = service
        .update_task(&id, payload.title, payload.description, status)
        .await?
        .ok_or(AppError::NotFound)?;
    Ok(Json(TaskResponse::from(task)))
}

pub async fn delete_task(
    State(service): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<String>, AppError> {
    service.delete_task(&id).await?.ok_or(AppError::NotFound)?;

    Ok(Json(format!("Task {id} has been deleted")))
}
