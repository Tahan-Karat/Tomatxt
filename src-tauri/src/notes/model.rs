use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub content: String,

    pub is_task: bool,
    pub is_done: bool,
    pub pomodoro_count: u32,

    pub created_at: u64,
    pub updated_at: u64,
}
pub struct SubTask {
    pub text: String,
    pub completed: bool,
}
impl Note {
    pub fn new(title: String, content: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let id = format!(
            "{}-{}",
            now,
            title.chars().take(8).collect::<String>().replace(' ', "-")
        );

        Self {
            id,
            title,
            content,
            created_at: now,
            updated_at: now,
            parent_id: None,
            is_task: false,
            is_done: false,
            pomodoro_count: 0,
        }
    }
}
