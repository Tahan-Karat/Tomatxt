// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod notes;

use notes::NotesState;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(NotesState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            notes::commands::create_note,
            notes::commands::get_notes,
            notes::commands::get_note,
            notes::commands::update_note,
            notes::commands::delete_note,
            notes::commands::load_all_notes,
            notes::commands::parse_checkboxes,
            notes::commands::update_note_checkbox_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
