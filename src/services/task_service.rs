use crate::models::task::{Status, Task};
use std::collections::HashMap;
use uuid::Uuid;

pub struct TaskService {
    tasks: HashMap<Uuid, Task>,
}

impl TaskService {
    pub fn new() -> Self {
        Self {
            tasks: HashMap::new(),
        }
    }

    pub fn insert_new_task(&mut self, task: Task) {
        self.tasks.insert(task.id, task);
    }

    pub fn delete_task(&mut self, task_id: Uuid) {
        self.tasks.remove(&task_id);
    }

    pub fn get_task(&self, task_id: &Uuid) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    pub fn task_count(&self) -> usize {
        self.tasks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

    pub fn list_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn new_service_is_empty() {
        let service = TaskService::new();
        assert!(service.is_empty());
        assert_eq!(service.task_count(), 0);
    }

    #[test]
    fn insert_task_increases_count() {
        let mut service = TaskService::new();
        let task = Task::new("test".into(), None, None);
        service.insert_new_task(task);
        assert_eq!(service.task_count(), 1);
        assert!(!service.is_empty());
    }

    #[test]
    fn get_task_returns_inserted_task() {
        let mut service = TaskService::new();
        let task = Task::new("test".into(), Some("desc".into()), None);
        let id = task.id;
        service.insert_new_task(task);
        let retrieved = service.get_task(&id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "test");
        assert_eq!(retrieved.unwrap().description, Some("desc".into()));
    }

    #[test]
    fn get_task_returns_none_for_unknown_id() {
        let service = TaskService::new();
        assert!(service.get_task(&Uuid::new_v4()).is_none());
    }

    #[test]
    fn insert_multiple_tasks() {
        let mut service = TaskService::new();
        service.insert_new_task(Task::new("a".into(), None, None));
        service.insert_new_task(Task::new("b".into(), None, None));
        service.insert_new_task(Task::new("c".into(), None, None));
        assert_eq!(service.task_count(), 3);
    }

    #[test]
    fn delete_task_removes_it() {
        let mut service = TaskService::new();
        let task = Task::new("test".into(), None, None);
        let id = task.id;
        service.insert_new_task(task);
        assert_eq!(service.task_count(), 1);
        service.delete_task(id);
        assert!(service.is_empty());
        assert!(service.get_task(&id).is_none());
    }

    #[test]
    fn delete_task_only_removes_target() {
        let mut service = TaskService::new();
        let t1 = Task::new("keep".into(), None, None);
        let t2 = Task::new("remove".into(), None, None);
        let id1 = t1.id;
        let id2 = t2.id;
        service.insert_new_task(t1);
        service.insert_new_task(t2);
        service.delete_task(id2);
        assert_eq!(service.task_count(), 1);
        assert!(service.get_task(&id1).is_some());
        assert!(service.get_task(&id2).is_none());
    }

    #[test]
    fn delete_nonexistent_task_is_noop() {
        let mut service = TaskService::new();
        let task = Task::new("test".into(), None, None);
        let id = task.id;
        service.insert_new_task(task);
        service.delete_task(Uuid::new_v4());
        assert_eq!(service.task_count(), 1);
    }

    #[test]
    fn insert_replaces_task_with_same_id() {
        let mut service = TaskService::new();
        let due = Utc::now();
        let t1 = Task::new("original".into(), None, Some(due));
        let id = t1.id;
        let t2 = Task {
            id,
            title: "replacement".into(),
            description: None,
            status: Status::Pending,
            due_date: None,
            created_at: t1.created_at,
            updated_at: t1.updated_at,
        };
        service.insert_new_task(t1);
        assert_eq!(service.task_count(), 1);
        assert_eq!(service.get_task(&id).unwrap().title, "original");
        service.insert_new_task(t2);
        assert_eq!(service.task_count(), 1);
        assert_eq!(service.get_task(&id).unwrap().title, "replacement");
    }
}
