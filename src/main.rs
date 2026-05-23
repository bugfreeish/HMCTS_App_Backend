use axum::{Json, Router, routing::get};
use serde::Serialize;

mod controllers;
mod models;
mod routes;
mod services;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route(
        "/health",
        get(async || Json(HealthResponse { status: "OK" })),
    );
    // .nest("/tasks", routes::tasks);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
