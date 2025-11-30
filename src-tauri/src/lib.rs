// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod notes;
mod pomodoro;

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
            notes::commands::load_all_notes
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
// src/lib.rs atau di bawah #[cfg(test)]
mod tests {
    // import fungsi dan struct dari pomodoro::commands
    use crate::pomodoro::commands::{
        PomodoroState,
        create_break_time,
        start_break_timer,
        handle_timer_tick,
        handle_pause_timer,
        handle_resume_timer,
    };

    #[test]
    fn test_create_break_time() {
        assert_eq!(create_break_time(5), 300); // 5 menit = 300 detik
        assert_eq!(create_break_time(0), 0);   // edge case
    }

    #[test]
    fn test_start_break_timer() {
        let state = PomodoroState {
            work_duration: 25,
            break_duration: 5,
            remaining: 0,
            is_break: false,
            is_paused: false,
        };

        let new_state = start_break_timer(&state);
        assert_eq!(new_state.is_break, true);
        assert_eq!(new_state.remaining, 300);
        assert_eq!(new_state.is_paused, false);
        // memastikan fungsi tidak mengubah durasi kerja
        assert_eq!(new_state.work_duration, state.work_duration);
    }

    #[test]
    fn test_handle_timer_tick() {
        let state = PomodoroState {
            work_duration: 25,
            break_duration: 5,
            remaining: 10,
            is_break: false,
            is_paused: false,
        };

        let new_state = handle_timer_tick(state.clone());
        assert_eq!(new_state.remaining, 9);

        let finished_state = PomodoroState {
            remaining: 0,
            ..state.clone()
        };
        let result = handle_timer_tick(finished_state.clone());
        assert_eq!(result.remaining, 0); // tetap 0 jika timer habis
    }

    #[test]
    fn test_handle_pause_resume_timer() {
        let state = PomodoroState {
            work_duration: 25,
            break_duration: 5,
            remaining: 10,
            is_break: false,
            is_paused: false,
        };

        let paused = handle_pause_timer(state.clone());
        assert!(paused.is_paused);

        let resumed = handle_resume_timer(paused);
        assert!(!resumed.is_paused);
    }

    #[test]
    fn test_full_cycle() {
        // simulasi start break -> tick -> pause -> resume
        let mut state = PomodoroState {
            work_duration: 25,
            break_duration: 5,
            remaining: 0,
            is_break: false,
            is_paused: false,
        };

        state = start_break_timer(&state);
        assert_eq!(state.remaining, 300);
        assert!(state.is_break);

        state = handle_timer_tick(state);
        assert_eq!(state.remaining, 299);

        state = handle_pause_timer(state);
        assert!(state.is_paused);

        state = handle_resume_timer(state);
        assert!(!state.is_paused);
    }
}
