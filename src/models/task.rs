use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "pending")]
    Pending,

    #[serde(rename = "in-progress")]
    InProgress,

    #[serde(rename = "completed")]
    Completed,
}

#[derive(Debug)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: Mutex<Status>,
    pub due_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Mutex<DateTime<Utc>>,
}

impl Clone for Task {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            title: self.title.clone(),
            description: self.description.clone(),
            status: Mutex::new(self.status.lock().unwrap().clone()),
            due_date: self.due_date,
            created_at: self.created_at,
            updated_at: Mutex::new(self.updated_at.lock().unwrap().clone()),
        }
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.title == other.title
            && self.description == other.description
            && *self.status.lock().unwrap() == *other.status.lock().unwrap()
            && self.due_date == other.due_date
            && self.created_at == other.created_at
            && *self.updated_at.lock().unwrap() == *other.updated_at.lock().unwrap()
    }
}

impl Task {
    pub fn new(
        title: String,
        description: Option<String>,
        due_date: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            description,
            status: Mutex::new(Status::Pending),
            due_date,
            created_at: Utc::now(),
            updated_at: Mutex::new(Utc::now()),
        }
    }

    pub fn set_status(&self, status: Status) {
        *self.status.lock().unwrap() = status;
        self.set_updated_at();
    }

    pub fn set_due_date(&mut self, due_date: Option<DateTime<Utc>>) {
        self.due_date = due_date
    }

    pub fn set_updated_at(&self) {
        *self.updated_at.lock().unwrap() = Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn new_task_has_pending_status() {
        let task = Task::new("test".into(), None, None);
        assert_eq!(*task.status.lock().unwrap(), Status::Pending);
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
        assert!(*task.updated_at.lock().unwrap() > before);
        assert!(*task.updated_at.lock().unwrap() < after);
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
    fn set_status_pending() {
        let task = Task::new("test".into(), None, None);
        task.set_status(Status::Pending);
        assert_eq!(*task.status.lock().unwrap(), Status::Pending);
    }

    #[test]
    fn set_status_in_progress() {
        let task = Task::new("test".into(), None, None);
        task.set_status(Status::InProgress);
        assert_eq!(*task.status.lock().unwrap(), Status::InProgress);
    }

    #[test]
    fn set_status_completed() {
        let task = Task::new("test".into(), None, None);
        task.set_status(Status::Completed);
        assert_eq!(*task.status.lock().unwrap(), Status::Completed);
    }

    #[test]
    fn set_status_transitions() {
        let task = Task::new("test".into(), None, None);
        task.set_status(Status::InProgress);
        assert_eq!(*task.status.lock().unwrap(), Status::InProgress);
        task.set_status(Status::Completed);
        assert_eq!(*task.status.lock().unwrap(), Status::Completed);
        task.set_status(Status::Pending);
        assert_eq!(*task.status.lock().unwrap(), Status::Pending);
    }

    #[test]
    fn set_due_date_overwrites() {
        let mut task = Task::new("test".into(), None, Some(Utc::now()));
        let new_due = Utc::now() + Duration::days(1);
        task.set_due_date(Some(new_due));
        assert_eq!(task.due_date, Some(new_due));
    }

    #[test]
    fn set_due_date_clears() {
        let mut task = Task::new("test".into(), None, Some(Utc::now()));
        task.set_due_date(None);
        assert_eq!(task.due_date, None);
    }

    #[test]
    fn set_due_date_from_none_to_some() {
        let mut task = Task::new("test".into(), None, None);
        let due = Utc::now();
        task.set_due_date(Some(due));
        assert_eq!(task.due_date, Some(due));
    }

    #[test]
    fn set_updated_at_changes_timestamp() {
        let task = Task::new("test".into(), None, None);
        let original = *task.updated_at.lock().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
        task.set_updated_at();
        assert!(*task.updated_at.lock().unwrap() > original);
    }

    #[test]
    fn tasks_with_same_data_are_equal() {
        let due = Utc::now();
        let t1 = Task::new("same".into(), Some("data".into()), Some(due));
        let t2 = Task {
            id: t1.id,
            title: "same".into(),
            description: Some("data".into()),
            status: Mutex::new(t1.status.lock().unwrap().clone()),
            due_date: Some(due),
            created_at: t1.created_at,
            updated_at: Mutex::new(t1.updated_at.lock().unwrap().clone()),
        };
        assert_eq!(t1, t2);
    }
}
