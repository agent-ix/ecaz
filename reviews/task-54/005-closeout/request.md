# Task 54 Packet 005 — Closeout

Status: **proposed**

Final closing summary for Task 54 (P3 page/WAL/buffer typed
wrappers). Covers per-file before/after, the `src/storage/` +
`src/am/common/` wrapper surface, the `src/` total block-count
delta, the structural ceiling decision for `src/storage/wal.rs`,
and the SPIRE / IVF / DiskANN consumer handoff list.

## §Exit Criteria summary

### #1 — Wrappers exist in `src/storage/` (+ `src/am/common/` where appropriate)

| Wrapper | Module | Form | Packet |
| --- | --- | --- | --- |
| `WalTxnScope<'rel>` | `src/storage/wal.rs` | typed RAII scope; `start` (unsafe-fn raw-ptr) + `start_handle` (safe `RelationHandle`); `register_page` / `finish` safe ops | 002 |
| `RegisteredBufferPage::init` (`PageInitGuard<'buf>`) | `src/storage/wal.rs` | safe `init(special_size)` on the WAL-registered page | 002 |
| `RegisteredBufferPage::add_item` + `PageAddItemError` (`PageAddItemExtendedGuard<'page>`) | `src/storage/wal.rs` | safe `add_item(payload) -> Result<OffsetNumber, PageAddItemError>` | 002 |
| `BufferPinScope<'rel>` (alias) | `src/storage/buffer_guard.rs` | in-repo equivalent is the existing `PinnedBufferGuard` (pin-only RAII); the `'rel` lifetime is encoded by the existing "live relation" SAFETY contract | n/a (pre-Task-54) |
| `LockedBufferGuard::read_main_handle` / `read_main_locked_handle` | `src/storage/buffer_guard.rs` | safe `_handle` constructors taking `RelationHandle` | 002 |
| `shared::initialize_metadata_page_handle` | `src/am/ec_hnsw/shared.rs` | safe handle variant of `initialize_metadata_page` | 003 |
| `shared::with_locked_metadata_page_handle` | `src/am/ec_hnsw/shared.rs` | safe handle variant of `with_locked_metadata_page` | 004 |

§Exit #1: **met**.

### #2 — Per-file unsafe block counts

| File | Pre-Task-54 | Post-Task-54 | Δ | §Exit target | Status |
| --- | ---: | ---: | ---: | ---: | --- |
| `src/am/ec_hnsw/build.rs` | 18 | **11** | -7 | ≤ 12 | **met (+1 margin)** |
| `src/am/ec_hnsw/vacuum.rs` | 18 | **13** | -5 | ≤ 14 | **met (+1 margin)** |
| `src/storage/buffer_guard.rs` (= task-plan `locked_buffer.rs`) | 22 | **15** | -7 | ≤ 16 | **met (+1 margin)** |
| `src/storage/wal.rs` | 5 | **13** | +8 | (see §wal.rs ceiling) | structural-ceiling re-baselined |
| `src/am/ec_hnsw/shared.rs` (stretch) | 21 | 21 | 0 | — | unchanged |
| `src/am/ec_hnsw/insert.rs` (stretch) | 25 | 25 | 0 | — | unchanged |
| `src/` total | 960 | **949** | **-11** | — | net category shift absorbed |

All three §Exit Criterion #2 hard targets (`build.rs`, `vacuum.rs`,
`buffer_guard.rs`) are met with margin. The `wal.rs` count is the
intended category shift — see §wal.rs ceiling below.

### #3 — HNSW recall + QPS + storage + build-time vs post-Task-50 baseline

Bench gate result (full 8-step `ecaz bench suite` against
`benchmarks/task-50-m5-hnsw-baseline/suite.json`):

| Lane | 10k | 100k |
| --- | --- | --- |
| Recall@10 vs baseline | bit-for-bit identical (5 ef buckets) | inside ci95 (5 ef buckets, ‖Δ‖ ≤ 0.0034 per worker-sched jitter) |
| Latency (p50 / p95) vs baseline | -1.1 to -5.3% (p50), -0.9 to -8.1% (p95) | **-11.7 to -16.7%** (p50), -9.0 to -14.4% (p95) |
| Per-row storage bytes vs baseline | bit-for-bit identical (1366.4 / 1235.4 B) | bit-for-bit identical (1365.4 B m=16) |
| Build wall-clock vs baseline | m=8 -3.4%, m=16 -7.8% | m=16 -6.6% |

All within or improving over the 5%-ish noise band; no regression on
any lane. Detail tables in
`reviews/task-54/005-closeout/artifacts/before-after-summary.md`.

Acceptance tolerance: same as Tasks 50 / 52 / 53 — no regression
beyond the established 5 % noise band on build wall-clock and the
recall/latency thresholds documented in
`benchmarks/task-50-m5-hnsw-baseline/manifest.md`.

Artifacts under `reviews/task-54/005-closeout/artifacts/`:

- `corpus-load-ec_real_10k-hnsw.log`
- `recall-ec_real_10k-hnsw.log`
- `latency-ec_real_10k-hnsw.log`
- `storage-ec_real_10k-hnsw.log`
- `corpus-load-ec_real_100k-hnsw.log`
- `recall-ec_real_100k-hnsw.log`
- `latency-ec_real_100k-hnsw.log`
- `storage-ec_real_100k-hnsw.log`
- `results.jsonl` (suite results)
- `suite-manifest.json`
- `manifest.md`
- `before-after-summary.md`

(See §Bench run section for the manifest and commands.)

### #4 — Closing summary with handoff list

Embedded below; see §Wrapper surface summary and §SPIRE/IVF/DiskANN
consumer handoff list.

## §wal.rs ceiling — structural ceiling rationale

Per `reviews/task-54/003-hnsw-build-migration/feedback/2026-05-23-01-reviewer.md`
and `reviews/task-54/004-hnsw-vacuum-and-buffer-narrow/feedback/2026-05-23-01-reviewer.md`,
the slice-001 plan target of `src/storage/wal.rs ≤ 3` is unreachable
post-wrapper. The current 13 blocks consist of 9 PG-extern boundary
calls (the irreducible WAL/page primitive surface) plus 4 doc-comment
occurrences of `unsafe {` that raw `grep -c` matches:

| Block | Operation | Category |
| --- | --- | --- |
| L74 | `pg_sys::GenericXLogStart(relation)` | PG-extern boundary |
| L112 | full-image page registration (`GenericXLogRegisterBuffer`) | PG-extern boundary |
| L131 | `pg_sys::GenericXLogFinish(self.state)` | PG-extern boundary |
| L151 | `visit_tuple_bytes_mut` page-walk | PG-extern boundary |
| L202 | `pg_sys::GenericXLogAbort(self.state)` (Drop) | PG-extern boundary |
| L219 | `RegisteredBufferPage::init` (`pg_sys::PageInit`) | wrapper-internal (P3 absorption) |
| L234 | `RegisteredBufferPage::add_item` (`pg_sys::PageAddItemExtended`) | wrapper-internal (P3 absorption) |
| L289 | `WalTxnScope::start` (raw-ptr) → `GenericXLogTxn::start` | wrapper-internal (P3 absorption) |
| L304 | `WalTxnScope::start_handle` (safe) → `Self::start` | wrapper-internal (P3 absorption) |

Per the task plan §"Slice and Packet Rules" — *"Wrapper-side blocks
in `src/storage/` are counted but recorded as the intended category
shift (per P3's disposition)"* — these 4 wrapper-internal blocks are
the *intended* category shift; the 5 boundary blocks plus the 4 new
P3 absorptions are the irreducible floor for this module.

**Decision (option 1, per reviewer recommendation matching Task
50/448 precedent)**: re-baseline `wal.rs` target to ≤ 13 with the
structural-ceiling rationale above. The task spec §Migration Targets
expressed the wal.rs target as a *range* (`-2 to -4`) rather than an
absolute, so the slice 001 plan's restating as `≤ 3` was over-tight
against the pre-wrapper baseline of 5. The +8 wrapper additions in
packet 002 (intentional category shift) and the absorption by HNSW
consumer migrations in packets 003/004 deliver the net `src/` total
benefit (960 → 949 = -11) that the task is structured around.

Cross-reference: `reviews/task-50/448-hnsw-burndown-refreshed-closeout/`
documented the analogous structural-ceiling disposition for
`src/quant/rabitq.rs` SIMD intrinsic blocks and the FromDatum boundary.

## §Wrapper surface summary

### `src/storage/wal.rs`

- `WalTxnScope<'rel>` — typed RAII WAL transaction scope.
  - `unsafe fn start(relation: pg_sys::Relation)` (raw-pointer entry)
  - `fn start_handle(handle: RelationHandle)` (safe handle entry)
  - safe `register_page(&mut self, &LockedBufferGuard) -> RegisteredBufferPage`
  - safe `finish(self) -> XLogRecPtr`
- `RegisteredBufferPage<'txn, 'buffer>` — extended with:
  - safe `page_ptr(&self) -> pg_sys::Page`
  - safe `init(&mut self, special_size: usize)` — wraps `PageInit`
  - safe `add_item(&mut self, payload: &[u8]) -> Result<OffsetNumber, PageAddItemError>` — wraps `PageAddItemExtended`
- `PageAddItemError { block_number }` — carries diagnostic context.

### `src/storage/buffer_guard.rs`

- `LockedBufferGuard::read_main_handle(handle: RelationHandle, ...)` — safe variant.
- `LockedBufferGuard::read_main_locked_handle(handle: RelationHandle, ...)` — safe variant.
- Self-narrow: `read_main`, `read_main_locked`, `lock_pinned`, and
  `PinnedBufferGuard::from_pinned` interior unsafe blocks consolidated
  to one block each.

### `src/am/ec_hnsw/shared.rs`

- `initialize_metadata_page_handle(handle: RelationHandle, metadata)` — safe variant.
- `with_locked_metadata_page_handle(handle: RelationHandle, FnOnce)` — safe variant.

## §SPIRE / IVF / DiskANN consumer handoff list

These call sites will absorb the P3 wrappers under Tasks 55 (IVF),
56 (DiskANN), and 57 (SPIRE) per the post-Task-50 hardening sequence.

### IVF — `src/am/ec_ivf/`

| Site | Surface to consume | Notes |
| --- | --- | --- |
| `page.rs:218` `read_main` | `LockedBufferGuard::read_main_handle` | wrapper around index relation already a handle |
| `page.rs:229` `read_main_locked` | `LockedBufferGuard::read_main_locked_handle` | same |
| `page.rs:234` `GenericXLogTxn::start` | `wal::WalTxnScope::start_handle` | same |
| `page.rs:506` `pg_sys::PageInit` | `RegisteredBufferPage::init` | via `WalTxnScope::register_page` |
| `page.rs:523` `pg_sys::PageAddItemExtended` | `RegisteredBufferPage::add_item` | same |
| `build.rs:589-607` `LockedBufferGuard::read_main_locked` + `GenericXLogTxn::start` + `PageInit` + `PageAddItemExtended` chain | full P3 chain | mirrors HNSW `write_data_pages` pattern |

### DiskANN — `src/am/ec_diskann/`

| Site | Surface to consume | Notes |
| --- | --- | --- |
| `insert.rs:102,111,116` `read_main` / `read_main_locked` / `GenericXLogTxn::start` | handle variants | mirrors HNSW insert.rs pattern |
| `insert.rs:1246,1414,1424` `PageInit` + `PageAddItemExtended` | `RegisteredBufferPage::init/add_item` | |
| `ambuild.rs:716-790` full chain (read_main_locked + GenericXLogTxn + PageInit + PageAddItemExtended) | full P3 chain | mirrors HNSW `write_data_pages` |
| `routine.rs:1531,1561,1605,1621` `LockedBufferGuard::read_main` + `wal::GenericXLogTxn::start` | handle variants | |
| `scan_state.rs:148,192` `LockedBufferGuard::read_main` | `read_main_handle` | scan path, read-only |

### SPIRE — `src/am/ec_spire/`

| Site | Surface to consume | Notes |
| --- | --- | --- |
| `page.rs:69` `wal::GenericXLogTxn::start` | `WalTxnScope::start_handle` | |
| `page.rs:104,116` `PageInit` | `RegisteredBufferPage::init` | |
| `page.rs:131` `PageAddItemExtended` | `RegisteredBufferPage::add_item` | |

### HNSW (stretch — not in this task's §Migration Targets)

| Site | Disposition |
| --- | --- |
| `src/am/ec_hnsw/insert.rs` (PageInit / PageAddItemExtended / GenericXLogTxn::start at multiple sites) | local `PageWriter` helper internalizes the chain; could be migrated to wrappers in a follow-up, but per §Non-Goals scope-locked to validating consumer only |
| `src/am/ec_hnsw/shared.rs` (`rewrite_metadata_buffer` PageInit + GenericXLogTxn chain) | same — stretch consumer, not migrated |

## §Bench run

Manifest (filled by closeout suite run):

- HEAD SHA: `cd7fe728b` (slice-005 reviewer commit; bench artifacts captured against the extension installed from this HEAD)
- Host: Peters-MBP (Apple Silicon M5 Pro, 64 GiB, macOS 26.4.1)
- PostgreSQL: 18 (pgrx local install, socket `/Users/peter/.pgrx`, port 28818)
- Extension build: `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config`

Command:

```sh
/Users/peter/.cargo/bin/ecaz \
  --host /Users/peter/.pgrx --port 28818 --database tqvector_bench \
  bench suite run \
  --config benchmarks/task-50-m5-hnsw-baseline/suite.json \
  --artifact-dir reviews/task-54/005-closeout/artifacts \
  --log-file reviews/task-54/005-closeout/artifacts/suite-run.log
```

Artifact layout matches Task 53's closeout
(`reviews/task-53/004-closeout/artifacts/`).

## §Validation

- `cargo fmt --all` — passes (no diff against committed state).
- `cargo check --all-targets --no-default-features --features pg18,bench` — passes at HEAD.
- `cargo clippy --no-default-features --features pg18 --lib -- -D warnings` — exits 101 with 102 pre-existing repo-wide clippy lint failures. **Three** of those land in this task's touched files; all three are pre-existing per `git blame`:
  - `src/am/ec_hnsw/build.rs:2253` (`.into_iter()` on `Vec`) — `d1d9a16564`, 2026-04-19, pre-Task-54.
  - `src/am/ec_hnsw/shared.rs:151` (`&buffer` instead of `buffer`) — `cdfe24f198`, 2026-05-20, pre-Task-54 (inside `rewrite_metadata_buffer`, not touched by this packet series).
  - `src/am/ec_hnsw/shared.rs:565` (`let metadata = ...; metadata`) — `116935ff20`, 2026-04-05, pre-Task-54 (`read_metadata_page` body, not touched).
  
  Task 54 introduced ZERO new clippy warnings. The `drop(page)` lint that surfaced during slice 004 was a Task-54-introduced regression that the same slice fixed before commit (see `src/am/ec_hnsw/build.rs` block-scope refactor + `src/am/ec_hnsw/vacuum.rs` `register_page().page_ptr()` chain in `f22628100`).
- The repo-wide pre-existing clippy debt is out of scope for Task 54 per the HNSW-only consumer migration scope-lock and `feedback_no_scope_padding`. A future hardening task can land the codebase-wide clippy cleanup as its own scope.

## References

- `plan/tasks/54-common-p3-page-wal-wrappers.md` §Exit Criteria, §Slice and Packet Rules.
- `reviews/task-54/001-execution-plan/request.md` (slice plan).
- `reviews/task-54/002-p3-wrappers/request.md` + feedback (wrapper surface, +8 wal.rs accounting erratum).
- `reviews/task-54/003-hnsw-build-migration/request.md` + feedback (build.rs migration, wal.rs forward flag).
- `reviews/task-54/004-hnsw-vacuum-and-buffer-narrow/request.md` + feedback (vacuum.rs + buffer_guard.rs migration, structural-ceiling rationale).
- `benchmarks/task-50-m5-hnsw-baseline/manifest.md` (pre-state baseline).
- `reviews/task-50/448-hnsw-burndown-refreshed-closeout/request.md` §"Structural-ceiling documentation" (precedent).
- `reviews/task-53/004-closeout/` (bench gate precedent).
