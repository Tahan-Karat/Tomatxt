// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod notes;
mod pomodoro;

use notes::{model::Note, NotesState};
use pomodoro::commands::{to_seconds, PomodoroState};
use pomodoro::TimerState;
use std::sync::Mutex;
use tauri::State;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command(rename_all = "snake_case")]
fn create_note(title: String, content: String, state: State<NotesState>) -> Result<Note, String> {
    notes::commands::create_note(title, content, state)
}

#[tauri::command(rename_all = "snake_case")]
fn get_notes(state: State<NotesState>) -> Result<Vec<notes::model::NotePreview>, String> {
    notes::commands::get_notes(state)
}

#[tauri::command(rename_all = "snake_case")]
fn get_note(id: String, state: State<NotesState>) -> Result<Note, String> {
    notes::commands::get_note(id, state)
}

#[tauri::command(rename_all = "snake_case")]
fn update_note(
    id: String,
    title: String,
    content: String,
    state: State<NotesState>,
) -> Result<Note, String> {
    notes::commands::update_note(id, title, content, state)
}

#[tauri::command(rename_all = "snake_case")]
fn delete_note(id: String, state: State<NotesState>) -> Result<(), String> {
    notes::commands::delete_note(id, state)
}

#[tauri::command(rename_all = "snake_case")]
fn load_all_notes(state: State<NotesState>) -> Result<Vec<Note>, String> {
    notes::commands::load_all_notes(state)
}

#[tauri::command(rename_all = "snake_case")]
fn parse_checkboxes(content: String) -> Result<Vec<notes::checkbox_parser::Checkbox>, String> {
    notes::commands::parse_checkboxes(content)
}

#[tauri::command(rename_all = "snake_case")]
fn update_note_checkbox_status(
    note_id: String,
    checkbox_text: String,
    new_status: bool,
    state: State<NotesState>,
) -> Result<Note, String> {
    notes::commands::update_note_checkbox_status(note_id, checkbox_text, new_status, state)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(NotesState::new())
        .manage(TimerState {
            timer: Mutex::new(PomodoroState {
                work_duration: to_seconds(25),
                break_duration: to_seconds(5),
                remaining: to_seconds(25),
                is_break: false,
                is_paused: false,
            }),
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            create_note,
            get_notes,
            get_note,
            update_note,
            delete_note,
            load_all_notes,
            parse_checkboxes,
            update_note_checkbox_status,
            // Pomodoro commands
            pomodoro::commands::init_timer,
            pomodoro::commands::get_timer_state,
            pomodoro::commands::start_work,
            pomodoro::commands::start_break,
            pomodoro::commands::tick_timer,
            pomodoro::commands::pause_timer,
            pomodoro::commands::resume_timer,
            pomodoro::commands::reset_timer,
            pomodoro::commands::is_timer_finished,
            // pomodoro::commands::check_work_finished,
            // pomodoro::commands::check_break_finished,
            pomodoro::commands::update_work_duration,
            pomodoro::commands::update_break_duration,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
