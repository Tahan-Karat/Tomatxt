use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub created_at: u64,
    pub updated_at: u64,
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
        }
    }
}
