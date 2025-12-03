use super::model::Note;
use rayon::prelude::*;
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
    get_notes_dir().map(|dir| dir.join(format!("{}.md", id)))
}

fn extract_field(metadata: &str, key: &str) -> Option<String> {
    metadata
        .lines()
        .find(|line| line.trim().starts_with(key))
        .and_then(|line| line.split_once(':'))
        .map(|(_, val)| val.trim().to_string())
}

fn build_frontmatter(note: &Note) -> String {
    vec![
        format!("id: {}", note.id),
        format!("is_task: {}", note.is_task),
        format!("is_done: {}", note.is_done),
        format!("pomodoro_count: {}", note.pomodoro_count),
        format!("title: {}", note.title),
        format!("created_at: {}", note.created_at),
        format!("updated_at: {}", note.updated_at),
    ]
    .join("\n")
}

fn serialize_child_note(child: &Note, indent: usize) -> String {
    let prefix = " ".repeat(indent);
    let indented_content = if child.content.is_empty() {
        String::new()
    } else {
        format!(
            "\n{}{}",
            prefix,
            child.content.replace('\n', &format!("\n{}", prefix))
        )
    };

    let nested_children = serialize_children(&child.children, indent + 2);
    let nested_section = if nested_children.is_empty() {
        String::new()
    } else {
        format!("\n{}", nested_children)
    };

    format!(
        "{}---\n{}id: {}\n{}is_task: {}\n{}is_done: {}\n{}pomodoro_count: {}\n{}title: {}\n{}created_at: {}\n{}updated_at: {}\n{}---{}{}",
        prefix, prefix, child.id, prefix, child.is_task, prefix, child.is_done, prefix,
        child.pomodoro_count, prefix, child.title, prefix, child.created_at, prefix,
        child.updated_at, prefix, indented_content, nested_section
    )
}

fn serialize_children(children: &[Note], indent: usize) -> String {
    children
        .iter()
        .map(|child| serialize_child_note(child, indent))
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_note_content(note: &Note) -> String {
    let children_section = if note.children.is_empty() {
        String::new()
    } else {
        format!("\n\n{}", serialize_children(&note.children, 0))
    };

    format!(
        "---\n{}\n---\n\n{}{}",
        build_frontmatter(note),
        note.content,
        children_section
    )
}

fn split_metadata_and_content(content: &str) -> Result<(&str, &str), String> {
    let parts: Vec<&str> = content.splitn(3, "---").collect();
    match parts.as_slice() {
        [_, metadata, rest] => Ok((metadata, rest)),
        _ => Err("Invalid note format".to_string()),
    }
}

fn extract_fields(metadata: &str) -> Result<Note, String> {
    let get_field = |key: &str| extract_field(metadata, key);

    Ok(Note {
        id: get_field("id").ok_or("Missing ID")?,
        parent_id: get_field("parent_id"),
        title: get_field("title").unwrap_or_default(),
        content: String::new(),
        content_without_checkboxes: None,
        is_task: get_field("is_task")
            .and_then(|v| v.parse().ok())
            .unwrap_or(false),
        is_done: get_field("is_done")
            .and_then(|v| v.parse().ok())
            .unwrap_or(false),
        pomodoro_count: get_field("pomodoro_count")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        created_at: get_field("created_at")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        updated_at: get_field("updated_at")
            .and_then(|v| v.parse().ok())
            .unwrap_or(0),
        children: Vec::new(),
    })
}

fn find_nested_section(content: &str) -> (String, Option<String>) {
    if let Some(pos) = content.find("\n---\n") {
        let before_dash = &content[..pos];
        let last_newline = before_dash.rfind('\n').unwrap_or(0);
        let line_before = &before_dash[last_newline..];

        if !line_before.chars().all(|c| c.is_whitespace() || c == '\n') {
            (content.trim().to_string(), None)
        } else {
            (
                content[..pos].trim().to_string(),
                Some(content[pos + 1..].to_string()),
            )
        }
    } else {
        (content.trim().to_string(), None)
    }
}

fn parse_nested_notes(children_str: &str, parent_id: &str) -> Result<Vec<Note>, String> {
    fn parse_recursive(content: &str, parent_id: &str) -> Result<Vec<Note>, String> {
        if let Some(pos) = content.find("---\n") {
            let child_section = &content[pos + 4..];

            if let Some(end_pos) = child_section.find("\n---\n") {
                let child_full = &child_section[..end_pos];
                let child_content = format!("---\n{}---\n", child_full);

                match parse_note_file(&child_content, Some(parent_id.to_string())) {
                    Ok(child) => {
                        let remaining = &child_section[end_pos..];
                        let mut result = vec![child];
                        result.extend(parse_recursive(remaining, parent_id)?);
                        Ok(result)
                    }
                    Err(_e) => {
                        // If there's an error parsing this child, continue with remaining
                        parse_recursive(&child_section[end_pos..], parent_id)
                    }
                }
            } else {
                match parse_note_file(
                    &format!("---\n{}---\n", child_section),
                    Some(parent_id.to_string()),
                ) {
                    Ok(child) => Ok(vec![child]),
                    Err(_) => Ok(vec![]),
                }
            }
        } else {
            Ok(vec![])
        }
    }

    parse_recursive(children_str, parent_id)
}

fn compose_note(note: Note, content: String, children: Vec<Note>) -> Note {
    Note {
        content,
        children,
        ..note
    }
}

pub fn parse_note_file(content: &str, parent_id: Option<String>) -> Result<Note, String> {
    let (metadata, rest) = split_metadata_and_content(content)?;
    let mut note = extract_fields(metadata)?;
    note.parent_id = parent_id.or(note.parent_id);

    let (note_content, children_section) = find_nested_section(rest);
    let children = children_section
        .as_ref()
        .and_then(|cs| parse_nested_notes(cs, &note.id).ok())
        .unwrap_or_default();

    Ok(compose_note(note, note_content, children))
}

// I/O operations

pub fn save_note(note: &Note) -> Result<(), String> {
    if note.parent_id.is_some() {
        return Ok(());
    }

    let path = get_note_path(&note.id)?;
    let content = build_note_content(note);
    fs::write(path, content).map_err(|e| e.to_string())
}

fn load_note_files() -> Result<Vec<PathBuf>, String> {
    get_notes_dir().and_then(|notes_dir| {
        fs::read_dir(notes_dir)
            .map_err(|e| e.to_string())
            .map(|dir_entries| {
                dir_entries
                    .filter_map(|entry| entry.ok())
                    .map(|entry| entry.path())
                    .filter(|path| path.extension().and_then(|s| s.to_str()) == Some("md"))
                    .collect()
            })
    })
}

fn read_note_from_path(path: PathBuf) -> Option<Note> {
    fs::read_to_string(path)
        .ok()
        .and_then(|content| parse_note_file(&content, None).ok())
}

pub fn load_all_notes() -> Result<Vec<Note>, String> {
    load_note_files().map(|paths| {
        paths
            .into_par_iter()
            .filter_map(read_note_from_path)
            .collect()
    })
}

pub fn delete_note(id: &str) -> Result<(), String> {
    let path = get_note_path(id)?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}
