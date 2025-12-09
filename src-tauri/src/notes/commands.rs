use super::model::{note_to_preview, Note, NotePreview};
use super::{checkbox_parser, storage, NotesState};
use std::sync::MutexGuard;
use tauri::State;

fn with_notes_state<T, F>(state: &State<NotesState>, transformer: F) -> Result<T, String>
where
    F: FnOnce(MutexGuard<Vec<Note>>) -> Result<T, String>,
{
    state
        .notes
        .lock()
        .map_err(|e| e.to_string())
        .and_then(transformer)
}

fn modify_notes_state<T, F>(state: &State<NotesState>, modifier: F) -> Result<T, String>
where
    F: FnOnce(&mut Vec<Note>) -> Result<T, String>,
{
    state
        .notes
        .lock()
        .map_err(|e| e.to_string())
        .and_then(|mut notes| modifier(&mut notes))
}

fn find_note<'a>(notes: &'a [Note], id: &str) -> Option<&'a Note> {
    notes.iter().find(|note| note.id == id)
}

fn remove_note_from_list(notes: Vec<Note>, id: &str) -> Vec<Note> {
    notes.into_iter().filter(|note| note.id != id).collect()
}

fn add_note_to_list(notes: Vec<Note>, note: Note) -> Vec<Note> {
    [notes, vec![note]].concat()
}

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

pub(crate) fn create_note(
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

pub(crate) fn get_notes(state: State<NotesState>) -> Result<Vec<NotePreview>, String> {
    with_notes_state(&state, |notes_guard| {
        Ok(notes_guard.iter().map(|note| note_to_preview(note, 0)).collect())
    })
}

pub(crate) fn get_note(id: String, state: State<NotesState>) -> Result<Note, String> {
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

pub(crate) fn update_note(
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

pub(crate) fn delete_note(id: String, state: State<NotesState>) -> Result<(), String> {
    // Delete from storage first
    storage::delete_note(&id)?;

    // Update in-memory state functionally
    modify_notes_state(&state, |notes| {
        *notes = remove_note_from_list(std::mem::take(notes), &id);
        Ok(())
    })?;

    Ok(())
}

pub(crate) fn load_all_notes(state: State<NotesState>) -> Result<Vec<Note>, String> {
    let loaded_notes = storage::load_all_notes()?;

    modify_notes_state(&state, |notes| {
        *notes = loaded_notes.clone();
        Ok(())
    })?;

    Ok(loaded_notes)
}

pub(crate) fn parse_checkboxes(content: String) -> Result<Vec<checkbox_parser::Checkbox>, String> {
    Ok(checkbox_parser::parse_checkboxes(&content))
}

pub(crate) fn update_note_checkbox_status(
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
