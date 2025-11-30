use tauri::State;

#[derive(Clone)] 
pub struct PomodoroState {
    pub work_duration: u32,
    pub break_duration: u32,
    pub remaining: u32,
    pub is_break: bool,
    pub is_paused: bool,
}

pub fn to_second(minutes:u32) -> u32 {
    minutes * 60;
}

pub fn start_break_timer(state: &PomodoroState) -> PomodoroState {
    PomodoroState {
        is_break: true,                                   
        remaining: create_break_time(state.break_duration), 
        is_paused: false,
        ..state                             
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

pub fn handle_pause_timer(state: PomodoroState) -> PomodoroState {
    PomodoroState {
        is_paused: true,
        ..state
    }
}

pub fn handle_resume_timer(state: PomodoroState) -> PomodoroState {
    PomodoroState {
        is_paused: false,
        ..state
    }
}

pub fn reset(state: PomodoroState) -> PomodoroState {
    let reset_seconds = if state.is_break {
        to_seconds(state.break_duration)
    } else {
        to_seconds(state.work_duration)
    };
    
    PomodoroState {
        remaining: reset_seconds,
        is_paused: false,
        ..state
    }
}

pub fn is_finished(state: &PomodoroState) -> bool {
    state.remaining == 0
}

/// Pure: Apakah timer sedang berjalan?
pub fn is_running(state: &PomodoroState) -> bool {
    !state.is_paused && state.remaining > 0
}

/// Pure: Apakah sedang break time?
pub fn is_break_time(state: &PomodoroState) -> bool {
    state.is_break
}

/// Pure: Apakah sedang work time?
pub fn is_work_time(state: &PomodoroState) -> bool {
    !state.is_break
}

/// Pure: Apakah break time selesai? (perlu switch ke work)
pub fn is_break_finished(state: &PomodoroState) -> bool {
    state.is_break && is_finished(state)
}

/// Pure: Apakah work time selesai? (perlu switch ke break)
pub fn is_work_finished(state: &PomodoroState) -> bool {
    !state.is_break && is_finished(state)
}

pub fn format_time(seconds: u32) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;
    
    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, secs)
    } else {
        format!("{:02}:{:02}", minutes, secs)
    }
}

/// Pure: Get timer info sebagai string
pub fn timer_info(state: &PomodoroState) -> String {
    let mode = if state.is_break { "BREAK" } else { "WORK" };
    let status = if state.is_paused { "PAUSED" } else { "RUNNING" };
    
    format!(
        "[{}] {} | {}",
        mode,
        status,
        format_time(state.remaining_seconds)
    )
}
