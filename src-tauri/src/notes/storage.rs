use super::model::Note;
use std::env;
use std::fs;
use std::path::PathBuf;

fn get_notes_dir() -> Result<PathBuf, String> {
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| "Could not find home directory")?;

    let notes_dir = PathBuf::from(home).join(".tomatxt").join("notes");

    if !notes_dir.exists() {
        fs::create_dir_all(&notes_dir).map_err(|e| e.to_string())?;
    }

    Ok(notes_dir)
}

fn get_note_path(id: &str) -> Result<PathBuf, String> {
    let notes_dir = get_notes_dir()?;
    Ok(notes_dir.join(format!("{}.md", id)))
}

pub fn save_note(note: &Note) -> Result<(), String> {
    let path = get_note_path(&note.id)?;

    let parent_line = match &note.parent_id {
        Some(pid) => format!("parent_id: {}\n", pid),
        None => String::new(),
    };

    let content = format!(
        "---\nid: {}\n{}is_task: {}\nis_done: {}\npomodoro_count: {}\ntitle: {}\ncreated_at: {}\nupdated_at: {}\n---\n\n{}",
        note.id,
        parent_line,
        note.is_task,
        note.is_done,
        note.pomodoro_count,
        note.title,
        note.created_at,
        note.updated_at,
        note.content
    );

    fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn load_note(id: &str) -> Result<Note, String> {
    let path = get_note_path(id)?;
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;

    parse_note_file(&content)
}

pub fn load_all_notes() -> Result<Vec<Note>, String> {
    let notes_dir = get_notes_dir()?;

    let notes: Vec<Note> = fs::read_dir(notes_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("md"))
        .filter_map(|path| fs::read_to_string(path).ok())
        .filter_map(|content| parse_note_file(&content).ok())
        .collect();

    Ok(notes)
}
pub fn get_child_note(parent_id: &str) -> Result<Vec<Note>, String> {
    let all_notes = load_all_notes()?;

    let children: Vec<Note> = all_notes
        .into_iter()
        .filter(|n| n.parent_id.as_deref() == Some(parent_id))
        .collect();

    Ok(children)
}

pub fn delete_note(id: &str) -> Result<(), String> {
    let path = get_note_path(id)?;

    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn parse_note_file(content: &str) -> Result<Note, String> {
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    match parts.as_slice() {
        [_, metadata, note_content] => {
            let find_field = |key: &str| -> Option<String> {
                metadata
                    .lines()
                    .find(|line| line.trim().starts_with(key))
                    .and_then(|line| line.split_once(':'))
                    .map(|(_, val)| val.trim().to_string())
            };
            Ok(Note {
                id: find_field("id").ok_or("Missing ID")?,
                title: find_field("title").unwrap_or_default(),
                content: note_content.trim().to_string(),
                created_at: find_field("created_at")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                updated_at: find_field("updated_at")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                parent_id: find_field("parent_id"),

                is_task: find_field("is_task")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(false),
                is_done: find_field("is_done")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(false),
                pomodoro_count: find_field("pomodoro_count")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
            })
        }
        _ => Err("Invalid note format".to_string()),
    }
}
