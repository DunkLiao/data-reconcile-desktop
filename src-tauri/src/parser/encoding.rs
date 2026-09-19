use chardetng::EncodingDetector;
use encoding_rs::{Encoding, BIG5, UTF_16BE, UTF_16LE, UTF_8};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedEncoding {
    Utf8,
    Big5,
    Utf16Le,
    Utf16Be,
    Cp950,
}

impl DetectedEncoding {
    pub fn label(&self) -> &'static str {
        match self {
            DetectedEncoding::Utf8 => "UTF-8",
            DetectedEncoding::Big5 => "Big5",
            DetectedEncoding::Utf16Le => "UTF-16 LE",
            DetectedEncoding::Utf16Be => "UTF-16 BE",
            DetectedEncoding::Cp950 => "CP950",
        }
    }

    pub fn encoding(&self) -> &'static Encoding {
        match self {
            DetectedEncoding::Utf8 => UTF_8,
            DetectedEncoding::Big5 | DetectedEncoding::Cp950 => BIG5,
            DetectedEncoding::Utf16Le => UTF_16LE,
            DetectedEncoding::Utf16Be => UTF_16BE,
        }
    }
}

/// Detect the encoding of raw sample bytes.
/// Returns the encoding and whether the detection is reliable.
pub fn detect_encoding(bytes: &[u8]) -> (DetectedEncoding, bool) {
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return (DetectedEncoding::Utf8, true);
    }
    if bytes.starts_with(&[0xFF, 0xFE, 0x00, 0x00]) {
        return (DetectedEncoding::Utf16Le, true);
    }
    if bytes.starts_with(&[0xFF, 0xFE]) {
        return (DetectedEncoding::Utf16Le, true);
    }
    if bytes.starts_with(&[0xFE, 0xFF]) {
        return (DetectedEncoding::Utf16Be, true);
    }
    if bytes.len() >= 4
        && bytes[0] == 0x00
        && bytes[1] == 0x00
        && bytes[2] == 0xFE
        && bytes[3] == 0xFF
    {
        return (DetectedEncoding::Utf16Be, true);
    }
    if bytes.len() >= 4
        && bytes[0] == 0xFF
        && bytes[1] == 0xFE
        && bytes[2] == 0x00
        && bytes[3] == 0x00
    {
        return (DetectedEncoding::Utf16Le, true);
    }

    if std::str::from_utf8(bytes).is_ok() {
        return (DetectedEncoding::Utf8, true);
    }

    let mut det = EncodingDetector::new();
    det.feed(bytes, true);
    let guessed = det.guess(None, true);

    if guessed == UTF_8 {
        return (DetectedEncoding::Utf8, true);
    }
    if guessed == UTF_16LE {
        return (DetectedEncoding::Utf16Le, true);
    }
    if guessed == UTF_16BE {
        return (DetectedEncoding::Utf16Be, true);
    }
    if guessed == BIG5 {
        return (DetectedEncoding::Big5, true);
    }

    // Fallback for traditional Chinese environments (Big5 / CP950).
    (DetectedEncoding::Cp950, false)
}
