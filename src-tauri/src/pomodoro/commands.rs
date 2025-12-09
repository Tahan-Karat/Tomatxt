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


fn modify_timer_state<F>(state: &State<TimerState>, modifier: F) -> PomodoroState
where
    F: FnOnce(&PomodoroState) -> PomodoroState,
{
    let guard = state.timer.lock().unwrap();
    let new_state = modifier(&guard);
    drop(guard);
    *state.timer.lock().unwrap() = new_state.clone();
    new_state
}


fn start_work_timer(state: &PomodoroState) -> PomodoroState {
    PomodoroState {
        is_break: false,
        remaining: state.work_duration,
        is_paused: false,
        ..*state
    }
}

fn start_break_timer(state: &PomodoroState) -> PomodoroState {
    PomodoroState {
        is_break: true,
        remaining: state.break_duration,
        is_paused: false,
        ..*state
    }
}

fn tick(state: &PomodoroState) -> PomodoroState {
    if state.is_paused || state.remaining == 0 {
        return state.clone();
    }

    PomodoroState {
        remaining: state.remaining - 1,
        ..*state
    }
}

fn pause(state: &PomodoroState) -> PomodoroState {
    PomodoroState {
        is_paused: true,
        ..*state
    }
}

fn resume(state: &PomodoroState) -> PomodoroState {
    PomodoroState {
        is_paused: false,
        ..*state
    }
}

fn reset(state: &PomodoroState) -> PomodoroState {
    let reset_seconds = if state.is_break {
        state.break_duration
    } else {
        state.work_duration
    };

    PomodoroState {
        remaining: reset_seconds,
        is_paused: true,
        ..*state
    }
}

fn is_finished(state: &PomodoroState) -> bool {
    state.remaining == 0
}

fn is_break_finished(state: &PomodoroState) -> bool {
    state.is_break && is_finished(state)
}

fn is_work_finished(state: &PomodoroState) -> bool {
    !state.is_break && is_finished(state)
}

fn next_state(state: &PomodoroState) -> PomodoroState {
    if is_work_finished(state) {
        start_break_timer(state)
    } else if is_break_finished(state) {
        start_work_timer(state)
    } else {
        state.clone()
    }
}

#[tauri::command]
pub fn init_timer(work_min: u32, break_min: u32, state: State<TimerState>) -> PomodoroState {
    let new_state = PomodoroState {
        work_duration: to_seconds(work_min),
        break_duration: to_seconds(break_min),
        remaining: to_seconds(work_min),
        is_break: false,
        is_paused: false,
    };

    *state.timer.lock().unwrap() = new_state.clone();
    new_state
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_timer_state(state: State<TimerState>) -> PomodoroState {
    state.timer.lock().unwrap().clone()
}

#[tauri::command(rename_all = "snake_case")]
pub fn tick_timer(state: State<TimerState>) -> PomodoroState {
    modify_timer_state(&state, |timer| next_state(&tick(timer)))
}

#[tauri::command(rename_all = "snake_case")]
pub fn start_work(state: State<TimerState>) -> PomodoroState {
    modify_timer_state(&state, start_work_timer)
}

#[tauri::command(rename_all = "snake_case")]
pub fn start_break(state: State<TimerState>) -> PomodoroState {
    modify_timer_state(&state, start_break_timer)
}

#[tauri::command(rename_all = "snake_case")]
pub fn pause_timer(state: State<TimerState>) -> PomodoroState {
    modify_timer_state(&state, pause)
}

#[tauri::command(rename_all = "snake_case")]
pub fn resume_timer(state: State<TimerState>) -> PomodoroState {
    modify_timer_state(&state, resume)
}

#[tauri::command(rename_all = "snake_case")]
pub fn reset_timer(state: State<TimerState>) -> PomodoroState {
    modify_timer_state(&state, reset)
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_work_duration(state: State<TimerState>, minutes: u32) -> PomodoroState {
    let guard = state.timer.lock().unwrap();
    let new_duration = to_seconds(minutes);
    
    let new_state = PomodoroState {
        work_duration: new_duration,
        remaining: if !guard.is_break {
            new_duration
        } else {
            guard.remaining
        },
        ..*guard
    };
    drop(guard);
    
    *state.timer.lock().unwrap() = new_state.clone();
    new_state
}

#[tauri::command(rename_all = "snake_case")]
pub fn update_break_duration(state: State<TimerState>, minutes: u32) -> PomodoroState {
    let guard = state.timer.lock().unwrap();
    let new_duration = to_seconds(minutes);
    
    let new_state = PomodoroState {
        break_duration: new_duration,
        remaining: if guard.is_break {
            new_duration
        } else {
            guard.remaining
        },
        ..*guard
    };
    drop(guard);
    
    *state.timer.lock().unwrap() = new_state.clone();
    new_state
}

#[tauri::command(rename_all = "snake_case")]
pub fn is_timer_finished(state: State<TimerState>) -> bool {
    is_finished(&state.timer.lock().unwrap())
}