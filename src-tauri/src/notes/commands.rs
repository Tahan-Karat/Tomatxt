use super::model::{Note, NotePreview};
use super::{checkbox_parser, storage, NotesState};
use std::sync::MutexGuard;
use tauri::State;

/// Functional utility to safely access and transform state
fn with_notes_state<T, F>(state: &State<NotesState>, transformer: F) -> Result<T, String>
where
    F: FnOnce(MutexGuard<Vec<Note>>) -> Result<T, String>,
{
    let notes = state.notes.lock().map_err(|e| e.to_string())?;
    transformer(notes)
}

/// Functional utility to modify state with proper locking
fn modify_notes_state<T, F>(state: &State<NotesState>, modifier: F) -> Result<T, String>
where
    F: FnOnce(&mut Vec<Note>) -> Result<T, String>,
{
    let mut notes = state.notes.lock().map_err(|e| e.to_string())?;
    modifier(&mut notes)
}

/// Find a note by ID functionally
fn find_note<'a>(notes: &'a [Note], id: &str) -> Option<&'a Note> {
    notes.iter().find(|note| note.id == id)
}

/// Remove a note from a list functionally
fn remove_note_from_list(notes: Vec<Note>, id: &str) -> Vec<Note> {
    notes.into_iter().filter(|note| note.id != id).collect()
}

/// Add a note to a list functionally
fn add_note_to_list(notes: Vec<Note>, note: Note) -> Vec<Note> {
    [notes, vec![note]].concat()
}

/// Immutably update note timestamp using struct update syntax
fn update_timestamp(note: Note) -> Note {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    Note {
        updated_at: now,
        ..note
    }
}

/// Immutably find and update a note using fold
fn find_and_update_note<F>(
    notes: Vec<Note>,
    id: &str,
    updater: F,
) -> Result<(Vec<Note>, Note), String>
where
    F: Fn(Note) -> Note,
{
    let (new_notes, updated_note): (Vec<Note>, Option<Note>) =
        notes
            .into_iter()
            .fold((Vec::new(), None), |(mut acc, found), note| {
                if note.id == id && found.is_none() {
                    let updated = updater(note);
                    acc.push(updated.clone());
                    (acc, Some(updated))
                } else {
                    acc.push(note);
                    (acc, found)
                }
            });

    updated_note
        .ok_or("Note not found".to_string())
        .map(|updated| (new_notes, updated))
}

#[tauri::command(rename_all = "snake_case")]
pub fn create_note(
    title: String,
    content: String,
    state: State<NotesState>,
) -> Result<Note, String> {
    let note = Note::new(title, content);
    storage::save_note(&note)?;

    modify_notes_state(&state, |notes| {
        *notes = add_note_to_list(std::mem::take(notes), note.clone());
        Ok(())
    })?;

    Ok(note)
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_notes(state: State<NotesState>) -> Result<Vec<NotePreview>, String> {
    with_notes_state(&state, |notes_guard| {
        Ok(notes_guard.iter().map(|note| note.to_preview(0)).collect())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_note(id: String, state: State<NotesState>) -> Result<Note, String> {
    with_notes_state(&state, |notes_guard| {
        find_note(&*notes_guard, &id)
            .cloned()
            .map(|mut note| {
                note.content_without_checkboxes = Some(
                    note.content
                        .lines()
                        .filter(|line| {
                            !line.trim().starts_with("- [") && !line.trim().starts_with("* [")
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                        .trim()
                        .to_string(),
                );
                note
            })
            .ok_or_else(|| "Note not found".to_string())
    })
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_note(
    id: String,
    title: String,
    content: String,
    state: State<NotesState>,
) -> Result<Note, String> {
    let notes = with_notes_state(&state, |notes_guard| Ok(notes_guard.clone()))?;

    let (new_notes, mut updated) = find_and_update_note(notes, &id, |note| {
        update_timestamp(Note {
            title: title.clone(),
            content: content.clone(),
            ..note
        })
    })?;

    // Add content_without_checkboxes
    updated.content_without_checkboxes = Some(
        updated
            .content
            .lines()
            .filter(|line| !line.trim().starts_with("- [") && !line.trim().starts_with("* ["))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string(),
    );

    // Save to storage
    storage::save_note(&updated)?;

    // Update in-memory state
    modify_notes_state(&state, |state_notes| {
        *state_notes = new_notes;
        Ok(())
    })?;

    Ok(updated)
}

#[tauri::command(rename_all = "snake_case")]
pub fn delete_note(id: String, state: State<NotesState>) -> Result<(), String> {
    // Delete from storage first
    storage::delete_note(&id)?;

    // Update in-memory state functionally
    modify_notes_state(&state, |notes| {
        *notes = remove_note_from_list(std::mem::take(notes), &id);
        Ok(())
    })?;

    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn load_all_notes(state: State<NotesState>) -> Result<Vec<Note>, String> {
    let loaded_notes = storage::load_all_notes()?;

    modify_notes_state(&state, |notes| {
        *notes = loaded_notes.clone();
        Ok(())
    })?;

    Ok(loaded_notes)
}

#[tauri::command(rename_all = "snake_case")]
pub fn parse_checkboxes(content: String) -> Result<Vec<checkbox_parser::Checkbox>, String> {
    Ok(checkbox_parser::parse_checkboxes(&content))
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_note_checkbox_status(
    note_id: String,
    checkbox_text: String,
    new_status: bool,
    state: State<NotesState>,
) -> Result<Note, String> {
    let notes = with_notes_state(&state, |notes_guard| Ok(notes_guard.clone()))?;

    let (new_notes, updated) = find_and_update_note(notes, &note_id, |note| {
        update_timestamp(Note {
            content: checkbox_parser::update_checkbox_in_content(
                &note.content,
                &checkbox_text,
                new_status,
            ),
            ..note
        })
    })?;

    // Save to storage
    storage::save_note(&updated)?;

    // Update in-memory state
    modify_notes_state(&state, |state_notes| {
        *state_notes = new_notes;
        Ok(())
    })?;

    Ok(updated)
}
