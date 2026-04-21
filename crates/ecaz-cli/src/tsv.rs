//! Corpus/query TSV file reader.
//!
//! Each non-empty line is `<id>\t<json_array>` where `<json_array>` is a
//! JSON array of floats. No header row.
//!
//! The reader streams one row at a time so 10M-row corpora don't need to
//! fit in RAM all at once. Callers that need to pre-validate file shape
//! (for manifest checks) can iterate the reader to completion first; the
//! file is small relative to the eventual Postgres footprint.

use color_eyre::eyre::{eyre, Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug)]
pub struct VectorLine {
    pub id: i64,
    pub values: Vec<f32>,
}

#[derive(Debug)]
pub struct VectorFileStats {
    pub rows: usize,
    pub sha256_hex: String,
    pub first_id: Option<i64>,
    pub last_id: Option<i64>,
}

/// Iterate non-empty rows from `path`. Each parsed row is validated to
/// contain exactly `dim` floats; mismatched rows are surfaced with a
/// line-number-prefixed error so operators can find bad input fast.
pub fn iter_rows(
    path: &Path,
    dim: usize,
) -> Result<impl Iterator<Item = Result<VectorLine>>> {
    let file = File::open(path)
        .wrap_err_with(|| format!("opening vector file {}", path.display()))?;
    let reader = BufReader::new(file);
    let path_display = path.display().to_string();
    Ok(reader
        .lines()
        .enumerate()
        .filter_map(move |(idx, line)| {
            let line_number = idx + 1;
            match line {
                Ok(raw) => {
                    let trimmed = raw.trim_end_matches(['\r', '\n']);
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(parse_line(&path_display, line_number, trimmed, dim))
                    }
                }
                Err(e) => Some(Err(eyre!(
                    "{}:{}: read error: {}",
                    path_display,
                    line_number,
                    e
                ))),
            }
        }))
}

/// Single-pass stats computation without materialising the full corpus in
/// RAM. Intended for manifest verification and the pre-load summary line.
pub fn inspect(path: &Path, dim: usize) -> Result<VectorFileStats> {
    use sha2::{Digest, Sha256};
    let file = File::open(path)
        .wrap_err_with(|| format!("opening vector file {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut rows = 0usize;
    let mut first_id = None;
    let mut last_id = None;
    let path_display = path.display().to_string();

    for (idx, line) in reader.lines().enumerate() {
        let line_number = idx + 1;
        let raw = line.wrap_err_with(|| format!("{}:{}: read error", path_display, line_number))?;
        // Hash the line's on-disk bytes (including the trailing newline) so the
        // sha256 matches a plain-file digest; we reconstruct the \n we stripped.
        hasher.update(raw.as_bytes());
        hasher.update(b"\n");
        let trimmed = raw.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            continue;
        }
        let parsed = parse_line(&path_display, line_number, trimmed, dim)?;
        if first_id.is_none() {
            first_id = Some(parsed.id);
        }
        last_id = Some(parsed.id);
        rows += 1;
    }

    Ok(VectorFileStats {
        rows,
        sha256_hex: hex::encode(hasher.finalize()),
        first_id,
        last_id,
    })
}

fn parse_line(path: &str, line_number: usize, line: &str, dim: usize) -> Result<VectorLine> {
    let (id_str, json_str) = line.split_once('\t').ok_or_else(|| {
        eyre!(
            "{}:{}: expected '<id>\\t<json_array>' line, got {:?}",
            path,
            line_number,
            line
        )
    })?;
    let id: i64 = id_str.parse().map_err(|_| {
        eyre!(
            "{}:{}: id {:?} is not an integer",
            path,
            line_number,
            id_str
        )
    })?;
    let values: Vec<f32> = serde_json::from_str(json_str).map_err(|e| {
        eyre!(
            "{}:{}: embedding column is not valid JSON: {}",
            path,
            line_number,
            e
        )
    })?;
    if values.len() != dim {
        return Err(eyre!(
            "{}:{}: expected dim {}, got {}",
            path,
            line_number,
            dim,
            values.len()
        ));
    }
    Ok(VectorLine { id, values })
}

/// Format a `real[]` array literal for Postgres COPY (text format).
pub fn format_real_array_literal(values: &[f32]) -> String {
    let mut buf = String::with_capacity(values.len() * 12 + 2);
    buf.push('{');
    for (i, v) in values.iter().enumerate() {
        if i > 0 {
            buf.push(',');
        }
        use std::fmt::Write;
        // `{:?}` on f32 prints a round-trippable decimal (e.g. "0.123456"),
        // matching the legacy loader's `repr(float(v))` output.
        write!(buf, "{v:?}").expect("writing to String never fails");
    }
    buf.push('}');
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn write_temp(contents: &str) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn iter_rows_parses_valid_file() {
        let f = write_temp("1\t[0.1, 0.2, 0.3]\n2\t[0.4,0.5,0.6]\n");
        let rows: Vec<_> = iter_rows(f.path(), 3)
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, 1);
        assert_eq!(rows[0].values, vec![0.1, 0.2, 0.3]);
        assert_eq!(rows[1].id, 2);
    }

    #[test]
    fn iter_rows_skips_blank_lines() {
        let f = write_temp("1\t[1.0, 2.0]\n\n2\t[3.0, 4.0]\n");
        let rows: Vec<_> = iter_rows(f.path(), 2)
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert_eq!(rows.len(), 2);
    }

    #[test]
    fn iter_rows_rejects_dim_mismatch_with_line_number() {
        let f = write_temp("1\t[1.0, 2.0]\n2\t[3.0]\n");
        let mut it = iter_rows(f.path(), 2).unwrap();
        let _ = it.next().unwrap().unwrap();
        let err = it.next().unwrap().unwrap_err().to_string();
        assert!(err.contains(":2:"), "err missing line number: {err}");
        assert!(err.contains("expected dim 2"), "err: {err}");
    }

    #[test]
    fn iter_rows_rejects_non_integer_id() {
        let f = write_temp("abc\t[1.0]\n");
        let err = iter_rows(f.path(), 1).unwrap().next().unwrap().unwrap_err().to_string();
        assert!(err.contains("is not an integer"), "err: {err}");
    }

    #[test]
    fn inspect_counts_rows_and_tracks_first_last_id() {
        let f = write_temp("10\t[1.0]\n11\t[2.0]\n12\t[3.0]\n");
        let stats = inspect(f.path(), 1).unwrap();
        assert_eq!(stats.rows, 3);
        assert_eq!(stats.first_id, Some(10));
        assert_eq!(stats.last_id, Some(12));
        assert_eq!(stats.sha256_hex.len(), 64);
    }

    #[test]
    fn format_real_array_literal_uses_pg_array_syntax() {
        assert_eq!(format_real_array_literal(&[1.0, 2.5, -3.0]), "{1.0,2.5,-3.0}");
        assert_eq!(format_real_array_literal(&[]), "{}");
    }

    #[test]
    fn format_real_array_literal_roundtrips_fractional_floats() {
        // `{:?}` on f32 must produce the shortest round-trippable decimal so
        // Postgres parses back to the same bits. Guarding against a regression
        // that would subtly change recall numbers.
        let v = [0.123_456_f32, -0.000_1, 1e-10, 1e10];
        let rendered = format_real_array_literal(&v);
        let trimmed = &rendered[1..rendered.len() - 1];
        let parsed: Vec<f32> = trimmed.split(',').map(|s| s.parse().unwrap()).collect();
        assert_eq!(parsed, v);
    }

    #[test]
    fn iter_rows_handles_crlf_line_endings() {
        let f = write_temp("1\t[1.0, 2.0]\r\n2\t[3.0, 4.0]\r\n");
        let rows: Vec<_> = iter_rows(f.path(), 2)
            .unwrap()
            .collect::<Result<Vec<_>>>()
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].values, vec![1.0, 2.0]);
    }

    #[test]
    fn iter_rows_rejects_missing_tab_separator() {
        let f = write_temp("nope_no_tab\n");
        let err = iter_rows(f.path(), 1).unwrap().next().unwrap().unwrap_err().to_string();
        assert!(err.contains("expected '<id>\\t<json_array>'"), "err: {err}");
    }

    #[test]
    fn iter_rows_rejects_invalid_json() {
        let f = write_temp("1\t[not-json]\n");
        let err = iter_rows(f.path(), 1).unwrap().next().unwrap().unwrap_err().to_string();
        assert!(err.contains("not valid JSON"), "err: {err}");
    }

    #[test]
    fn inspect_on_empty_file_reports_zero_rows_and_no_ids() {
        let f = write_temp("");
        let s = inspect(f.path(), 1).unwrap();
        assert_eq!(s.rows, 0);
        assert_eq!(s.first_id, None);
        assert_eq!(s.last_id, None);
        assert_eq!(s.sha256_hex.len(), 64);
    }

    #[test]
    fn inspect_sha256_matches_plain_file_digest() {
        use sha2::{Digest, Sha256};
        let body = "10\t[1.0]\n11\t[2.0]\n";
        let f = write_temp(body);
        let stats = inspect(f.path(), 1).unwrap();
        let expected = hex::encode(Sha256::digest(body.as_bytes()));
        assert_eq!(stats.sha256_hex, expected);
    }
}
