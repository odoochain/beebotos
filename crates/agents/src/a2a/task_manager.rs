use std::collections::HashMap;

use tokio::sync::RwLock;
use uuid::Uuid;

use crate::a2a::message::A2AMessage;
use crate::a2a::protocol::TaskStatus;

pub struct TaskManager {
    tasks: RwLock<HashMap<String, A2ATask>>,
}

/// A2A protocol task
#[derive(Debug, Clone)]
pub struct A2ATask {
    pub id: String,
    pub parent_id: Option<String>,
    pub status: TaskStatus,
    pub messages: Vec<A2AMessage>,
    pub artifacts: Vec<Artifact>,
    pub history: Vec<HistoryEntry>,
}

/// Type alias for backward compatibility
pub type Task = A2ATask;

#[derive(Debug, Clone)]
pub struct Artifact {
    pub id: String,
    pub artifact_type: String,
    pub content: String,
    pub mime_type: String,
}

#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub timestamp: u64,
    pub state: TaskState,
    pub message: String,
}

#[derive(Debug, Clone)]
pub enum TaskState {
    Created,
    Working,
    InputRequired,
    Completed,
    Canceled,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_task(&self, parent_id: Option<String>) -> String {
        let id = Uuid::new_v4().to_string();
        let task = Task {
            id: id.clone(),
            parent_id,
            status: TaskStatus::Created,
            messages: vec![],
            artifacts: vec![],
            history: vec![HistoryEntry {
                timestamp: chrono::Utc::now().timestamp() as u64,
                state: TaskState::Created,
                message: "Task created".to_string(),
            }],
        };
        self.tasks.write().await.insert(id.clone(), task);
        id
    }

    pub async fn update_status(&self, task_id: &str, status: TaskStatus) {
        if let Some(task) = self.tasks.write().await.get_mut(task_id) {
            task.status = status;
        }
    }

    pub async fn add_artifact(&self, task_id: &str, artifact: Artifact) {
        if let Some(task) = self.tasks.write().await.get_mut(task_id) {
            task.artifacts.push(artifact);
        }
    }

    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        self.tasks.read().await.get(task_id).cloned()
    }
}
