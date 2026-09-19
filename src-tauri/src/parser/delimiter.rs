const CANDIDATES: &[u8] = b",;\t|:";

/// Detect the most likely delimiter from a decoded UTF-8 sample.
/// Analyzes logical records for stable field counts under each candidate.
pub fn detect_delimiter(sample: &str) -> Option<u8> {
    let mut best: Option<(u8, u64, usize)> = None;

    for &candidate in CANDIDATES {
        if !sample.contains(candidate as char) {
            continue;
        }

        let mut reader = csv::ReaderBuilder::new()
            .has_headers(false)
            .delimiter(candidate)
            .flexible(true)
            .from_reader(sample.as_bytes());

        let mut counts: Vec<usize> = Vec::new();
        let mut record = csv::StringRecord::new();
        while reader.read_record(&mut record).unwrap_or(false) {
            if !record.is_empty() {
                counts.push(record.len());
            }
        }

        if counts.is_empty() {
            continue;
        }

        // Find the mode (most common field count).
        let mut frequency: std::collections::HashMap<usize, u64> = std::collections::HashMap::new();
        for &c in &counts {
            *frequency.entry(c).or_insert(0) += 1;
        }
        let (&mode, &matching) = frequency
            .iter()
            .max_by(|a, b| a.1.cmp(b.1).then(a.0.cmp(b.0)))
            .unwrap();

        if mode < 2 {
            continue;
        }

        // Prefer candidates that produce many stable multi-field records.
        let score = matching.saturating_mul(1000).saturating_add(mode as u64);

        if best.is_none() || score > best.as_ref().unwrap().1 {
            best = Some((candidate, score, mode));
        }
    }

    let (delim, _score, mode) = best?;
    if mode < 2 {
        return None;
    }
    Some(delim)
}

pub fn delimiter_label(delim: u8) -> String {
    match delim {
        b',' => "Comma (,)".to_string(),
        b';' => "Semicolon (;)".to_string(),
        b'\t' => "Tab".to_string(),
        b'|' => "Pipe (|)".to_string(),
        b':' => "Colon (:)".to_string(),
        c => format!("Custom ({})", c as char),
    }
}
