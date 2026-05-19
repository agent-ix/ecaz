# Review Request: Master Unsafe Burndown Execution Plan

Head: `7732bb4d09272105e4b95ebb033db65570cb487b`

Scope:
- `reviews/task-35/004-master-unsafe-burndown-plan/request.md`
- `reviews/task-35/004-master-unsafe-burndown-plan/artifacts/*`

What changed:
- Added a task-local master plan for burning down the remaining unsafe-comment
  baseline.
- Captured a current unsafe survey so later packets can be sliced by subsystem
  and risk rather than by arbitrary file order.
- Identified an immediate blocker: `bash scripts/check_unsafe_comments.sh`
  does not pass on this checkout because the line-number baseline has drifted
  from the current `rg 'unsafe\s*\{' src` scan.

Current state:
- Baseline file count: 3,686 entries across 106 files.
- Current missing-SAFETY scan: 3,657 entries across 107 files.
- `unsafe { ... }` blocks under `src`: 3,778.
- `// SAFETY:` comments under `src`: 138.
- Baseline/current drift: 1,596 current missing entries are not in the baseline
  by exact `file:line`; 1,625 baseline entries no longer match current
  missing lines.

Largest current missing-SAFETY buckets:

| Bucket | Entries |
| --- | ---: |
| `src/am/ec_hnsw` | 1,299 |
| `src/am/ec_spire` | 886 |
| `src/tests` | 499 |
| `src/am/ec_ivf` | 326 |
| `src/am/ec_diskann` | 230 |
| `src/lib.rs` | 181 |
| `src/am/common` | 152 |
| `src/quant` | 74 |
| `src/am/mod.rs` | 10 |

Execution strategy:

1. Restore the audit gate before ordinary burndown.
   - Create a narrow normalization packet that explains the line-number drift,
     records the semantic before/after scan, and updates
     `scripts/unsafe_comment_baseline.txt` only to match current missing
     unsafe blocks.
   - This packet must explicitly call out that it adds exact `file:line`
     entries while reducing the net count from 3,686 to 3,657. That is
     bookkeeping for a stale line-based baseline, not acceptance of new unsafe
     sites.
   - Do not proceed with regular burndown packets until
     `bash scripts/check_unsafe_comments.sh` passes again.

2. Burn production entrypoints and wrappers before test-only surfaces.
   - Prefer deletion or safe wrappers over comments.
   - Use comments only when the unsafe operation is inherently the local
     PostgreSQL or SIMD boundary.
   - Keep each packet in the Task 35 target range of roughly 100-300 baseline
     entries unless a subsystem boundary demands a smaller review.

3. Reorganize unsafe by boundary type, not only by file.
   - PostgreSQL callback boundary: keep `#[pg_guard]` on `extern "C-unwind"`
     entrypoints and move body logic into safe internal helpers when possible.
   - Relation/snapshot/buffer/page boundary: route repeated operations through
     `src/storage/*_guard.rs` or subsystem-local wrappers that encode pin,
     lock, page size, tuple offset, and relation lifetime invariants.
   - Plan tree and catalog boundary: introduce typed helpers for `Query`,
     `List`, `RangeTblEntry`, `Relation`, `TupleDesc`, and relcache hook state
     instead of restating raw pointer assumptions at every dereference.
   - DSM/shared-memory boundary: isolate layout arithmetic, slot claiming, and
     `Send`/`Sync` claims in common parallel wrappers.
   - SIMD boundary: group target-feature and slice-lane invariants around the
     vectorized kernels rather than documenting every intrinsic load/store as a
     separate fact.

Proposed packet sequence:

| Order | Slice | Current entries | Intended handling |
| ---: | --- | ---: | --- |
| 0 | Baseline normalization and audit-gate repair | net 3,657 | Reconcile line drift, no semantic safety claim. |
| 1 | `src/lib.rs` SQL/FFI entrypoints | 181 | Split C ABI wrappers from safe Rust bodies; document `StringInfo`, Datum, typmod, relation OID, and `_PG_init` invariants. |
| 2 | common AM support and dispatch | 162 | Cover `am/mod.rs`, cost callbacks, detoast helpers, explain hooks, and common parallel scan shared-memory layout. |
| 3 | HNSW production scan/read path | about 387 | Split into scan, source/shared/graph, and page tuple read wrappers; avoid broad scan comments. |
| 4 | HNSW build/insert/vacuum/shared-state | about 465 | Wrap graph page mutation, tuple bytes, WAL/buffer locking, and insert/vacuum callback assumptions. |
| 5 | HNSW debug/test-gated scan support | 354 | Move after production wrappers so debug helpers reuse safe read surfaces. |
| 6 | IVF page/storage substrate | 133 | Coordinate with Task 42 format invariants; replace raw byte access with checked codecs where feasible. |
| 7 | IVF scan/build/insert/vacuum/admin | about 193 | Reuse page codecs; document callback, relation, snapshot, and posting-list chain invariants. |
| 8 | SPIRE DML frontdoor and planner hooks | 168 | Wrap plan-tree walking, relcache hook installation, backend-local mutable state, and catalog row extraction. |
| 9 | SPIRE coordinator and remote candidate paths | 294 | Separate snapshot/catalog wrappers from network/result-contract logic; defer concurrency-sensitive state to Task 40 where needed. |
| 10 | SPIRE CustomScan, storage, page, and scan | about 285 | Typed CustomScan private-state helpers, relation-store page wrappers, and checked page/header decoding. |
| 11 | SPIRE build, insert, update, vacuum, options, cost | about 139 | Small packet after SPIRE storage wrappers land; likely mostly callback documentation plus wrapper reuse. |
| 12 | DiskANN routine and scan-state | 115 | Split AM callbacks from exact-rerank heap access and scan materialization helpers. |
| 13 | DiskANN build/insert/cost/options | 115 | Wrap tuple byte copying, page chain mutation, relation options, and build callback invariants. |
| 14 | Quant SIMD kernels | 74 | Consolidate AVX2/NEON target-feature, lane-width, and slice-length invariants near kernel entrypoints. |
| 15 | Test-only unsafe sites | 499 | Replace repeated raw test FFI with safe fixtures/helpers after production wrappers are available. |
| 16 | Declaration and documentation cleanup | secondary | Audit `unsafe fn`, `unsafe extern`, `unsafe impl`, and docs once `unsafe {}` baseline is zero. |

High-risk/defer list:
- Page layout codecs in HNSW, IVF, SPIRE, and DiskANN should align with Task
  42. Do not annotate through missing on-disk invariants if Task 42 can encode
  them structurally.
- DSM/shared-memory and concurrent callback state should align with Task 40.
  Do not paper over unsound `Send`/`Sync`, slot ownership, or shared mutable
  state with comments.
- PostgreSQL resource wrappers should align with Task 41. Prefer moving raw
  relation, buffer, snapshot, scan, slot, SPI, and WAL use behind the wrapper
  layer instead of adding local comments.
- Miri/cargo-careful proof coverage from Task 43 should follow wrapper changes
  that touch pointer arithmetic, page codecs, SIMD kernels, or ownership
  boundaries.

Per-packet checklist:
- Scan the owning task bucket for feedback first.
- Record before/after `make unsafe-baseline-report` output in packet-local
  artifacts.
- Run `bash scripts/check_unsafe_comments.sh`.
- Run `git diff --check`.
- Run `make fmt-check` when Rust files changed.
- Run focused `cargo check`, `cargo test`, or PG18 `cargo pgrx test` only when
  wrappers, callbacks, page/WAL behavior, scan/build/vacuum/DML behavior, or
  SIMD behavior changed.
- Request review with explicit counts: removed unsafe blocks, safe wrappers
  added, comments added, remaining high-risk invariants, and validation run or
  skipped.

Review focus:
- Whether the normalization packet is the right first step despite Task 35's
  normal "baseline entries may only decrease" rule.
- Whether the packet sequence is narrow enough for review while still grouping
  repeated unsafe patterns behind real wrappers.
- Whether any slices should be blocked until Tasks 40, 41, 42, or 43 land.

Validation:
- `make unsafe-baseline-report`
  - artifact: `artifacts/unsafe-baseline-report.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
  - result: fails on this checkout because the line-number baseline is stale.
- `rg -n 'unsafe\s*\{' src | wc -l`
  - artifact: `artifacts/unsafe-block-count.log`
- Current missing-SAFETY scan using the same local algorithm as
  `scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/current-missing-unsafe-lines.txt`
- Baseline and current count breakdowns
  - artifacts: `artifacts/baseline-by-subsystem.log`,
    `artifacts/baseline-by-file.log`,
    `artifacts/baseline-by-spire-area.log`,
    `artifacts/current-missing-by-subsystem.log`,
    `artifacts/current-missing-by-file.log`
- Baseline drift comparisons
  - artifacts: `artifacts/current-missing-not-in-baseline.txt`,
    `artifacts/baseline-entries-not-current.txt`

Tests skipped:
- No Rust behavior changed; this is a planning and survey packet.
