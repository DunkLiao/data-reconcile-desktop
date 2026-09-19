use std::fs::File;
use std::io::{BufReader, Read, Result as IoResult};

use encoding_rs::Decoder;

use crate::error::AppError;
use crate::models::parse_options::{DelimiterOption, EncodingOption};

use super::delimiter::detect_delimiter;
use super::encoding::{detect_encoding, DetectedEncoding};

/// Read up to `max_bytes` raw bytes from the start of a file.
pub fn read_sample(path: &str, max_bytes: usize) -> Result<Vec<u8>, AppError> {
    if !std::path::Path::new(path).exists() {
        return Err(AppError::FileNotFound(path.to_string()));
    }
    let file = File::open(path).map_err(|_| AppError::FileAccessError)?;
    let mut reader = BufReader::new(file);
    let mut buf = vec![0u8; max_bytes];
    let mut total = 0usize;
    loop {
        let n = reader.read(&mut buf[total..])?;
        if n == 0 {
            break;
        }
        total += n;
        if total >= max_bytes {
            break;
        }
    }
    buf.truncate(total);
    Ok(buf)
}

pub fn decode_to_string(bytes: &[u8], encoding: &'static encoding_rs::Encoding) -> String {
    let (cow, _, _) = encoding.decode(bytes);
    cow.into_owned()
}

/// Resolve the concrete encoding for a file given a (possibly Auto) option.
/// Returns the detected encoding and whether the result is reliable.
pub fn resolve_encoding(
    path: &str,
    option: EncodingOption,
) -> Result<(DetectedEncoding, bool), AppError> {
    match option {
        EncodingOption::Utf8 => Ok((DetectedEncoding::Utf8, true)),
        EncodingOption::Big5 => Ok((DetectedEncoding::Big5, true)),
        EncodingOption::Cp950 => Ok((DetectedEncoding::Cp950, true)),
        EncodingOption::Utf16Le => Ok((DetectedEncoding::Utf16Le, true)),
        EncodingOption::Utf16Be => Ok((DetectedEncoding::Utf16Be, true)),
        EncodingOption::Auto => {
            let sample = read_sample(path, 128 * 1024)?;
            Ok(detect_encoding(&sample))
        }
    }
}

/// Resolve the concrete delimiter for a file given a (possibly Auto) option.
pub fn resolve_delimiter(
    path: &str,
    encoding: &'static encoding_rs::Encoding,
    option: DelimiterOption,
) -> Result<u8, AppError> {
    match option {
        DelimiterOption::Comma => Ok(b','),
        DelimiterOption::Semicolon => Ok(b';'),
        DelimiterOption::Tab => Ok(b'\t'),
        DelimiterOption::Pipe => Ok(b'|'),
        DelimiterOption::Colon => Ok(b':'),
        DelimiterOption::Custom(c) => {
            if c.len_utf8() != 1 || c as u32 > 0xFF {
                return Err(AppError::Other(
                    "自訂分隔符號僅支援單一位元組字元（ASCII）。".to_string(),
                ));
            }
            Ok(c as u8)
        }
        DelimiterOption::Auto => {
            let sample = read_sample(path, 256 * 1024)?;
            let decoded = decode_to_string(&sample, encoding);
            match detect_delimiter(&decoded) {
                Some(d) => Ok(d),
                None => Err(AppError::DelimiterDetectionError),
            }
        }
    }
}

pub fn file_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

/// A streaming reader that decodes raw bytes from a source encoding to UTF-8
/// as it is consumed, so files never need to be loaded fully into memory.
pub struct DecodingReader<R: Read> {
    inner: BufReader<R>,
    decoder: Decoder,
    input_buf: Vec<u8>,
    pending: Vec<u8>,
    finished: bool,
}

impl<R: Read> DecodingReader<R> {
    pub fn new(inner: R, encoding: &'static encoding_rs::Encoding) -> Self {
        DecodingReader {
            inner: BufReader::new(inner),
            decoder: encoding.new_decoder(),
            input_buf: vec![0u8; 64 * 1024],
            pending: Vec::new(),
            finished: false,
        }
    }

    /// Read and decode the next raw chunk into the pending buffer.
    fn refill(&mut self) -> IoResult<()> {
        if self.finished {
            return Ok(());
        }

        let n = self.inner.read(&mut self.input_buf)?;
        let last = n == 0;

        let mut decoded = vec![0u8; n * 2 + 64];
        let mut written_total = 0usize;

        if n > 0 {
            let (result, _, written, _had_errors) =
                self.decoder
                    .decode_to_utf8(&self.input_buf[..n], &mut decoded, false);
            written_total = written;
            if result == encoding_rs::CoderResult::OutputFull {
                // Our buffer is sized at 2x input, which is always sufficient for
                // the supported encodings (max expansion is 1.5x for UTF-16).
            }
        }

        if last {
            let mut flush_buf = vec![0u8; 64];
            let (_, _, w, _) = self.decoder.decode_to_utf8(&[], &mut flush_buf, true);
            self.pending.extend_from_slice(&flush_buf[..w]);
            self.finished = true;
        }

        if written_total > 0 {
            self.pending.extend_from_slice(&decoded[..written_total]);
        }

        Ok(())
    }
}

impl<R: Read> Read for DecodingReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        loop {
            if !self.pending.is_empty() {
                let take = buf.len().min(self.pending.len());
                buf[..take].copy_from_slice(&self.pending[..take]);
                self.pending.drain(..take);
                return Ok(take);
            }
            if self.finished {
                return Ok(0);
            }
            self.refill()?;
            if self.finished && self.pending.is_empty() {
                return Ok(0);
            }
        }
    }
}
