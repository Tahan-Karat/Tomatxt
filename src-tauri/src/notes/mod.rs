pub mod commands;
pub mod model;
pub mod storage;
pub mod checkbox_parser;

use model::Note;
use std::sync::Mutex;

pub struct NotesState {
    pub notes: Mutex<Vec<Note>>,
}

impl NotesState {
    pub fn new() -> Self {
        let notes = storage::load_all_notes().unwrap_or_default();
        Self {
            notes: Mutex::new(notes),
        }
    }
}
