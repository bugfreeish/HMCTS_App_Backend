use axum::{Json, Router, routing::get};
use serde::Serialize;
use std::sync::Arc;

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
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = sqlx::postgres::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let shared_service = Arc::new(TaskService::new(pool));

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
