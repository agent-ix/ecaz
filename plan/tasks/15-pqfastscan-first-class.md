# Task 15: Land PqFastScan as First-Class Index Format

Status: proposed — unblocks merge of the `adr030-v2-*` branch line to `main`.

Executes ADR-032.

## Scope

Close the gap between the `adr030-v2-*` experimental build and a
production-supported index format. When this task completes, `main` carries
both **TurboQuant** (renamed from `ScalarV1`) and **PqFastScan** (renamed
from `GroupedV2`) as first-class, per-index-selectable storage formats.

No experimental gate. No env-var selection. Parity on build, scan, insert,
and vacuum for both formats.

## Subtasks

### Rename

- [ ] **Rust enum rename.** `GraphStorageDescriptor::ScalarV1` → `TurboQuant`;
  `::GroupedV2(GroupedGraphLayout)` → `::PqFastScan(PqFastScanLayout)`.
  Rename struct `GroupedGraphLayout` → `PqFastScanLayout`. Wire tags
  (`INDEX_FORMAT_V1_SCALAR` / `INDEX_FORMAT_V2_GROUPED`) are **not**
  renamed — they stay as disk versioning bytes.
- [ ] **Module/function rename.** `grouped_v2_flush_output` →
  `pq_fastscan_flush_output`. `experimental_grouped_v2_*` → delete or rename
  without the experimental prefix. `src/quant/grouped_pq.rs` stays as-is —
  the algorithm *is* grouped PQ, independent of the surrounding format name.
- [ ] **Error messages and log lines.** Replace "grouped-v2" / "scalar-v1"
  references with "PqFastScan" / "TurboQuant". Update
  `ADR030_GROUPED_V2_*_UNSUPPORTED` constants (removed by the next
  sections, but audit strings in the interim).

### Reloption

- [ ] **Add `storage_format` reloption** in `src/am/options.rs`. Values:
  `'turboquant'` (default), `'pq_fastscan'`. Validate at CREATE INDEX.
- [ ] **Plumb reloption into `flush_build_state`**, replacing the
  `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD` env var check at
  `src/am/build.rs:1102`.
- [ ] **Remove the env var.** Delete `ADR030_EXPERIMENTAL_BUILD_ENV`
  and `experimental_grouped_v2_build_enabled()`. Update any dev scripts
  that reference it.

### Insert parity

- [ ] **Implement `tqhnsw_aminsert` for PqFastScan.** Re-encode the new
  vector into the existing learned subvector codebooks, emit the hot
  search code, the optional binary sidecar, and the cold rerank payload.
  Follow ADR-026 lock ordering (layer-0 backlink lock before upper-layer
  write locks).
- [ ] **Insert test.** PgTAP or `#[pg_test]` case that builds a
  PqFastScan index, inserts N rows post-build, and confirms
  `ORDER BY v <#> q` returns expected recall on the inserted rows.
- [ ] **Remove `ADR030_GROUPED_V2_INSERT_UNSUPPORTED`** reject path at
  `src/am/insert.rs:186`.

### Vacuum parity

- [ ] **Implement `tqhnsw_ambulkdelete` for PqFastScan.** Scan hot and
  cold data pages, mark dead tuples, repair neighbor arrays per ADR-027.
- [ ] **Implement `tqhnsw_amvacuumcleanup`** to finalize page compaction
  for both hot and cold payload pages.
- [ ] **Vacuum test.** Build a PqFastScan index, delete 10% of rows,
  VACUUM, confirm recall and validate page counts.
- [ ] **Remove `ADR030_GROUPED_V2_VACUUM_UNSUPPORTED`** reject path at
  `src/am/vacuum.rs:107`.

### Parameterization

- [ ] **Audit hardcoded defaults.** `ADR030_EXPERIMENTAL_GROUP_SIZE = 16`
  and `bits == 4` asserts across `src/am/build.rs` and
  `src/am/scan.rs`. Move to `PqFastScanLayout` fields or downgrade
  runtime `assert!` to `debug_assert!` where the value is already
  metadata-validated at scan open.
- [ ] **Codebook training determinism.** Make grouped k-means
  deterministic per `(dim, seed)` so corpus-scale regressions reproduce
  byte-for-byte across rebuilds. Flagged in task 14 reviewer feedback
  packets 310–333.

### Shared packing contract (from task 14 reviewer feedback)

- [ ] **Collapse duplicate grouped-code packing** between
  `src/am/build.rs` and `src/bin/approx_score_study.rs` into a shared
  module, or add a strong cross-path equality test that would catch
  divergence.

### Scan seam hygiene (advisory, not blocking)

- [ ] **Audit `GraphStorageDescriptor` match sites** across
  `src/am/scan.rs` (18 sites today). Not required for merge. If a third
  format is ever proposed, extract a `ScoringStrategy` trait before
  landing it.

### Docs

- [ ] **README** section on choosing a format. Rule of thumb: TurboQuant
  for small/medium indexes and simple ops; PqFastScan for NFR-001-
  critical workloads once measured.
- [ ] **Migration note.** Switching format = REINDEX. No auto-upgrade.

## Owns

- Execution of ADR-032.

## Dependencies

- Task 14 design, feasibility, and metadata contract (in progress →
  mostly done).
- ADR-026 lock-ordering rules for insert.
- ADR-027 lock-ordering rules for vacuum.

## Unblocks

- Task 16 (TurboQuant iteration). Deliberately comes after this task so
  TurboQuant speedups do not ride on PqFastScan polish.
- Any future decision to flip the default format to PqFastScan, once
  insert throughput and vacuum robustness are measured.

## Out of scope

- Flipping the default format (separate decision after measurement).
- OPQ transform front-end (ADR-030 follow-on).
- PQ8 rerank payload (ADR-030 follow-on).
- Extracting the scoring-strategy trait into a typed abstraction
  (advisory only).

## Definition of done

- [ ] `CREATE INDEX ... WITH (storage_format='turboquant')` and
  `...WITH (storage_format='pq_fastscan')` both succeed and pass the
  50k real seam recall harness.
- [ ] Insert + vacuum round-trip on both formats.
- [ ] No `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD` references remain in
  the tree.
- [ ] No `ADR030_GROUPED_V2_*_UNSUPPORTED` constants remain.
- [ ] Branch merges to `main`.
