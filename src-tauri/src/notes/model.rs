use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub parent_id: Option<String>,
    pub title: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_without_checkboxes: Option<String>,

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

fn is_checkbox_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("- [") || trimmed.starts_with("* [")
}

fn remove_checkboxes(content: &str) -> String {
    content
        .lines()
        .filter(|line| !is_checkbox_line(line))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn truncate_content(content: &str, max_length: usize) -> String {
    if content.len() > max_length {
        format!("{}...", &content[..max_length.min(content.len())])
    } else {
        content.to_string()
    }
}

fn create_preview(content_without_checkboxes: String) -> String {
    truncate_content(&content_without_checkboxes, 100)
}

pub fn note_to_preview(note: &Note, child_count: u32) -> NotePreview {
    let content_without_checkboxes = remove_checkboxes(&note.content);
    let content_preview = create_preview(content_without_checkboxes.clone());

    NotePreview {
        id: note.id.clone(),
        parent_id: note.parent_id.clone(),
        title: note.title.clone(),
        content_preview,
        content_without_checkboxes,
        child_count,
        is_task: note.is_task,
        is_done: note.is_done,
        pomodoro_count: note.pomodoro_count,
        created_at: note.created_at,
        updated_at: note.updated_at,
    }
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
            content_without_checkboxes: None,
            created_at: now,
            updated_at: now,
            parent_id: None,
            is_task: false,
            is_done: false,
            pomodoro_count: 0,
            children: Vec::new(),
        }
    }
}
