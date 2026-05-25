use axum::{Json, Router, routing::get};
use serde::Serialize;
use std::sync::{Arc, RwLock};

mod controllers;
mod models;
mod routes;
mod services;

use services::task_service::TaskService;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[tokio::main]
async fn main() {
    let shared_service = Arc::new(RwLock::new(TaskService::new()));

    let app = Router::new()
        .route(
            "/health",
            get(async || Json(HealthResponse { status: "OK" })),
        )
        .nest("/tasks", routes::tasks::router())
        .with_state(shared_service);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
