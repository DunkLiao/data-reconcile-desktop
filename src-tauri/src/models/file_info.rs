use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub encoding: String,
    pub encoding_confident: bool,
    pub encoding_warning: Option<String>,
    pub delimiter: String,
    pub delimiter_confident: bool,
    pub headers: Vec<String>,
    pub row_count: Option<u64>,
}
