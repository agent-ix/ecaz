# Task 43 Coverage Survey And Strategy

Planning packet only. No code changes are under review in this packet.

## Context

- Branch: `task43-miri-careful-depth`
- Base refreshed from `origin/main` after Task 41 invariant #1/#3 landed.
- Survey SHA: `602bd391`
- Task: `plan/tasks/43-miri-careful-depth.md`

Task 41 added FFI/resource guard work, `ffi-lint`, and storage guard wrappers.
Task 43 should not duplicate that boundary work. This lane should stay on
Miri/cargo-careful depth for pure-Rust logic and only lift pgrx-tangled modules
when a narrow pure core is clear.

## Current Coverage

Current Miri entry point:

- `make miri`
- `make miri-expanded`
- both run `cargo +nightly miri test --lib -- miri_`

Current `miri_` inventory is 19 tests:

- `src/storage/page.rs`: `ItemPointer`, `DataPageChain`.
- `src/quant/hadamard.rs`: FWHT scalar paths.
- `src/quant/prod.rs`: encode/decode/pack/score paths.
- `src/am/ec_diskann/page.rs`: DiskANN metadata page roundtrip.
- `src/am/ec_hnsw/page.rs`: HNSW element, grouped/turbo hot, rerank,
  neighbor, and metadata tuple roundtrips.
- `src/am/ec_hnsw/scan.rs`: raw opaque stats-delta pointer update.
- `src/am/ec_spire/storage/tests/leaf.rs`: SPIRE leaf V2 metadata/column
  invariants.

Current cargo-careful coverage is much narrower:

- `hardening/careful/src/lib.rs` path-lifts only `src/storage/page.rs`.
- The careful harness has two storage-page tests and does not cover the
  existing in-crate `miri_` inventory.

## In-Scope Surfaces

### Depth lanes

Files:

- `Makefile`
- `scripts/hardening.sh`
- `scripts/hardening_tiers_report.sh`
- `docs/hardening.md`
- `docs/hardening-governance.md`

Need first-class lanes:

- `make miri-tree`: `MIRIFLAGS="-Zmiri-tree-borrows"` over `miri_`.
- `make miri-many-seeds`: `MIRIFLAGS="-Zmiri-many-seeds=128"` over threaded
  or atomic `miri_` tests.
- `make miri-full`: default + Tree Borrows + many-seeds.
- `hardening-nightly-local` should depend on `miri-full`, not only
  `miri-expanded`.

The hardening script still defaults `RUSTUP_CARGO` and `RUSTUP_BIN` to
Homebrew paths. Fix that while adding lanes by discovering `cargo` and `rustup`
from `PATH`, preserving the Homebrew paths as fallback.

### DiskANN graph helpers

Files:

- `src/am/ec_diskann/vamana.rs`

Pure candidates:

- `greedy_search` / `greedy_search_view`
- `robust_prune`
- `build_vamana_graph_with_stats`
- `build_vamana_graph_with_pass1_extra_candidates`

Existing unit tests already exercise the algorithmic core, but none are
Miri-prefixed. Add small `miri_` tests with bounded fixture graphs rather than
running large build tests under Miri.

### HNSW graph traversal helpers

Files:

- `src/am/ec_hnsw/search.rs`
- `src/am/ec_hnsw/graph.rs`

Pure candidates:

- `BeamSearch`
- `VisibleFrontier`
- `select_next_with_refill`
- deterministic frontier ordering and stale-candidate removal

`src/am/ec_hnsw/search.rs` has rich pure unit tests and is a strong first
coverage target. Prefer adding Miri prefixes to one or two representative tests
instead of copying all tests into the Miri lane.

### Top-k / candidate merge

Files:

- `src/am/ec_spire/scan/candidates.rs`
- `src/am/ec_spire/scan/tests/candidates.rs`
- `src/am/ec_spire/scan/tests/runtime_state.rs`

Pure candidates:

- `SpireScoredCandidateAccumulator`
- `rank_bounded_scored_candidates`
- `scored_candidate_cmp`
- `rerank_scored_candidates_by_ip`
- `SpireScanCandidateCursor`

Existing tests cover bounded ranking, vec-id dedupe, primary-vs-boundary
replica tie breaks, non-finite score rejection, rerank prefix behavior, and
cursor output shape. Add narrowly named `miri_` tests around bounded dedupe,
tie-break order, and cursor exhaustion.

### SPIRE routing / coordinator state

Files:

- `src/am/ec_spire/scan/routing.rs`
- `src/am/ec_spire/scan/tests/routing.rs`
- `src/am/ec_spire/coordinator/types.rs`
- `src/am/ec_spire/coordinator/remote_candidates/production_transport.rs`
- `src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`

Pure candidates:

- top-graph route ordering and validation,
- route count/nprobe selection,
- production executor state summaries that operate on Rust structs.

Avoid live libpq/SPI/pgrx entry points. If a useful coordinator state machine
is still tangled with pgrx rows or relation access, extract a small pure helper
first and test that helper under Miri/careful.

### Remote parser / typed payload validation

Files:

- `src/am/ec_spire/coordinator/remote_candidates/payload.rs`
- `src/am/ec_spire/coordinator/remote_candidates/payload_limits.rs`
- `src/am/ec_spire/coordinator/remote_candidates/tests/production_executor_state.rs`

Current blocker:

- The main decoders take `postgres::Row`, which is not Miri-friendly.

Pure candidates:

- `typed_payload_hex_decoded_bytes`
- row/batch payload limit validators
- tuple payload width, OID string, collation fallback, format, and transport
  validation after extracting the Row-independent part.

Strategy: first add `miri_` tests for already-pure payload limit helpers, then
extract a Row-independent typed-payload parser if deeper adversarial coverage is
needed.

### Vacuum helpers

Files:

- `src/am/ec_diskann/vacuum.rs`
- `src/am/ec_spire/vacuum/mod.rs`
- `src/am/ec_spire/vacuum/tests.rs`

DiskANN has a clean pure primitive module:

- `mark_deleted`
- `strip_dead_primary_heaptid`
- `is_fully_dead`
- `repair_neighbors`

These are good Miri/careful targets now. SPIRE vacuum is more relation/object
store heavy, but `collect_visible_assignments` and delete-delta grouping can be
covered if existing mock object-store tests can be kept pure. Do not pull live
relation access into Miri.

### Serialization / on-disk helpers

Files:

- `src/am/ec_diskann/tuple.rs`
- `src/am/ec_hnsw/page.rs`
- `src/am/ec_spire/storage/*`

Existing HNSW page coverage is good. Missing Miri coverage is strongest for:

- DiskANN `VamanaNodeTuple` and `VamanaCodebookTuple` encode/decode.
- SPIRE top-graph object encode/decode and rejection paths.
- SPIRE assignment/delta/vec-id helpers beyond leaf V2.

Keep tests bounded and deterministic. Prefer adding `miri_` prefixes to small
existing roundtrip/rejection tests rather than creating large fixture builders.

## Cargo-Careful Strategy

The careful harness should grow in the same order as Miri coverage, but only
for modules whose dependency closure stays outside pgrx callbacks:

1. Add lifted modules for pure files that already compile standalone:
   `storage/page.rs`, `am/ec_diskann/vacuum.rs`, `am/ec_diskann/vamana.rs`,
   and `am/ec_hnsw/search.rs`.
2. For SPIRE scan/routing/storage code, first check whether path-lifting pulls
   in pgrx-heavy dependencies. If it does, extract a small pure helper module
   instead of forcing the whole module into `hardening/careful`.
3. Keep careful tests representative, not exhaustive. The goal is debug-assert
   stdlib coverage over the same high-value pure paths, not a duplicate unit
   test suite.

## Many-Seeds Strategy

There are no existing `miri_` tests that spawn threads. The repo has atomics and
threaded tests, but most are pgrx/live-cluster oriented. Do not claim
many-seeds interleaving depth until at least one real pure threaded or atomic
Miri test exists.

Candidate later targets:

- a small quantizer cache coherence test around `OnceLock<Mutex<...>>`,
- a lifted pure subset of `src/am/common/parallel.rs`,
- any Task 40 lifted coordinator state machine once available.

Until then, `miri-many-seeds` can exist as a lane but packet evidence should say
it currently reruns the same non-threaded prefix and is ready for future
threaded coverage.

## Proposed Checkpoints

1. **Lane infrastructure.** Add `miri-tree`, `miri-many-seeds`, `miri-full`,
   hardening docs, governance inventory, and PATH-based rustup discovery.
   Validate with shell syntax, Makefile dry runs, `hardening-validate`, and one
   focused Miri lane if time permits.
2. **Pure graph coverage.** Add representative `miri_` tests for DiskANN
   Vamana and HNSW `BeamSearch`/`VisibleFrontier`. Extend careful to those
   modules if path-lift stays clean.
3. **Candidate merge coverage.** Add Miri tests for SPIRE bounded candidate
   accumulator, tie-breaks, rerank prefix, and cursor exhaustion.
4. **Serialization/vacuum coverage.** Add Miri tests for DiskANN tuple/vacuum
   helpers and SPIRE top-graph/assignment/delta encode/decode.
5. **Remote payload parser.** Cover pure payload limit helpers first; extract
   Row-independent typed-payload validation only if needed.
6. **Threaded many-seeds coverage.** Add one real pure threaded/atomic Miri
   test before making any stronger claim about many-seeds depth.

## Validation Policy

Task 43 is hardening work, so run tests only when the checkpoint changes
behavior or claims tool evidence. For strategy-only changes, no tests are
needed. For implementation packets, prefer narrow commands:

- `bash -n scripts/hardening.sh`
- `make -n miri-tree miri-many-seeds miri-full`
- `bash scripts/hardening_validate.sh`
- `cargo +nightly miri test --lib -- <specific miri test name>`
- `cargo test --manifest-path hardening/careful/Cargo.toml --lib`

Store any logs cited by future review requests under that packet's
`artifacts/` directory.
