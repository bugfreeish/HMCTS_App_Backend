use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Type)]
#[sqlx(type_name = "text")]
pub enum Status {
    #[serde(rename = "pending")]
    #[sqlx(rename = "pending")]
    Pending,

    #[serde(rename = "in-progress")]
    #[sqlx(rename = "in-progress")]
    InProgress,

    #[serde(rename = "completed")]
    #[sqlx(rename = "completed")]
    Completed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, sqlx::FromRow)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: Status,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Task {
    pub fn new(
        title: String,
        description: Option<String>,
        due_date: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: Status::Pending,
            due_date,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn new_task_has_pending_status() {
        let task = Task::new("test".into(), None, None);
        assert_eq!(task.status, Status::Pending);
    }

    #[test]
    fn new_task_generates_id() {
        let task = Task::new("test".into(), None, None);
        assert_ne!(task.id, Uuid::nil());
    }

    #[test]
    fn new_task_sets_created_at() {
        let before = Utc::now() - Duration::milliseconds(1);
        let task = Task::new("test".into(), None, None);
        let after = Utc::now() + Duration::milliseconds(1);
        assert!(task.created_at > before);
        assert!(task.created_at < after);
    }

    #[test]
    fn new_task_sets_updated_at() {
        let before = Utc::now() - Duration::milliseconds(1);
        let task = Task::new("test".into(), None, None);
        let after = Utc::now() + Duration::milliseconds(1);
        assert!(task.updated_at > before);
        assert!(task.updated_at < after);
    }

    #[test]
    fn new_task_sets_title() {
        let task = Task::new("my title".into(), None, None);
        assert_eq!(task.title, "my title");
    }

    #[test]
    fn new_task_sets_description() {
        let task = Task::new("test".into(), Some("desc".into()), None);
        assert_eq!(task.description, Some("desc".into()));
    }

    #[test]
    fn new_task_description_is_none_when_omitted() {
        let task = Task::new("test".into(), None, None);
        assert_eq!(task.description, None);
    }

    #[test]
    fn new_task_sets_due_date() {
        let due = Utc::now();
        let task = Task::new("test".into(), None, Some(due));
        assert_eq!(task.due_date, Some(due));
    }

    #[test]
    fn new_task_due_date_is_none_when_omitted() {
        let task = Task::new("test".into(), None, None);
        assert_eq!(task.due_date, None);
    }

    #[test]
    fn new_task_empty_title() {
        let task = Task::new(String::new(), None, None);
        assert_eq!(task.title, "");
    }

    #[test]
    fn tasks_with_same_data_are_equal() {
        let due = Utc::now();
        let t1 = Task::new("same".into(), Some("data".into()), Some(due));
        let t2 = Task {
            id: t1.id,
            title: "same".into(),
            description: Some("data".into()),
            status: Status::Pending,
            due_date: Some(due),
            created_at: t1.created_at,
            updated_at: t1.updated_at,
        };
        assert_eq!(t1, t2);
    }
}
