// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod notes;
mod pomodoro;

use notes::NotesState;
use pomodoro::TimerState;
use pomodoro::commands::{PomodoroState, to_seconds};
use std::sync::Mutex;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
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
            })
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            notes::commands::create_note,
            notes::commands::get_notes,
            notes::commands::get_note,
            notes::commands::update_note,
            notes::commands::delete_note,
            notes::commands::load_all_notes,
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
