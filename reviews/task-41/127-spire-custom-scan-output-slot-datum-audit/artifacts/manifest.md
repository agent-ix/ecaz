# Manifest: Task 41 Invariant #2 SPIRE CustomScan output slot audit

- head SHA: `a927f145377440f3adc1f2a00dddd9a75f835091`
- task bucket and packet path:
  `reviews/task-41/127-spire-custom-scan-output-slot-datum-audit/`
- lane / fixture / storage format / rerank mode: source audit; no SQL fixture,
  storage-format matrix, or rerank-mode execution.
- timestamp: `2026-05-18T03:12:32Z`
- isolated one-index-per-table or shared-table surfaces: not applicable; no
  benchmark or SQL execution.

## Artifacts

### custom-scan-slot-inventory.log

- command used:
  `rg -n "tts_values|tts_isnull|slot_getsomeattrs_int|ExecClearTuple|TupleTableSlot" src/am/ec_spire/custom_scan -g '*.rs'`
- key result lines:
  - `tuple_payload.rs` entries are assignments to `tts_isnull` and
    `tts_values`.
  - no `slot_getsomeattrs_int` entry appears under `src/am/ec_spire/custom_scan`.
  - `begin_exec.rs` entries clear or return scan tuple slots but do not read
    `tts_values`.

### json-payload-slot-output-excerpt.log

- command used:
  `sed -n '36,103p' src/am/ec_spire/custom_scan/tuple_payload.rs`
- key result lines:
  - JSON payload path clears the slot, writes null/value arrays, sets
    `tts_nvalid`, and calls `ExecStoreVirtualTuple`.

### typed-payload-slot-output-excerpt.log

- command used:
  `sed -n '145,229p' src/am/ec_spire/custom_scan/tuple_payload.rs`
- key result lines:
  - typed payload path clears the slot, writes null/value arrays, sets
    `tts_nvalid`, and calls `ExecStoreVirtualTuple`.

### git-status.log

- command used:
  `git status --short --branch`
- key result lines:
  - branch was `task41-invariant2-lifetimes`.
  - only the new CustomScan audit packet was untracked when the audit artifacts
    were captured.
