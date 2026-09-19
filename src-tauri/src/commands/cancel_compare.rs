use std::sync::atomic::Ordering;

use tauri::State;

use crate::AppState;

#[tauri::command]
pub fn cancel_compare(state: State<'_, AppState>) {
    state.cancel.store(true, Ordering::Relaxed);
}
