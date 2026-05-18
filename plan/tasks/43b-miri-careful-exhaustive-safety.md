# Task 43b: Miri / cargo-careful Exhaustive Safety Extension

Status: **parked, scoped post-Task-35**

Sequencing: Task 35 (unsafe burndown) runs first. 43b begins after 35
lands so the surface it must test is the post-35 surface, not the
pre-35 surface. Several items below may dissolve organically when 35
removes `unsafe` blocks (see "Items that 35 will likely shrink").

## Scope

Task 43 stood up the Miri/Tree-Borrows/many-seeds/cargo-careful lanes
with breadth across every named subsystem, plus mutation-probe
sensitivity for nine subsystems and a campaign tracker that refuses
vacuous criteria. 43 is the *floor* of systematic safety testing.

43b raises the ceiling. The user-stated bar is "aggressive, extensive,
exhaustive safety testing" — meaning the campaign should not stop at
non-vacuous coverage but should drive the lane to a point where every
concurrent primitive, every adversarial input class, every unsafe
block, and every on-disk format helper has direct evidence under the
hardening stack.

Five depth dimensions:

1. **Concurrency depth — second through Nth surfaces.**
2. **SPIRE cargo-careful micro-harness via extraction.**
3. **Property-based and fuzz-driven adversarial inputs.**
4. **Per-test mutation probes (not just per-subsystem).**
5. **Coverage of `unsafe` blocks that survive Task 35.**

## Why

The Task 43 campaign tracker at
`reviews/task-43/001-coverage-survey-strategy/artifacts/campaign-tracker.md`
explicitly does *not* claim the following:

- Concurrency exhaustion. Only one threaded `miri_` test exists
  (`miri_parallel_worker_slots_are_unique_under_threaded_contention`).
  Every other concurrent primitive in the codebase (quantizer
  codebook `OnceLock<Mutex<_>>`, atomic counters elsewhere, any
  future Task 40 SPIRE coordinator state machine) is outside Miri's
  scheduler interleavings.
- SPIRE under cargo-careful. The SPIRE top-k, routing,
  vacuum/delete-delta, remote payload, and serialization mirrors are
  blocked on pgrx/Oid/SPIRE-harness extraction work that the campaign
  tracker named but did not execute.
- Exhaustive mutation sensitivity. Nine probes proved one
  obvious-bug class per major subsystem. Subtler regressions
  (borrow splits that preserve algorithmic answers, off-by-one in
  rejection paths that still rejects, etc.) are not directly probed.
- Property-based / fuzz layer. The serialization rejection paths and
  remote typed payload validation have the shape of fuzz targets but
  no fuzz driver has been written.
- Concurrent code paths that Task 35 will modify. Refactors that
  remove `unsafe` may introduce regressions in concurrent or
  non-concurrent code; the lane catches what it covers, not what 35
  changes.

The campaign-tracker rules also exclude pgrx callbacks, PostgreSQL
memory contexts, SPI, libpq, and live relation access from Miri/careful
by design. 43b respects that boundary; it extracts pure helpers where
needed instead of dragging pgrx in.

## Approach

### 1. Concurrency depth — second through Nth surfaces

Add real threaded `miri_` tests under `-Zmiri-many-seeds` for every
concurrent primitive ECAZ exposes. Each test must use a real
production primitive, not a synthetic counter.

Candidate primitives (audit before starting; 35 may add or remove):

- `OnceLock<Mutex<_>>` around the quantizer codebook init path
  (`src/quant/...`). Test concurrent init, concurrent reads, and
  the once-only contract.
- Atomic counters / sequence numbers anywhere they coordinate across
  threads (audit `AtomicU64`, `AtomicUsize`, `AtomicBool` usages).
- SPIRE coordinator worker state, *after* Task 40 lifts it into pure
  Rust. Tests should cover the legal transitions and reject illegal
  ones.
- `src/am/common/parallel.rs` deeper coverage: epoch transitions
  across concurrent workers, runtime snapshot races, claim/release
  interleavings beyond the one slot-uniqueness test.
- Any new concurrent primitive 35 introduces while refactoring away
  `unsafe`.

For each primitive, the test must:

- Spawn more threads than slots / more contenders than the primitive
  is sized for.
- Force contention before any release.
- Assert both happy-path invariants and rejection invariants.
- Pass under default Miri, Tree Borrows, and `-Zmiri-many-seeds=0..128`.

### 2. SPIRE cargo-careful micro-harness via extraction

The Task 43 tracker has the SPIRE careful blockers scoped. 43b
executes them.

Strategy:

- Inventory every SPIRE pure helper currently behind a pgrx-tangled
  module boundary.
- For each, decide: (a) lift into `hardening/careful` via path-lift
  (preferred), (b) extract into a sibling pure module that careful
  can lift, or (c) build a SPIRE-specific careful micro-harness that
  isolates pgrx-free SPIRE code.
- Add to `hardening/careful/src/lib.rs` and run the existing
  `make careful` lane plus a new SPIRE-specific careful sub-target if
  needed.

Target coverage growth: from 69 tests today to *at least* 100, ideally
120+. The exact number depends on what's extractable post-35.

### 3. Property-based and fuzz-driven adversarial inputs

The Task 43 rejection-path coverage is hand-written: per format, one
or two adversarial inputs. 43b adds generators.

- **proptest layer** for serialization round-trip and rejection paths
  (SPIRE assignment row, delta partition object, vec-id, DiskANN
  tuple/codebook, HNSW page tuples, top-graph). Property:
  encode-then-decode equals the input; rejection paths trigger on
  every malformed variant the strategy can generate.
- **cargo-fuzz** target for the Row-independent remote typed payload
  parser (packet 009's extraction enables this directly). The parser
  takes raw fields → fuzz the field bytes.
- **cargo-fuzz** target for HNSW/DiskANN graph traversal helpers with
  adversarial graph fixtures (cycles, parallel edges, empty layers,
  disconnected components).
- Integrate fuzz corpora as packet-local artifacts. Crashes go into
  `artifacts/fuzz-crashes/<target>/` for replay.

### 4. Per-test mutation probes

Task 43's packet 013 ran one mutation per major subsystem (nine
total). 43b extends this to per-test for the highest-risk surfaces.

Approach:

- For every `miri_` test in the rejection-path categories (parser,
  serialization, vacuum visibility, top-k tie ordering), generate
  at least one targeted mutation that *should* break that specific
  contract.
- Use a mutation-testing tool if one is available
  (`cargo-mutants`?); otherwise produce hand-written diffs the same
  way packet 013 did.
- Each mutation must produce a failing log; restore source before
  commit; record diff and log packet-locally.

Target: 30–50 mutation probes total across the campaign, up from 9.

### 5. `unsafe` block coverage that survives Task 35

After 35 lands, audit every remaining `unsafe` block. For each, ensure
at least one `miri_` test exercises that block under both Stacked
Borrows and Tree Borrows. If 35 leaves an `unsafe` block that no Miri
test reaches, write a test before declaring 43b complete.

## Items that 35 will likely shrink

These are 43b line items that may dissolve when 35 removes `unsafe`:

- SPIRE storage / serialization careful mirror — if 35 removes pgrx
  `Datum` juggling from SPIRE helpers, those modules become path-lift
  candidates rather than extraction candidates.
- Mutation probes targeting `unsafe` in modules 35 cleans up — if 35
  deletes the unsafe block, no mutation probe is needed.
- Some fuzz targets — if 35 replaces a hand-rolled byte parser with
  `bytes::Buf` or similar safe abstractions, the fuzz target shifts to
  the safe-Rust boundary.

Re-scope 43b after 35 lands and before starting execution. Drop any
line item whose underlying surface no longer exists.

## Validation

- `make miri-many-seeds` runs clean over the full `0..128` seed range
  with at least 3 distinct concurrent surfaces represented in the test
  set.
- `make miri-tree` and `make miri` both report ≥ 100 prefixed tests
  passing (up from 87).
- `make careful` reports ≥ 100 tests passing (up from 69).
- `cargo proptest` (or equivalent) lane lands and runs clean over the
  serialization round-trip and rejection-path properties.
- `cargo fuzz` corpora persist as packet-local artifacts; the lane
  runs for a bounded budget (e.g. 5 minutes per target) without
  crashes.
- Per-test mutation matrix is at least 30 entries with paired diff +
  failing-log evidence.
- Every `unsafe` block remaining post-35 has at least one `miri_`
  test reaching it; the audit table lives in the campaign tracker.

## Exit Criteria

- All four 43b depth dimensions (concurrency, careful mirroring,
  fuzz/property, per-test mutation) have packet-local evidence under
  `reviews/task-43b/<NN>-<topic>/artifacts/`.
- The Task 43 campaign tracker is extended to include 43b gates
  (G9–G12 or similar) and marks each `Done` only with evidence.
- The `unsafe` block coverage audit table is in the tracker and
  shows zero blocks without Miri coverage.
- `docs/hardening.md` documents the proptest / fuzz lanes alongside
  the existing Miri / careful lanes.

## Dependencies

- **Task 35 (unsafe burndown)** — required predecessor. 43b is
  scoped against the post-35 surface, not the current surface. Do not
  start 43b until 35 lands.
- **Task 40 (concurrency model checking)** — soft dependency. If
  Task 40 lifts the SPIRE coordinator state machine into pure Rust
  before 43b starts, 43b adds that to the concurrency depth roster
  (item 1). If Task 40 lands during 43b, integrate when available.
- **Tasks 39 / 47** — independent. Different surface (gate
  entrypoints). Safe to run in parallel.

## Out of scope

- PG18 callback paths under Miri (by campaign rule — pgrx stays out).
- Live-cluster integration tests (different lane).
- Performance regression detection — that's the benchmark
  packets' job, not the Miri lane.

## Open questions for re-scoping after Task 35

1. Which `unsafe` blocks does 35 leave behind? That set defines item
   5's exact scope.
2. Did 35 extract any SPIRE pure helpers that should now be lifted
   into the careful harness?
3. Did 35 introduce any new concurrent primitives that need item 1
   coverage?
4. Is there a mutation-testing tool now in the toolchain (`cargo-mutants`,
   `mutagen`)? If so, item 4 should use it; otherwise stick with the
   hand-rolled diff pattern from packet 013.
