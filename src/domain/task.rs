use chrono::{DateTime, Utc};
use serde::de;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::Display;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    EmptyTitle,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyTitle => write!(f, "Title must not be empty"),
        }
    }
}

impl std::error::Error for ValidationError {}

#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Pending,
    InProgress,
    Completed,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Pending => write!(f, "pending"),
            Status::InProgress => write!(f, "in-progress"),
            Status::Completed => write!(f, "completed"),
        }
    }
}

impl FromStr for Status {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Status::Pending),
            "in-progress" => Ok(Status::InProgress),
            "completed" => Ok(Status::Completed),
            _ => Err(format!("Invalid status: {s}")),
        }
    }
}

impl Serialize for Status {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Status {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Status::from_str(&s).map_err(de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    ) -> Result<Self, ValidationError> {
        if title.trim().is_empty() {
            return Err(ValidationError::EmptyTitle);
        }
        let now = Utc::now();
        Ok(Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: Status::Pending,
            due_date,
            created_at: now,
            updated_at: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn make_task() -> Task {
        Task::new("test".into(), None, None).unwrap()
    }

    #[test]
    fn new_task_has_pending_status() {
        let task = make_task();
        assert_eq!(task.status, Status::Pending);
    }

    #[test]
    fn new_task_generates_id() {
        let task = make_task();
        assert_ne!(task.id, Uuid::nil());
    }

    #[test]
    fn new_task_sets_created_at() {
        let before = Utc::now() - Duration::milliseconds(1);
        let task = make_task();
        let after = Utc::now() + Duration::milliseconds(1);
        assert!(task.created_at > before);
        assert!(task.created_at < after);
    }

    #[test]
    fn new_task_sets_updated_at() {
        let before = Utc::now() - Duration::milliseconds(1);
        let task = make_task();
        let after = Utc::now() + Duration::milliseconds(1);
        assert!(task.updated_at > before);
        assert!(task.updated_at < after);
    }

    #[test]
    fn new_task_sets_title() {
        let task = Task::new("my title".into(), None, None).unwrap();
        assert_eq!(task.title, "my title");
    }

    #[test]
    fn new_task_sets_description() {
        let task = Task::new("test".into(), Some("desc".into()), None).unwrap();
        assert_eq!(task.description, Some("desc".into()));
    }

    #[test]
    fn new_task_description_is_none_when_omitted() {
        let task = make_task();
        assert_eq!(task.description, None);
    }

    #[test]
    fn new_task_sets_due_date() {
        let due = Utc::now();
        let task = Task::new("test".into(), None, Some(due)).unwrap();
        assert_eq!(task.due_date, Some(due));
    }

    #[test]
    fn new_task_due_date_is_none_when_omitted() {
        let task = make_task();
        assert_eq!(task.due_date, None);
    }

    #[test]
    fn new_task_rejects_empty_title() {
        let result = Task::new(String::new(), None, None);
        assert_eq!(result, Err(ValidationError::EmptyTitle));
    }

    #[test]
    fn new_task_rejects_whitespace_title() {
        let result = Task::new("   ".into(), None, None);
        assert_eq!(result, Err(ValidationError::EmptyTitle));
    }

    #[test]
    fn tasks_with_same_data_are_equal() {
        let due = Utc::now();
        let t1 = Task::new("same".into(), Some("data".into()), Some(due)).unwrap();
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

    #[test]
    fn status_display() {
        assert_eq!(Status::Pending.to_string(), "pending");
        assert_eq!(Status::InProgress.to_string(), "in-progress");
        assert_eq!(Status::Completed.to_string(), "completed");
    }
    #[test]
    fn status_from_str() {
        assert_eq!("pending".parse::<Status>().unwrap(), Status::Pending);
        assert_eq!("in-progress".parse::<Status>().unwrap(), Status::InProgress);
        assert_eq!("completed".parse::<Status>().unwrap(), Status::Completed);
        assert!("invalid".parse::<Status>().is_err());
    }
}
