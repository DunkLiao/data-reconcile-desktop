use std::sync::atomic::Ordering;

use tauri::{Emitter, State};

use crate::compare::key_comparator::compare_key_based;
use crate::compare::row_comparator::compare_row_by_row;
use crate::compare::util::ProgressFn;
use crate::error::AppError;
use crate::models::compare_options::{CompareOptions, ComparisonMode};
use crate::models::compare_result::CompareResult;
use crate::AppState;

#[tauri::command]
pub async fn compare_files(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    options: CompareOptions,
) -> Result<CompareResult, AppError> {
    state.cancel.store(false, Ordering::Relaxed);
    let cancel = state.cancel.clone();
    let app_clone = app.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let progress: &ProgressFn = &move |processed: u64, total: Option<u64>, stage: &str| {
            let payload = serde_json::json!({
                "processed": processed,
                "total": total,
                "stage": stage,
            });
            let _ = app_clone.emit("compare-progress", payload);
        };

        match options.comparison_mode {
            ComparisonMode::KeyBased => compare_key_based(&options, &cancel, progress),
            ComparisonMode::RowByRow => compare_row_by_row(&options, &cancel, progress),
        }
    })
    .await
    .map_err(|e| AppError::Other(format!("背景任務失敗：{e}")))??;

    if result.cancelled {
        return Err(AppError::Cancelled);
    }

    let _ = app.emit("compare-done", ());
    Ok(result)
}
