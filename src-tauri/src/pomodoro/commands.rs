use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PomodoroState {
    pub work_duration: u32,
    pub break_duration: u32,
    pub remaining: u32,
    pub is_break: bool,
    pub is_paused: bool,
}

pub struct TimerState {
    pub timer: Mutex<PomodoroState>,
}

pub fn to_seconds(minutes: u32) -> u32 {
    minutes * 60
}

pub fn start_work_timer(state: &PomodoroState) -> PomodoroState {
    PomodoroState {
        is_break: false,
        remaining: state.work_duration,
        is_paused: false,
        ..state.clone()
    }
}

pub fn start_break_timer(state: &PomodoroState) -> PomodoroState {
    PomodoroState {
        is_break: true,
        remaining: state.break_duration,
        is_paused: false,
        ..state.clone()
    }
}

pub fn tick(state: PomodoroState) -> PomodoroState {
    if state.is_paused || state.remaining == 0 {
        return state;
    }
    PomodoroState {
        remaining: state.remaining - 1,
        ..state
    }
}

pub fn pause(state: PomodoroState) -> PomodoroState {
    PomodoroState {
        is_paused: true,
        ..state
    }
}

pub fn resume(state: PomodoroState) -> PomodoroState {
    PomodoroState {
        is_paused: false,
        ..state
    }
}

pub fn reset(state: PomodoroState) -> PomodoroState {
    let reset_seconds = if state.is_break {
        state.break_duration
    } else {
        state.work_duration
    };

    PomodoroState {
        remaining: reset_seconds,
        is_paused: true,

        ..state
    }
}

pub fn is_finished(state: &PomodoroState) -> bool {
    state.remaining == 0
}

// pub fn is_running(state: &PomodoroState) -> bool {
//     !state.is_paused && state.remaining > 0
// }
//
// pub fn is_break_time(state: &PomodoroState) -> bool {
//     state.is_break
// }
//
// pub fn is_work_time(state: &PomodoroState) -> bool {
//     !state.is_break
// }

pub fn is_break_finished(state: &PomodoroState) -> bool {
    state.is_break && is_finished(state)
}

pub fn is_work_finished(state: &PomodoroState) -> bool {
    !state.is_break && is_finished(state)
}

pub fn next_state(state: &PomodoroState) -> PomodoroState {
    if is_work_finished(state) {
        start_break_timer(state)
    } else if is_break_finished(state) {
        start_work_timer(state)
    } else {
        state.clone()
    }
}

// pub fn format_time(seconds: u32) -> String {
//     let minutes = seconds / 60;
//     let secs = seconds % 60;
//     format!("{:02}:{:02}", minutes, secs)
// }

// pub fn timer_info(state: &PomodoroState) -> String {
//     let mode = if state.is_break { "BREAK" } else { "WORK" };
//     let status = if state.is_paused { "PAUSED" } else { "RUNNING" };
//     format!("[{}] {} | {}", mode, status, format_time(state.remaining))
// }

#[tauri::command]
pub fn init_timer(work_min: u32, break_min: u32, state: State<TimerState>) -> PomodoroState {
    let new_state = PomodoroState {
        work_duration: to_seconds(work_min),
        break_duration: to_seconds(break_min),
        remaining: to_seconds(work_min),
        is_break: false,
        is_paused: false,
    };
    let mut timer = state.timer.lock().unwrap();
    *timer = new_state.clone();
    new_state
}

#[tauri::command]
pub fn get_timer_state(state: State<TimerState>) -> PomodoroState {
    state.timer.lock().unwrap().clone()
}

#[tauri::command]
pub fn tick_timer(state: State<TimerState>) -> PomodoroState {
    let mut timer = state.timer.lock().unwrap();
    *timer = tick(timer.clone());
    *timer = next_state(&timer);
    timer.clone()
}

#[tauri::command]
pub fn start_work(state: State<TimerState>) -> PomodoroState {
    let mut timer = state.timer.lock().unwrap();
    *timer = start_work_timer(&timer);
    timer.clone()
}

#[tauri::command]
pub fn start_break(state: State<TimerState>) -> PomodoroState {
    let mut timer = state.timer.lock().unwrap();
    *timer = start_break_timer(&timer);
    timer.clone()
}

#[tauri::command]
pub fn pause_timer(state: State<TimerState>) -> PomodoroState {
    let mut timer = state.timer.lock().unwrap();
    *timer = pause(timer.clone());
    timer.clone()
}

#[tauri::command]
pub fn resume_timer(state: State<TimerState>) -> PomodoroState {
    let mut timer = state.timer.lock().unwrap();
    *timer = resume(timer.clone());
    timer.clone()
}

#[tauri::command]
pub fn reset_timer(state: State<TimerState>) -> PomodoroState {
    let mut timer = state.timer.lock().unwrap();
    *timer = reset(timer.clone());
    timer.clone()
}

#[tauri::command]
pub fn update_work_duration(state: State<TimerState>, minutes: u32) -> PomodoroState {
    let mut timer = state.timer.lock().unwrap();
    let new_duration = to_seconds(minutes);
    *timer = PomodoroState {
        work_duration: new_duration,
        remaining: if !timer.is_break {
            new_duration
        } else {
            timer.remaining
        },
        ..timer.clone()
    };
    timer.clone()
}

#[tauri::command]
pub fn update_break_duration(state: State<TimerState>, minutes: u32) -> PomodoroState {
    let mut timer = state.timer.lock().unwrap();
    let new_duration = to_seconds(minutes);
    *timer = PomodoroState {
        break_duration: new_duration,
        remaining: if timer.is_break {
            new_duration
        } else {
            timer.remaining
        },
        ..timer.clone()
    };
    timer.clone()
}

#[tauri::command]
pub fn is_timer_finished(state: State<TimerState>) -> bool {
    let timer = state.timer.lock().unwrap();
    is_finished(&timer)
}
