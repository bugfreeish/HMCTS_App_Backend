use crate::controllers::task_controllers::{
    SharedService, create_task, delete_task, edit_task, get_task, list_tasks,
};
use axum::{
    Router,
    routing::{delete, get, post},
};

pub fn router() -> Router<SharedService> {
    Router::new()
        .route("/", post(create_task).get(list_tasks))
        .route("/{id}", get(get_task).patch(edit_task).delete(delete_task))
}
