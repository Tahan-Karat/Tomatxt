pub mod commands;
pub mod model;
pub mod storage;

use model::Note;
use std::sync::Mutex;

pub struct NotesState {
    pub notes: Mutex<Vec<Note>>,
}

impl NotesState {
    pub fn new() -> Self {
        Self {
            notes: Mutex::new(Vec::new()),
        }
    }
}
