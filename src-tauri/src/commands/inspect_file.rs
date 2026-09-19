use std::fs::File;
use std::io::BufReader;

use crate::error::AppError;
use crate::models::file_info::FileInfo;
use crate::models::parse_options::{DelimiterOption, EncodingOption};
use crate::parser::delimited_parser::{check_duplicate_headers, DelimitedReader};
use crate::parser::delimiter::delimiter_label;
use crate::parser::reader::{resolve_delimiter, resolve_encoding, DecodingReader};

#[tauri::command]
pub async fn inspect_file(
    path: String,
    encoding: EncodingOption,
    delimiter: DelimiterOption,
) -> Result<FileInfo, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        if !std::path::Path::new(&path).exists() {
            return Err(AppError::FileNotFound(path.clone()));
        }

        let (detected, confident) = resolve_encoding(&path, encoding)?;
        let encoding_label = detected.label().to_string();
        let encoding_warning = if encoding == EncodingOption::Auto && !confident {
            Some("無法可靠判斷文字編碼，請手動指定 Encoding。".to_string())
        } else {
            None
        };

        let delim = resolve_delimiter(&path, detected.encoding(), delimiter)?;
        let delim_label = delimiter_label(delim);

        let file = File::open(&path).map_err(|_| AppError::FileAccessError)?;
        let reader = DelimitedReader::new(
            DecodingReader::new(BufReader::new(file), detected.encoding()),
            delim,
            false,
        )?;
        let headers = reader.headers().to_vec();
        check_duplicate_headers(&headers)?;

        Ok(FileInfo {
            path,
            encoding: encoding_label,
            encoding_confident: confident,
            encoding_warning,
            delimiter: delim_label,
            delimiter_confident: true,
            headers,
            row_count: None,
        })
    })
    .await
    .map_err(|e| AppError::Other(format!("背景任務失敗：{e}")))?
}
