use std::collections::HashMap;
use std::io::Read;

use crate::error::AppError;

/// Streaming delimited-text reader.
/// First logical record is treated as the header; subsequent records are data.
/// Field-count mismatches against the header are reported as InvalidRecord.
pub struct DelimitedReader<R: Read> {
    csv: csv::Reader<R>,
    headers: Vec<String>,
    header_len: usize,
    record_no: u64,
    done: bool,
    strict: bool,
}

impl<R: Read> DelimitedReader<R> {
    pub fn new(inner: R, delimiter: u8, strict: bool) -> Result<Self, AppError> {
        let mut csv = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(delimiter)
            .flexible(true)
            .from_reader(inner);

        let mut rec = csv::StringRecord::new();
        match csv.read_record(&mut rec) {
            Ok(true) => {}
            Ok(false) => {
                return Err(AppError::Other(
                    "檔案內容為空白，無法解析 Header。".to_string(),
                ));
            }
            Err(e) => {
                return Err(AppError::Other(format!("CSV 解析錯誤：{e}")));
            }
        }

        let headers: Vec<String> = rec.iter().map(|s| s.to_string()).collect();
        if headers.is_empty() {
            return Err(AppError::Other(
                "檔案內容為空白，無法解析 Header。".to_string(),
            ));
        }
        let header_len = headers.len();

        Ok(DelimitedReader {
            csv,
            headers,
            header_len,
            record_no: 0,
            done: false,
            strict,
        })
    }

    pub fn headers(&self) -> &[String] {
        &self.headers
    }

    /// Returns the next data record as `(logical_record_number, fields)`.
    pub fn next_record(&mut self) -> Result<Option<(u64, Vec<String>)>, AppError> {
        if self.done {
            return Ok(None);
        }
        let mut rec = csv::StringRecord::new();
        match self.csv.read_record(&mut rec) {
            Ok(true) => {
                self.record_no += 1;
                let fields: Vec<String> = rec.iter().map(|s| s.to_string()).collect();
                if self.strict && fields.len() != self.header_len {
                    return Err(AppError::InvalidRecord {
                        record: self.record_no,
                        expected: self.header_len,
                        actual: fields.len(),
                    });
                }
                Ok(Some((self.record_no, fields)))
            }
            Ok(false) => {
                self.done = true;
                Ok(None)
            }
            Err(e) => Err(AppError::Other(format!("CSV 解析錯誤：{e}"))),
        }
    }
}

pub fn check_duplicate_headers(headers: &[String]) -> Result<(), AppError> {
    let mut seen: HashMap<&str, u32> = HashMap::new();
    for h in headers {
        *seen.entry(h.as_str()).or_insert(0) += 1;
    }
    if let Some(dup) = seen.iter().find(|(_, &c)| c > 1) {
        return Err(AppError::DuplicateHeader(dup.0.to_string()));
    }
    Ok(())
}
