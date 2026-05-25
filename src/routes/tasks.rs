use crate::controllers::task_controllers::{SharedService, create_task, list_tasks};
use axum::{
    Router,
    routing::{get, post},
};

pub fn router() -> Router<SharedService> {
    Router::new().route("/", post(create_task).get(list_tasks))
}
