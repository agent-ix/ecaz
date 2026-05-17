//! On-disk format-version compatibility matrix checks.

use std::{collections::BTreeSet, path::Path};

#[derive(Debug, Clone, PartialEq, Eq)]
struct MatrixRow {
    am: String,
    format_version: u16,
    can_read: bool,
    can_write: bool,
    fixture: String,
    notes: String,
}

fn parse_bool(value: &str, line_no: usize, field: &str) -> bool {
    match value {
        "true" => true,
        "false" => false,
        other => panic!("line {line_no}: invalid {field} boolean {other:?}"),
    }
}

fn parse_matrix() -> Vec<MatrixRow> {
    let contents = include_str!("../fixtures/upgrade/matrix.csv");
    let mut lines = contents.lines();
    let header = lines.next().expect("matrix should have a header");
    assert_eq!(
        header, "am,format_version,can_read,can_write,fixture,notes",
        "unexpected matrix header"
    );

    lines
        .enumerate()
        .filter(|(_, line)| !line.trim().is_empty())
        .map(|(index, line)| {
            let line_no = index + 2;
            let fields = line.splitn(6, ',').collect::<Vec<_>>();
            assert_eq!(fields.len(), 6, "line {line_no}: expected 6 CSV fields");
            MatrixRow {
                am: fields[0].to_owned(),
                format_version: fields[1]
                    .parse()
                    .unwrap_or_else(|error| panic!("line {line_no}: invalid version: {error}")),
                can_read: parse_bool(fields[2], line_no, "can_read"),
                can_write: parse_bool(fields[3], line_no, "can_write"),
                fixture: fields[4].to_owned(),
                notes: fields[5].to_owned(),
            }
        })
        .collect()
}

#[test]
fn upgrade_matrix_has_unique_entries_and_existing_fixtures() {
    let rows = parse_matrix();
    assert!(!rows.is_empty(), "upgrade matrix should not be empty");

    let mut keys = BTreeSet::new();
    for row in &rows {
        assert!(
            keys.insert((row.am.as_str(), row.format_version)),
            "duplicate matrix row for {} v{}",
            row.am,
            row.format_version
        );
        assert!(
            row.can_read || !row.can_write,
            "{} v{} cannot be writable unless it is readable",
            row.am,
            row.format_version
        );
        assert!(
            !row.notes.is_empty(),
            "matrix row should document rationale"
        );
        assert!(
            Path::new(&row.fixture).exists(),
            "fixture path does not exist: {}",
            row.fixture
        );
    }
}

#[test]
fn upgrade_matrix_names_current_writable_formats() {
    let rows = parse_matrix();
    let writable = rows
        .iter()
        .filter(|row| row.can_write)
        .map(|row| (row.am.as_str(), row.format_version))
        .collect::<BTreeSet<_>>();

    let expected = BTreeSet::from([
        ("diskann", 3),
        ("hnsw", 3),
        ("ivf", 1),
        ("spire-partition", 1),
        ("spire-partition", 2),
    ]);

    assert_eq!(writable, expected);
}
