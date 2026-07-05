# Manifest: Task 41 Invariant #2 SPIRE scan slot Datum lifetime audit

- head SHA: `4a9e63f524b26919fd6c4ecf664c515266704b15`
- task bucket and packet path:
  `reviews/task-41/125-spire-scan-slot-datum-lifetime-audit/`
- lane / fixture / storage format / rerank mode: source audit; no SQL fixture,
  storage-format matrix, or rerank-mode execution.
- timestamp: `2026-05-18T03:09:26Z`
- isolated one-index-per-table or shared-table surfaces: not applicable; no
  benchmark or SQL execution.

## Artifacts

### spire-scan-slot-callers.log

- command used:
  `rg -n "required_slot_datum\\(|fetch_heap_row_version\\(|load_indexed_source_vector_from_heap_row\\(|indexed_vector_datum_to_source_vector\\(|detoasted_varlena_bytes\\(|ExecClearTuple\\(" src/am/ec_spire/scan/relation.rs -g '*.rs'`
- key result lines:
  - `355`: returned Datum comes from local `required_slot_datum`.
  - `357`: Datum is immediately converted by
    `indexed_vector_datum_to_source_vector`.
  - `358`: slot is cleared after the conversion result is computed.
  - `400`: conversion copies through `detoasted_varlena_bytes`.

### spire-scan-slot-excerpt.log

- command used:
  `sed -n '345,407p' src/am/ec_spire/scan/relation.rs`
- key result lines:
  - `load_indexed_source_vector_from_heap_row` fetches the heap tuple, reads
    the Datum, converts it, and clears the slot.
  - `indexed_vector_datum_to_source_vector` returns an owned `Vec<f32>`.

### spire-detoast-copy-excerpt.log

- command used:
  `sed -n '417,452p' src/am/ec_spire/scan/relation.rs`
- key result lines:
  - `detoasted_varlena_bytes` maps `DetoastedScanDatum` to `datum.to_vec()`.
  - `DetoastedScanDatum` keeps copied detoast storage guard-owned.

### git-status.log

- command used:
  `git status --short --branch`
- key result lines:
  - branch was `task41-invariant2-lifetimes`.
  - worktree had unrelated comparator-script changes plus this new audit
    packet; no Task 41 source file changes were made for this packet.
