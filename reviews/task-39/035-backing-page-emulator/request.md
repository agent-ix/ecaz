# Task 39 / 035 — Backing-Page Emulator for ec_spire/page + relation_store

## Goal

Close priority #2 from the 033 reviewer feedback: replace the no-op page-level
stubs in `hardening/careful/src/pg_guards.rs::pg_sys` with a real
backing-page emulator so the success paths in
`src/am/ec_spire/page.rs` and `src/am/ec_spire/storage/relation_store.rs`
are exercisable from the careful shadow crate. The reviewer's Phase-1
shape — per-Buffer `[u8; 8192]` + stable ItemId table + real
`PageInit`/`PageAddItemExtended`/`PageGetItem`/`PageGetItemId`/
`PageGetMaxOffsetNumber`/`PageGetFreeSpace`/`PageGetSpecialPointer`/
`PageGetSpecialSize`/`PageIndexTupleDeleteNoCompact` — is implemented in
full and wired through `ReadBufferExtended` (P_NEW allocation),
`BufferGetPage`, `BufferGetBlockNumber`, and
`RelationGetNumberOfBlocksInFork`.

## Code Change

`hardening/careful/src/pg_guards.rs`

- New `BackingPage` (real `Box<[u8; 8192]>` bytes, `Vec<Box<ItemIdData>>`
  line pointers with stable heap addresses, tuple low-water mark, special
  area at end-of-page) and `BufferRegistry` thread-local indexed by
  `(rd_id, block_number)` with reverse `*mut u8 -> *mut BackingPage`
  lookup so the page helpers find the same backing struct the buffer
  guard pinned.
- `ReadBufferExtended`: `MAX + RBM_ZERO_AND_LOCK` is now the real P_NEW
  allocation path (`allocate_block` + `pin_buffer`); for non-null
  relations any block number is allocated-up-to lazily so existing tests
  that pre-pin arbitrary blocks keep working. `MAX + other modes` and
  null relation still return `InvalidBuffer`, preserving the prior
  guard-test contract.
- `BufferGetPage` / `BufferGetBlockNumber` source from the registry,
  falling back to the legacy `block + 1` mapping for synthetic Buffer
  ids that bypass `ReadBufferExtended` (the original
  `PinnedBufferGuard::from_pinned(5)` style tests).
- `RelationGetNumberOfBlocksInFork` now returns the registry's
  per-OID block count (0 for null relations) — the legacy
  `set_relation_block_count` hook becomes a backwards-compatible no-op
  so packets 033-034 keep compiling without behavior change.
- `reset_counters()` now also `reset_buffer_registry()` so every test
  that already serialises through `TEST_LOCK + reset_counters` (or the
  new emulator tests) starts with a clean registry.
- `PageInit` / `PageAddItemExtended` / `PageGetItem` / `PageGetItemId`
  / `PageGetMaxOffsetNumber` / `PageGetFreeSpace` /
  `PageGetSpecialPointer` / `PageGetSpecialSize` /
  `PageIndexTupleDeleteNoCompact` are real implementations against
  `BackingPage`, sized via the same `raw_tuple_storage_bytes` accounting
  the production helpers use.
- Updated `locked_buffer_read_main_variants_wrap_read_buffer` to assert
  the null-relation early-out for `RBM_ZERO_AND_LOCK + MAX` (which is
  now the legitimate P_NEW path) instead of the previous synthetic
  early-out behaviour the no-op stub forced.

`hardening/careful/src/spire.rs`

- New `page_tests::initialize_and_read_root_control_round_trip_through_emulator`,
  `append_and_read_object_tuple_round_trip_through_emulator`,
  `scan_object_tuples_visits_every_appended_tuple_in_order`,
  `rewrite_object_tuple_same_len_updates_payload_in_place`, and
  `delete_object_tuples_no_compact_removes_real_tuples_and_reports_bytes`
  round-trip through the emulator-backed P_NEW append and share-lock
  read paths in `src/am/ec_spire/page.rs`. They cover the previously
  unreachable WAL + buffer + page success branches that the no-op stubs
  blocked.
- New `storage::tests::relation_store_insert_and_read_{routing,leaf_v2,delta,top_graph}_round_trip`
  exercise `SpireRelationObjectStore::insert_*` + `read_*` +
  `read_object_header` dispatch through the emulator for all four
  partition-object kinds.

## Baseline Ratchets

`fixtures/quality/coverage-baseline.tsv`

| File | Pre-packet | This packet |
| --- | ---: | ---: |
| `am/ec_spire/page.rs` | 11.01 | **81.12** |
| `am/ec_spire/storage/relation_store.rs` | 3.98 | **27.20** |

`am/ec_spire/page.rs` crosses the 80% floor. `am/ec_spire/storage/
relation_store.rs` rises 6.8× via four direct round-trips through the
emulator; remaining uncovered surface is the large-object chain /
chain-meta + auxiliary read helpers, which need follow-up packets
(separate from this Phase-1 emulator slice).

## Validation

All artifacts under
`reviews/task-39/035-backing-page-emulator/artifacts/`:

- `backing-page-emulator-focused-tests.log` —
  `cargo test --manifest-path hardening/careful/Cargo.toml --lib` →
  **472 passed, 0 failed** (was 468 after packet 034).
- `coverage/summary.txt` + `coverage/coverage.json` +
  `coverage/careful-coverage.json` from
  `make coverage COVERAGE_OUTPUT_DIR=…` against the same head.
  Relevant lines:
  - `am/ec_spire/page.rs 445/84 81.12%`
  - `am/ec_spire/storage/relation_store.rs 1408/1025 27.20%`
- `coverage-delta-check.log` —
  `scripts/check_coverage_delta.sh … changed-files.txt`:
  both touched files green at the new baseline.
- `coverage-baseline-check.log` —
  `scripts/check_coverage_baseline_complete.sh`:
  **40 critical paths complete.**
- `changed-files.txt` — the two source paths whose baseline this packet
  ratchets.

Both code change and packet are pushed to
`origin/task39-continuation-20260519`. Code commit landed before this
packet commit per the workflow rules.

## Reviewer Direction

- Confirm the emulator is the right Phase-1 shape: stable ItemId
  pointers via `Vec<Box<ItemIdData>>`, raw bytes via
  `Box<[u8; 8192]>`, and the registry handles P_NEW + legacy
  buffer-id callers without breaking existing pg_guards tests.
- Confirm `page.rs` at 81.12% closes priority #2 from packet 033 so the
  next slice can move to priority #3 (Task 47 enforcement). The
  remaining ~19% in page.rs is dominated by `pgrx::error!` panics on
  rare failure paths that the careful crate stubs as `panic!`; pushing
  past the floor would require panic-catching tests rather than
  emulator extension.
- Flag whether `relation_store.rs` deserves Phase-2 emulator follow-up
  (FSM-backed `GetPageWithFreeSpace` + chain semantics) to lift the
  remaining ~73%, or whether that slice waits behind priority #3.
