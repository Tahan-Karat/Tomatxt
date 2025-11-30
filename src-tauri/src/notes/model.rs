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

    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub children: Vec<Note>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct NotePreview {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub content_preview: String,
    pub content_without_checkboxes: String,
    pub child_count: u32,
    pub is_task: bool,
    pub is_done: bool,
    pub pomodoro_count: u32,
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
            parent_id: None,
            is_task: false,
            is_done: false,
            pomodoro_count: 0,
            children: Vec::new(),
        }
    }

    pub fn to_preview(&self, child_count: u32) -> NotePreview {
        let content_without_checkboxes = self
            .content
            .lines()
            .filter(|line| !line.trim().starts_with("- [") && !line.trim().starts_with("* ["))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        let content_preview = if content_without_checkboxes.len() > 100 {
            format!("{}...", &content_without_checkboxes[..100.min(content_without_checkboxes.len())])
        } else {
            content_without_checkboxes.clone()
        };

        NotePreview {
            id: self.id.clone(),
            parent_id: self.parent_id.clone(),
            title: self.title.clone(),
            content_preview,
            content_without_checkboxes,
            child_count,
            is_task: self.is_task,
            is_done: self.is_done,
            pomodoro_count: self.pomodoro_count,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
