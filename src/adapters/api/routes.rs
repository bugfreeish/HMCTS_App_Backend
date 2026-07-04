use crate::adapters::api::controllers::{
    create_task, delete_task, edit_task, get_task, list_tasks, AppState,
};
use axum::routing::{get, post};
use axum::Router;

pub fn tasks_router() -> Router<AppState> {
    Router::new()
        .route("/", post(create_task).get(list_tasks))
        .route("/{id}", get(get_task).patch(edit_task).delete(delete_task))
}
