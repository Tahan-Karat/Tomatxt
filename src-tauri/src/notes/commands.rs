use tauri::State;
use super::{NotesState, storage};
use super::model::Note;

#[tauri::command]
pub fn create_note(title: String, content: String, state: State<NotesState>) -> Result<Note, String> {
    let note = Note::new(title, content);
    let mut notes = state.notes.lock().map_err(|e| e.to_string())?;
    notes.push(note.clone());
    
    storage::save_note(&note)?;
    
    Ok(note)
}

#[tauri::command]
pub fn get_notes(state: State<NotesState>) -> Result<Vec<Note>, String> {
    let notes = state.notes.lock().map_err(|e| e.to_string())?;
    Ok(notes.clone())
}

#[tauri::command]
pub fn get_note(id: String, state: State<NotesState>) -> Result<Note, String> {
    let notes = state.notes.lock().map_err(|e| e.to_string())?;
    notes.iter()
        .find(|n| n.id == id)
        .cloned()
        .ok_or("Note not found".to_string())
}

#[tauri::command]
pub fn update_note(id: String, title: String, content: String, state: State<NotesState>) -> Result<Note, String> {
    let mut notes = state.notes.lock().map_err(|e| e.to_string())?;
    let note = notes.iter_mut()
        .find(|n| n.id == id)
        .ok_or("Note not found")?;
    
    note.title = title;
    note.content = content;
    note.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    storage::save_note(&note)?;
    
    Ok(note.clone())
}

#[tauri::command]
pub fn delete_note(id: String, state: State<NotesState>) -> Result<(), String> {
    let mut notes = state.notes.lock().map_err(|e| e.to_string())?;
    notes.retain(|n| n.id != id);
    
    storage::delete_note(&id)?;
    
    Ok(())
}

#[tauri::command]
pub fn load_all_notes(state: State<NotesState>) -> Result<Vec<Note>, String> {
    let loaded_notes = storage::load_all_notes()?;
    let mut notes = state.notes.lock().map_err(|e| e.to_string())?;
    *notes = loaded_notes.clone();
    Ok(loaded_notes)
}
