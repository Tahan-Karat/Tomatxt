use std::fs;
use std::path::PathBuf;
use std::env;
use super::model::Note;

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
    
    let content = format!(
        "---\nid: {}\ntitle: {}\ncreated_at: {}\nupdated_at: {}\n---\n\n{}",
        note.id, note.title, note.created_at, note.updated_at, note.content
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
    let mut notes = Vec::new();
    
    let entries = fs::read_dir(notes_dir).map_err(|e| e.to_string())?;
    
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(note) = parse_note_file(&content) {
                    notes.push(note);
                }
            }
        }
    }
    
    notes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    
    Ok(notes)
}

pub fn delete_note(id: &str) -> Result<(), String> {
    let path = get_note_path(id)?;
    
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

fn parse_note_file(content: &str) -> Result<Note, String> {
    let parts: Vec<&str> = content.split("---").collect();
    
    if parts.len() < 3 {
        return Err("Invalid note format".to_string());
    }
    
    let metadata = parts[1];
    let note_content = parts[2..].join("---").trim().to_string();
    
    let mut id = String::new();
    let mut title = String::new();
    let mut created_at = 0u64;
    let mut updated_at = 0u64;
    
    for line in metadata.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("id:") {
            id = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("title:") {
            title = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("created_at:") {
            created_at = value.trim().parse().unwrap_or(0);
        } else if let Some(value) = line.strip_prefix("updated_at:") {
            updated_at = value.trim().parse().unwrap_or(0);
        }
    }
    
    Ok(Note {
        id,
        title,
        content: note_content,
        created_at,
        updated_at,
    })
}
