use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EncodingOption {
    Auto,
    Utf8,
    Big5,
    Cp950,
    Utf16Le,
    Utf16Be,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type", content = "value")]
pub enum DelimiterOption {
    Auto,
    Comma,
    Semicolon,
    Tab,
    Pipe,
    Colon,
    Custom(char),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ParseOptions {
    pub encoding: EncodingOption,
    pub delimiter: DelimiterOption,
}
