# Task 39 / 036 — Coverage Pushes for vec_id, leaf_v2, local_store_set

## Goal

Raise the remaining three SPIRE storage files that were sitting below
the 80% line-coverage floor at the start of this session, using direct
unit tests in the careful crate (no new shadow scaffolding needed —
these already compile in the existing storage module).

## Code Change

- `src/am/ec_spire/storage/tests/vec_and_routing.rs`: 5 new tests cover
  the previously-unreached `SpireVecIdKind::decode` error branch,
  `SpireVecIdRef` (`from_bytes`, `as_bytes`, `discriminator`,
  `local_sequence` both Some-for-local and None-for-global,
  `to_owned`) via `SpireLeafAssignmentRow::decode_prefix_ref`,
  the assignment-row and leaf-V2 segment layout const-fn helpers,
  and `SpireLeafObjectColumns::row` (happy path with `row_base`
  offset, out-of-range row offset error).
- `src/am/ec_spire/storage/tests/leaf.rs`: 7 new tests construct
  `SpireLeafPartitionObjectV2` directly via struct literals and drive
  `column_segments()` / `assignment_rows()` so the `validate()` error
  branches in `leaf_v2.rs` are individually observable: segment count
  mismatch, segment number mismatch, row_base mismatch, final segment
  with non-invalid locator, non-final segment missing locator, meta
  assignment-count mismatch, and the happy round-trip back to rows.
- `src/am/ec_spire/storage/tests/local_store.rs`: 4 new tests cover
  the `SpireLocalObjectStoreSet::from_config` duplicate-`local_store_id`
  guard, the previously-unreached `SpireObjectReader for
  SpireLocalObjectStoreSet::read_leaf_object` (V1) delegate, and full
  trait-dispatch coverage for the impl blocks at lines 115-163
  (`SpireObjectReader for SpireLocalObjectStoreSet`) and 165-207
  (`SpireObjectReader for SpireLocalObjectStore`).

## Baseline Ratchets

`fixtures/quality/coverage-baseline.tsv`:

| File | Pre-packet | This packet |
| --- | ---: | ---: |
| `am/ec_spire/storage/vec_id.rs` | 69.64 | **94.64** |
| `am/ec_spire/storage/leaf_v2.rs` | 71.76 | **95.29** |
| `am/ec_spire/storage/local_store_set.rs` | 63.74 | **88.89** |

All three cross the 80% floor in a single packet without any shadow
scaffolding changes — the tests run directly against the production
source via the storage module's existing include path.

## Validation

Artifacts under `reviews/task-39/036-coverage-pushes/artifacts/`:

- `coverage-pushes-focused-tests.log` — `cargo test --manifest-path
  hardening/careful/Cargo.toml --lib` → **488 passed, 0 failed**
  (was 472 after packet 035; 16 new tests in this packet).
- `coverage/summary.txt` + `coverage/coverage.json` +
  `coverage/careful-coverage.json` from `make coverage`.
- `coverage-delta-check.log` —
  `scripts/check_coverage_delta.sh … changed-files.txt`: all three
  ratcheted files green at the new baseline.
- `coverage-baseline-check.log` —
  `scripts/check_coverage_baseline_complete.sh`:
  **40 critical paths complete.**
- `changed-files.txt` — the three source paths whose baselines this
  packet ratchets.

Code commit and packet commit pushed separately to
`origin/task39-continuation-20260519` per the workflow rules.

## Reviewer Direction

Three of the four below-floor SPIRE storage files now exceed the 80%
target with this packet. Remaining gaps tracked by adjacent packets:

- `am/ec_spire/storage/relation_store.rs` at 27.20%. Needs the Phase-2
  emulator FSM follow-up (planned next packet) to lift the large-object
  chain and FSM-backed allocation paths.
- `am/ec_spire/coordinator/diagnostics.rs` at 0%. Requires a larger
  scaffold packet because the file is `include!`'d into `ec_spire/
  mod.rs` and pulls coordinator/types.rs, quantizer::*, and per-snapshot
  row types — out of scope for this packet, on the queue.
- `am/ec_diskann/routine.rs` at 0%. 4291 lines using `pgrx::PgBox`,
  `pgrx::FromDatum`, `pgrx::PgMemoryContexts`, and `extern "C-unwind"`
  callbacks; needs substantial new pgrx-bindings shadow types before it
  compiles in the careful crate.
