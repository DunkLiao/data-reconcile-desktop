use serde::ser::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("找不到檔案：\n\n{0}")]
    FileNotFound(String),
    #[error("無法讀取檔案。\n\n請確認檔案是否存在，以及目前帳號是否具有讀取權限。")]
    FileAccessError,
    #[error("無法可靠判斷欄位分隔符號。\n\n請手動指定 Delimiter。")]
    DelimiterDetectionError,
    #[error("無法可靠判斷檔案 Encoding。\n\n請手動指定 Encoding 後再進行比較。")]
    EncodingDetectionError,
    #[error("檔案格式異常。\n\nRecord：{record}\n預期欄位：{expected}\n實際欄位：{actual}")]
    InvalidRecord {
        record: u64,
        expected: usize,
        actual: usize,
    },
    #[error("發現重複欄位名稱：\n\n{0}\n\n請修正來源檔案後重新執行。")]
    DuplicateHeader(String),
    #[error("Key 欄位不存在於兩個檔案：\n\n{0}")]
    MissingKeyColumn(String),
    #[error("Excel 報告無法儲存。\n\n請確認：\n1. 輸出資料夾具有寫入權限\n2. 同名 Excel 是否正在開啟\n3. 磁碟空間是否足夠")]
    ExportError,
    #[error("比較已取消")]
    Cancelled,
    #[error("{0}")]
    Io(String),
    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn to_message(&self) -> String {
        match self {
            AppError::Io(e) => format!("無法讀取檔案：{e}"),
            other => other.to_string(),
        }
    }
}

impl From<std::io::Error> for AppError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::PermissionDenied => AppError::FileAccessError,
            _ => AppError::Io(e.to_string()),
        }
    }
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_message())
    }
}
