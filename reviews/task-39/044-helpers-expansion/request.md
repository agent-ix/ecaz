# Task 39 / 044 — Helpers Expansion + Emulator Phase-2 Hooks

## Goal

Close more of the 0% surface on `coordinator/diagnostics.rs` and
`ec_diskann/routine.rs` by extracting additional pure helpers, and
unlock the `relation_store.rs` chain-corruption and PG18-prefetch
branches via a Phase-2 emulator hook (test-only raw-byte page writer
and a queued `read_stream_next_buffer`).

## Code Change

### Helpers extracts

`src/am/ec_spire/coordinator/diagnostics_helpers.rs`:

- New `epoch_cleanup_blocked_reason(&SpireEpochManifest, …)` moved out
  of `diagnostics.rs`. Body byte-for-byte identical.

`src/am/ec_diskann/routine_helpers.rs`:

- New `count_live_tuples_in_chain`, `collect_node_tids`,
  `read_chain_node`, `write_chain_node`, `collect_tuple_rewrites`, and
  `expand_scan_results_with_bound_heap_tids` plus the `TupleRewrite`
  struct, all extracted from `routine.rs`. Bodies byte-for-byte
  identical.

`src/am/ec_spire/coordinator/diagnostics.rs` and
`src/am/ec_diskann/routine.rs`:

- Helpers above removed in favor of the existing `include!` of the
  sibling helpers file.

### Phase-2 emulator hooks

`hardening/careful/src/pg_guards.rs`:

- New `pg_sys::set_raw_tuple_bytes_for_test(rd_id, block, offset,
  bytes)` overwrites a known item's raw bytes on a BackingPage. This
  is the only path to crafted chain-corruption scenarios that the
  public insert API cannot produce.
- New `pg_sys::enqueue_read_stream_blocks_for_test(blocks)` queues
  `(rd_id, block_number)` entries that subsequent
  `read_stream_next_buffer` calls will surface.
- `read_stream_next_buffer` now drains the queue and pins each
  registered block before returning, falling back to `InvalidBuffer`
  when the queue is empty.

### Careful-side scaffolds

`hardening/careful/src/spire_diagnostics_helpers.rs`:

- Adds a `meta::SpireEpochManifest` shim (epoch, state, consistency
  mode, published/retain micros, active_query_count) so the new
  `epoch_cleanup_blocked_reason` helper compiles.
- New test
  `miri_epoch_cleanup_blocked_reason_walks_every_branch` covers all 8
  match arms.

`hardening/careful/src/diskann_routine_helpers.rs`:

- Adds a `scan::ScanResult` shim and an `insert::bound_heap_tids_for_owner`
  shim (returns just the primary heap tid).
- 7 new tests cover `count_live_tuples_in_chain`,
  `collect_node_tids`, `read_chain_node`, `write_chain_node` (happy +
  unknown-block error), `collect_tuple_rewrites` (empty / changed /
  page-count / block / tuple-count mismatch), and
  `expand_scan_results_with_bound_heap_tids` (happy + top_k cap).

`hardening/careful/src/spire.rs`:

- 3 new emulator-driven tests in `storage::tests` exercise relation
  store chain-error branches and PG18 prefetch:
  - `relation_store_chain_segment_decode_rejects_segment_number_mismatch`
    overwrites the segment header so chain-decode surfaces an error.
  - `relation_store_chain_walker_rejects_corrupted_byte_base`
    overwrites the segment's byte_base field.
  - `relation_store_prefetch_drains_read_stream_with_buffered_blocks`
    pre-populates the read-stream queue so the PG18 prefetch loop
    iterates instead of returning `InvalidBuffer` immediately.

## Baseline Ratchets

`fixtures/quality/coverage-baseline.tsv`:

| File | Pre-packet | This packet |
| --- | ---: | ---: |
| `am/ec_spire/coordinator/diagnostics_helpers.rs` | 100.00 (199 lines) | **100.00 (220 lines)** |
| `am/ec_diskann/routine_helpers.rs` | 100.00 (11 lines) | **100.00 (127 lines)** |
| `am/ec_spire/storage/relation_store.rs` | 58.52 | **58.66** |

Net: ~136 additional production lines now exercise under
`make coverage` (without changing production behavior — the helpers
file body is identical to what they replaced).

`diagnostics.rs` and `routine.rs` line counts shrink as the helpers
move out; their coverage baseline stays at 0.00% (the recorded
baseline tracks the remaining pgrx-bound body, which is still gated
on live PG18 coverage per `docs/hardening.md`).

## Validation

Artifacts under `reviews/task-39/044-helpers-expansion/artifacts/`:

- `helpers-expansion-focused-tests.log`: **528 passed**, was 513 (+15).
- `coverage/summary.txt` + JSON files from `make coverage`.
- `coverage-delta-check.log`: every ratcheted row green.
- `coverage-baseline-check.log`: **42 critical paths complete**.
- Production `cargo check --features pg18 --no-default-features`
  clean (extracted bodies identical; emulator helpers are
  `pg_sys`-level test hooks not consumed by production).

## Reviewer Direction

Two follow-ups that would let the remaining pgrx-bound bodies be
covered without writing more careful-side shims:

1. Fix the macOS PG18 `dyld _BufferBlocks` instrumentation block so
   `cargo pgrx test pg18 --features instrument-coverage` actually
   runs; once it does, the pgrx-bound bodies of `diagnostics.rs`,
   `routine.rs`, and the remaining ~22% of `relation_store.rs` start
   reporting non-zero baselines from the live backend.
2. Failing that, the `set_raw_tuple_bytes_for_test` /
   `enqueue_read_stream_blocks_for_test` pattern in this packet
   generalises to any other emulator-driven test that needs to walk
   crafted-state or PG18 read-stream branches. The next slice that
   wants to push `relation_store.rs` past 80% should reuse those
   helpers for the remaining leaf V2 chain-corruption branches.
