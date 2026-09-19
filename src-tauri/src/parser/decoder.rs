use std::fs::File;
use std::io::BufReader;

use crate::error::AppError;
use crate::models::parse_options::{DelimiterOption, EncodingOption};

use super::delimited_parser::DelimitedReader;
use super::reader::{resolve_delimiter, resolve_encoding, DecodingReader};

pub type OpenedDelimitedReader = DelimitedReader<DecodingReader<BufReader<File>>>;

/// Open a file and produce a streaming delimited reader using the given options.
/// Resolves Auto encoding / delimiter from a sample before parsing.
pub fn open_delimited_reader(
    path: &str,
    encoding_option: EncodingOption,
    delimiter_option: DelimiterOption,
    strict: bool,
) -> Result<(OpenedDelimitedReader, String, String), AppError> {
    if !std::path::Path::new(path).exists() {
        return Err(AppError::FileNotFound(path.to_string()));
    }

    let (encoding, confident) = resolve_encoding(path, encoding_option)?;
    if encoding_option == EncodingOption::Auto && !confident {
        return Err(AppError::EncodingDetectionError);
    }
    let delimiter = resolve_delimiter(path, encoding.encoding(), delimiter_option)?;

    let file = File::open(path).map_err(|_| AppError::FileAccessError)?;
    let decoding = DecodingReader::new(BufReader::new(file), encoding.encoding());
    let reader = DelimitedReader::new(decoding, delimiter, strict)?;

    let encoding_label = encoding.label().to_string();
    let delimiter_label = super::delimiter::delimiter_label(delimiter);

    Ok((reader, encoding_label, delimiter_label))
}
