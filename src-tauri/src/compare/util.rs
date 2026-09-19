use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Local;

use crate::error::AppError;
use crate::models::compare_result::CompareResult;

pub type ProgressFn = dyn Fn(u64, Option<u64>, &str) + Send + Sync;

pub fn check_cancel(cancel: &AtomicBool) -> Result<(), AppError> {
    if cancel.load(Ordering::Relaxed) {
        return Err(AppError::Cancelled);
    }
    Ok(())
}

pub fn now_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn is_identical(r: &CompareResult) -> bool {
    r.a_only_records == 0
        && r.b_only_records == 0
        && r.different_records == 0
        && r.duplicate_keys_a == 0
        && r.duplicate_keys_b == 0
        && r.empty_keys_a == 0
        && r.empty_keys_b == 0
        && r.incomplete_keys_a == 0
        && r.incomplete_keys_b == 0
        && r.column_a_only.is_empty()
        && r.column_b_only.is_empty()
}
