use crate::error::AppError;
use crate::excel::exporter::export_excel as do_export;
use crate::models::compare_result::CompareResult;

#[tauri::command]
pub async fn export_excel(result: CompareResult, output_path: String) -> Result<String, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        do_export(&result, &output_path)?;
        Ok(output_path)
    })
    .await
    .map_err(|e| AppError::Other(format!("背景任務失敗：{e}")))?
}
