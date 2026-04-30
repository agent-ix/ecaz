# Task 29c: DiskANN Build Performance Tuning

Status: landed on branch, no Task 29d blocker opened
Owner: coder1 / runtime-index track
Backstory: `review/11099-task29-diskann-landing-readiness/feedback.md`

## Goal

Bring DiskANN single-process build cost into a defensible range vs
reference implementations and the in-house `ec_hnsw` build, so the
Task 29 landing slice doesn't ship with surprise build latency.

The phrase "defensible range" is deliberate. The user's framing:

> I would try to match other diskann impls. HNSW is good as a
> reference too just for relative performance. My big concern is
> setting unreasonable expectations. Let's measure and try to
> exhaust performance tuning options within reason.

This is a measurement-first task with a stop condition tied to
diminishing returns, not a fixed-target sprint. We are not
promising specific speedups upfront because we don't yet know where
the cost is.

## Current State

Task 29c initially used the Task 29a baseline measurement
(`review/11096-task29a-diskann-binary-sidecar-prefilter/`):

- **10k × 1536-d real-corpus build**: 503.10 s total
  - copy: 4.27 s
  - encode: 4.55 s
  - **index build: 492.13 s**

Packet `11102` found that this number was not a release-performance
measurement: the local PG18 extension had been installed in the
debug/dev cargo profile. Reinstalling the same head with
`cargo pgrx install --release` and rerunning the isolated index-only
build changed the result to:

- **DiskANN index-only build**: 79.238 s
  - heap scan: 1.261 s
  - training: 0.130 s
  - payload derivation: 0.293 s
  - build/persist: 77.485 s
  - Vamana graph: 75.903 s
  - page writes: 0.059 s
- **HNSW reference build** on the same table with `m=32`,
  `ef_construction=100`, `build_source_column=source`: 5.23 s
- **Index size**: DiskANN 4.8 MiB, HNSW 14 MiB

The remaining gap is Vamana graph-construction work, dominated by
pass-1 greedy search, robust-prune, and backlink repair. It is not a
tuple-persistence or page-write bottleneck. Future performance packets
must state whether the extension is debug/dev-installed or
release-installed; release mode is the default for Task 29 local
performance claims.

## Phase 1 — Profile the gap

Completed in packets `11101` and `11102`.

Before changing optimization code, Task 29c figured out where the
apparent overhead between the in-memory replay and full ambuild went.
The candidates are well-known but unranked:

- **Page persistence and WAL**: every node tuple is written through
  the data-page chain; depending on how the chain stages writes, we
  could be paying per-tuple write cost rather than per-page batched
  cost.
- **Repeated tuple reads during prune**: the build calls `read_node`
  through `PersistedGraphReader` during graph construction. If the
  in-memory build keeps decoded tuples in scope and the persisted
  build re-reads/re-decodes, that's a 6× factor right there.
- **Codebook chain staging**: `ambuild.rs:313` calls
  `stage_grouped_codebook_chain`. Likely cheap, but unknown.
- **PG heap scan callback overhead**: each row goes through the
  pgrx index-build callback. Probably small for 10k rows but
  unmeasured.
- **Encoding inside the build loop**: source vectors get SRHT'd then
  PQ-encoded then sign-derived. The "encode" line in the load log is
  4.55 s — but that's the corpus pre-encode in the loader; the
  ambuild path may be re-encoding inside the build callback.

**Step 1.1**: flamegraph/perf was superseded by structured timing
because the release-vs-debug install caveat explained the 6.7x
apparent gap without requiring stack sampling.

**Step 1.2**: Done. Structured timing logs bracket model training,
payload derivation, `build_and_persist_vamana`, core medoid search,
core graph construction, core persistence, overflow stage, codebook
chain stage, data-page write, metadata page write, and total time.

**Step 1.3**: Done. The top contributor is Vamana graph construction
inside the in-memory core. Under release mode, persistence/page-write
time is negligible relative to graph construction.

**Decision gate after Phase 1**: Proceed to landing with observability
and open no Task 29d blocker. Further single-process optimization is
possible, but no longer required for the Task 29 landing slice because
the release local build is in the same order of magnitude as the
in-memory Vamana replay.

## Phase 2 — Attack the largest fixable contributor

Status: skipped for the landing slice after Phase 1 corrected the
debug/dev measurement caveat. The only code experiment attempted was
the build-frontier heap change, and packet `11101` showed the corrected
implementation regressed build time; it was reverted.

Order operations strictly by what the Phase 1 profile shows. The
following list is "candidate optimizations to consider", not "things
to do in order".

- **Decoded-tuple cache during build.** If `PersistedGraphReader::read_node`
  re-decodes the same tuples many times during the two-pass Vamana
  construction (which is plausible — every neighbor of every pivot
  is read), an in-memory `HashMap<ItemPointer, VamanaNodeTuple>` for
  the duration of the build could cut decode cost dramatically.
  Build memory can hold the entire decoded graph for 10k rows; for
  10M rows we'd need a bounded LRU.
- **Batch page persistence.** If pages are currently flushed
  one-at-a-time during construction, batching to end-of-build (or
  to fixed-size chunks) reduces WAL traffic and `pwrite` syscalls.
- **Eliminate redundant in-build encoding.** If the source vectors
  are encoded to grouped-PQ codes once in the loader and then
  re-encoded during ambuild, deduplicate. The loader's encode time
  is observable in the load log; the ambuild's is not, so Phase 1
  needs to confirm whether duplication exists.
- **Pipeline encoding with build.** The corpus encode (4.55 s) and
  index build (492 s) are sequential today. They can overlap: as
  the heap scan emits row N's source vector, encode it on a worker
  thread while the build thread continues on rows < N.
- **Tighten `robust_prune`.** The prune call is O(R²) per pivot at
  worst. If profiling shows a meaningful chunk in there, the
  pgvectorscale-style "skip prune when candidates ≤ R" may apply.
  Also consider whether the candidate distance recompute can reuse
  cached distances from the greedy descent that produced the
  candidate set.
- **In-build scan-path heap.** The scan-path frontier optimization
  from `27bb6af8` is also applicable to greedy descent during
  build. Currently `vamana::greedy_search` (the in-memory variant)
  uses a linear-scan frontier identical to what we already replaced
  in `scan::greedy_descent_with`. Same change shape, same expected
  win.

**Hard rule**: each optimization gets its own packet with a
before/after measurement on the same corpus and the same hardware.
No "we expect this will help" — measure or skip.

**Stop condition**: stop when the next optimization candidate would
need >2 days of work and the Phase 1 profile suggests <20% of
remaining cost. Document what was attempted and what remains; that's
the honest ceiling for the single-process landing slice.

## Phase 3 — Reference comparison

Status: completed for the in-house local reference row in packet
`11102`; external pgvectorscale/Microsoft comparisons remain optional
future context, not Task 29 landing blockers.

Once Phase 2 has exhausted within-reason single-process
optimizations, compare against:

- **`ec_hnsw` on the same 10k real corpus.** Build wall-time, build
  CPU time, peak memory. This is the "relative reference" the user
  asked for. HNSW build is ~30 minutes for 50k rows on the same
  hardware (per `plan/tasks/26-parallel-index-build.md` packet 666),
  which scales to roughly 6 minutes for 10k single-threaded; the
  parallel-build win brings it under 1 minute. So the practical
  reference is "DiskANN should be in the same order of magnitude as
  serial HNSW build at minimum, and we'll know if parallel build is
  the next ask based on how much daylight remains vs the parallel
  HNSW number."
- **pgvectorscale DiskANN on the same 10k real corpus, if
  installable in the local PG18 environment.** The original Task 29
  charter (Phase 1) explicitly asks for this:
  > Add pgvectorscale DiskANN or another PostgreSQL DiskANN
  > reference if installable without distorting the local benchmark
  > environment.
  This was deferred earlier; now is the right time to land it.
  Document install steps, version, and reloptions in the packet.
  Match `graph_degree`, `build_list_size`, `alpha` to ours so the
  comparison is apples-to-apples. If pgvectorscale's parallel build
  is on by default, also report its single-worker number for a fair
  per-process comparison.
- **Microsoft DiskANN reference numbers from the published paper or
  the canonical implementation**, if a directly comparable corpus
  is available. Lower priority — paper numbers come from different
  hardware and aren't directly comparable, but they bound what's
  algorithmically possible.

**Decision gate after Phase 3**: Is the post-Phase-2 build cost in
the "same order of magnitude" range as the references? If yes,
land. If no, the result of this task is an explicit "single-process
build is N× the reference; recommend Task 29d for parallel build
before landing" with a clear ratio and reference numbers.

Task 29c decision: land the single-process slice with structured build
observability. The release local build is slower than HNSW (`79.238s`
vs `5.23s` on real-10k), but the corrected result is no longer a
surprise latency blocker for this initial tuning lane. The next
optimization target is pass-1 Vamana graph construction, not a landing
dependency.

## Phase 4 — (Conditional) Parallel build scope

Status: not opened by Task 29c.

Only if Phase 3 concludes single-process is not landable, open
Task 29d for parallel build. **Note that Task 26 covers parallel
build for `ec_hnsw` only** — DSM graph assembly there assumes HNSW's
level structure and entry-point semantics, neither of which apply to
Vamana. So 29d would be a separate parallel-build implementation,
not a reuse of 26's scaffolding (some shared infrastructure in
`build_parallel.rs` may apply, but the algorithmic core is different).

Defer scoping 29d until Phase 3 says it's needed. Don't speculate
on its shape today.

## Out of scope for 29c

- Cleanup / vacuum consistency / SIMD verification — Task 29b.
- The grouped-PQ scan path removal — see 29b "Out of scope" note;
  not a removal lane at all currently.
- Distributed / GPU build — ADR-046 deferred follow-up, not in this
  lane.

## Validation gate

- Structured per-phase build timings shipped as part of the ambuild
  logging.
- Packet `11101` records the baseline phase split and the reverted
  heap-frontier experiment.
- Packet `11102` records the Vamana core split, the release-installed
  extension measurement, the HNSW reference build, and the corrected
  landing recommendation.
- Future performance packets explicitly state debug/dev vs release
  extension install profile.

## Acceptance criteria

- Build cost on real-10k is documented with a clean explanation of
  where the remaining cost lives.
- The HNSW reference comparison is published as a packet artifact with
  reproducibility metadata sufficient for outside review.
- The Task 29 landing slice ships with the corrected release-mode
  numbers and no explicit dependency on Task 29d.

## Estimated size

Completed in the current Task 29 branch. Further Vamana build
optimization can proceed as a new follow-up, scoped from packet
`11102`'s pass-1 timing split.
