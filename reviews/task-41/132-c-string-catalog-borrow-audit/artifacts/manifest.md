# Manifest: Task 41 Invariant #2 C string/catalog borrow audit

- head SHA: `ae3e516d1c1444ccc68d705738c985a2327327d3`
- task bucket and packet path:
  `reviews/task-41/132-c-string-catalog-borrow-audit/`
- lane / fixture / storage format / rerank mode: source audit; no SQL fixture,
  storage-format matrix, or rerank-mode execution.
- timestamp: `2026-05-18T03:25:45Z`
- isolated one-index-per-table or shared-table surfaces: not applicable; no
  benchmark or SQL execution.

## Artifacts

### c-string-inventory.log

- command used:
  `rg -n "CStr::from_ptr|to_str\\(\\)|to_string_lossy\\(\\)|pfree\\(|format_type_be|format_type_extended|NameStr" src/am src/lib.rs src/storage -g '*.rs'`
- key result lines:
  - `86` inventory lines.
  - `format_type_be` sites copy through `to_string_lossy().into_owned()` or
    `to_str().map(str::to_owned)` before `pfree`.
  - reloption helpers return owned `String`.
  - tuple descriptor attribute names are used synchronously while filling
    output slots.

### hnsw-format-type-excerpt.log

- command used:
  `sed -n '395,430p' src/am/ec_hnsw/source.rs`
- key result lines:
  - formatted type name is copied to owned `String`.
  - `pfree(formatted)` happens before parsing the owned string.

### spire-dml-format-type-excerpt.log

- command used:
  `sed -n '1483,1503p' src/am/ec_spire/dml_frontdoor/mod.rs`
- key result lines:
  - attribute names and formatted type names are returned as owned `String`.
  - `pfree(type_name)` happens after owned decode is computed.

### custom-scan-attr-name-excerpt.log

- command used:
  `sed -n '64,91p' src/am/ec_spire/custom_scan/tuple_payload.rs`
- key result lines:
  - tuple descriptor attribute name is used synchronously for payload lookup
    and datum conversion while the tuple descriptor remains live.

### options-owned-string-excerpt.log

- command used:
  `sed -n '1580,1595p' src/am/ec_spire/options/mod.rs`
- key result lines:
  - reloption string is validated as `&str` and returned via `Some(value.to_owned())`.

### git-status.log

- command used:
  `git status --short --branch`
- key result lines:
  - branch was `task41-invariant2-lifetimes`.
  - only the new C-string audit packet was untracked when the audit artifacts
    were captured.
