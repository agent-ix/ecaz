# Task 54: Common P3 — Page / WAL / Buffer Typed Wrappers

Status: **complete** (2026-06-16) — Common P3 page/WAL/buffer
wrappers + the 006 HNSW insert/shared stretch landed on `main`
(merge `4b063a33b`). Closeout `reviews/task-54/005-closeout/`; final
acceptance `reviews/task-54/005-closeout/feedback/2026-06-16-01-reviewer.md`.
Was the third Phase-1 lane in the post-Task-50 hardening sequence.

## Why

Task 41 heavily processed `src/storage/` for PG resource RAII
(relation / buffer / snapshot / LWLock guards), but `src/storage/`
still carries **78** unsafe blocks, and the 448 closeout records HNSW
`build.rs`'s -18.2% ceiling at the page/WAL primitive layer:

> 2. Build-time page-mutation primitives: `write_data_pages`,
>    `flush_build_state_with_timing`, `flush_build_output`. These
>    compose `LockedBufferGuard::read_main` (still unsafe),
>    `wal::GenericXLogTxn::start` (still unsafe), `pg_sys::PageInit`
>    (still unsafe), and `pg_sys::PageAddItemExtended` (still unsafe).
>    They are the build-side counterparts to the page primitives
>    flagged for **program P3** (Buffer/Page/WAL contracts) AM-wide
>    rollout.

The same primitives back IVF build/vacuum, SPIRE storage, and
DiskANN build paths. P3 lifts those once into typed wrappers; AM
consumers across all four become safe call sites.

## Non-Goals

- Do not touch the `extern "C-unwind" fn` AM callback shells. Those
  remain `unsafe extern` per the PostgreSQL ABI and pgrx's callback
  registration contract — P1 surface, irreducible.
- Do not refactor WAL record format. Wrappers gate calls into
  `GenericXLogTxn::start` / `GenericXLogTxn::insert`; they do not
  change what is logged.
- Do not migrate IVF / SPIRE / DiskANN consumer call sites in this
  task. They will consume the wrappers under Tasks 55/56/57.
- Do not deprecate the existing `src/storage/locked_buffer.rs` /
  `src/storage/wal.rs` modules — extend them rather than replace.

## Scope

Add or extend typed wrappers in `src/storage/` (and `src/am/common/`
where the surface is AM-shared but not raw-storage):

1. **`PageInitGuard<'buf>`** — typed wrapper around `pg_sys::PageInit`
   for a freshly-acquired buffer. Constructor takes a
   `LockedBufferGuard` and the page size constant; on Drop it does
   nothing (page already initialized in PG arena). Single `unsafe fn
   init`; safe accessor `as_page_ptr_mut() -> *mut PageData` for
   downstream `PageAddItem` calls.
2. **`PageAddItemExtendedGuard<'page>`** — typed wrapper around
   `pg_sys::PageAddItemExtended`. Safe `add(payload: &[u8], flags:
   PageAddItemFlags) -> Result<OffsetNumber, PageAddItemError>`.
3. **`WalTxnScope<'rel>`** — typed RAII scope around
   `wal::GenericXLogTxn::start` + `register_buffer` calls + `finish`
   / `abort`. Drop performs `finish` if not explicitly committed;
   panic-safe abort.
4. **`BufferPinScope<'rel>`** — typed pin-only borrow over
   `ReadBufferExtended` (no lock) used by read-only scan paths that
   want stable buffer access without LWLock acquisition. RAII Drop
   releases the pin.
5. Extend **`LockedBufferGuard`** with additional typed methods
   (`read_main`, `write_main`, `read_aux`) that no longer require
   caller-side `unsafe { ... }` wraps where the underlying invariant
   is already encoded by the guard.

Each wrapper records its buffer/page/WAL lifetime invariant in its
constructor doc, same pattern as Task 50 P8 wrappers.

## Migration Targets

This task migrates **HNSW only** as the validating consumer. SPIRE /
IVF / DiskANN consumer migrations belong to their own subsystem
tasks.

| File | Surface | Expected block delta |
| --- | --- | ---: |
| `src/am/ec_hnsw/build.rs` (18 currently) | `write_data_pages`, `flush_build_state_with_timing`, `flush_build_output` | -6 to -10 |
| `src/am/ec_hnsw/vacuum.rs` (18 currently) | residual `WalTxnScope` / `PageInit` chain inside `apply_page_pass1_updates` and friends | -2 to -4 |
| `src/storage/locked_buffer.rs` (22 currently) | self-narrow remaining caller-required `unsafe` wraps | -4 to -8 |
| `src/storage/wal.rs` (separately counted) | `WalTxnScope` self-contains the start/finish unsafe | -2 to -4 |

**Targets**: `build.rs` 18 → ≤ 12 (-33%); `vacuum.rs` 18 → ≤ 14
(-22%); `locked_buffer.rs` 22 → ≤ 16 (-27%).

## Techniques

- Same patterns as Task 50 — single `unsafe fn` constructor, safe
  methods, lifetime-bound borrows.
- `WalTxnScope` panic-safe abort uses `std::panic::catch_unwind` only
  if absolutely required by the WAL semantics; prefer designing the
  state machine so Drop's path is straight-line.
- `PageAddItemExtendedGuard::add` returns `Result` rather than
  `OffsetNumber == InvalidOffsetNumber` panic sentinel — keeps caller
  code straight-line and avoids the open-coded
  `if offset == InvalidOffsetNumber { error!(...) }` repetition.

## Slice and Packet Rules

Same as Tasks 50 / 52 / 53. Specifically:

- Each packet must report `unsafe { ... }` block count before / after
  for every touched file, plus `src/` total.
- Wrapper-side blocks in `src/storage/` are counted but recorded as
  the intended category shift (per P3's disposition).

## Performance Gate

`build.rs` and `vacuum.rs` page-mutation paths are not on the scoring
hot path but are on build-time and vacuum-time critical paths.

Required evidence per slice:

- `ecaz bench latency` + `ecaz bench storage` on the post-Task-50 M5
  baseline corpus (`benchmarks/task-50-m5-hnsw-baseline/`) at the
  same prefixes and sweep, before/after.
- Build-time wall-clock measured via the `load` step's
  `corpus-load-*.log` (already captured by the suite).

Acceptance: regression tolerance is the same as Tasks 50/52/53. Build
time must not regress beyond 5% noise band.

## Validation

- `cargo fmt --all`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- focused `cargo pgrx test pg18` for `ec_hnsw::build`,
  `ec_hnsw::vacuum`, and `storage::locked_buffer` when behavior could
  plausibly drift
- direct unsafe-block count per touched file
- `src/` total snapshot

## Exit Criteria

Task closes when:

- The wrappers listed in §Scope exist in `src/storage/` (and
  `src/am/common/` where appropriate).
- `src/am/ec_hnsw/build.rs` ≤ 12; `vacuum.rs` ≤ 14;
  `src/storage/locked_buffer.rs` ≤ 16.
- HNSW recall + QPS + storage + build-time show no regression vs
  the post-Task-50 baseline.
- A closing summary packet records:
  - per-file before/after for `build.rs`, `vacuum.rs`,
    `locked_buffer.rs`, `wal.rs`;
  - the `src/storage/` and `src/am/common/` wrapper surface added;
  - the `src/` total block count change;
  - explicit handoff list naming each SPIRE / IVF / DiskANN consumer
    site that the new wrappers will absorb under Tasks 55/56/57.

## Coordination

- Phase-1 lane — runs after Tasks 52 (P8) and 53 (P6). Order matters:
  P3 page/WAL touches storage primitives that some of the consumer
  call sites in Tasks 52/53 might already be near; landing P3 last
  in Phase 1 lets the previous lanes' consumer migrations stay
  stable.
- Coordinate with Task 41 (historic storage RAII) — extend rather
  than replace existing guards.
- Coordinate with Task 51 (IVF RaBitQ optimization): no overlap
  expected since IVF page primitives are not refactored here, only
  named in the handoff list.
- Reviewer scope-lock: HNSW-only consumer migration on this branch.

## Cross-References

- Supersedes `reviews/task-50/030-comprehensive-unsafe-burndown-plan`
  §P3 disposition.
- Closes the `build.rs` ceiling documented in
  `reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md`
  §"`build.rs` ceiling".
- Builds on Task 41's storage RAII work in `src/storage/`.
- Bench gate consumes
  `benchmarks/task-50-m5-hnsw-baseline/manifest.md` as the pre-state.
