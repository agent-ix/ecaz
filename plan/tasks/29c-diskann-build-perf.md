# Task 29c: DiskANN Build Performance Tuning

Status: planned, gate for landing decision (Task 29 lane)
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

From the Task 29a baseline measurement
(`review/11096-task29a-diskann-binary-sidecar-prefilter/`):

- **10k × 1536-d real-corpus build**: 503.10 s total
  - copy: 4.27 s
  - encode: 4.55 s
  - **index build: 492.13 s** ← the chunk we need to attack
- Per-row index-build cost: **~49 ms/row** single-threaded.

Plus a hint from the in-memory build probe in
`review/11089-task29-diskann-build-probe/`:

- In-memory Vamana algorithm core (no persistence): 73.2 s on the
  same 10k corpus.
- Implies a **~6.7× gap** between the algorithmic core and the full
  persisted ambuild. That gap is where Phase 1 of this task should
  look first.

## Phase 1 — Profile the gap

Before changing any code: figure out where the 419 s of overhead
between the in-memory replay and the full ambuild actually goes.
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

**Step 1.1**: Run `cargo flamegraph` (or perf record + perf report)
on a real-10k DiskANN build, with debug symbols enabled. Capture
the top 20 hot stack frames as a packet artifact. This is the
single most valuable measurement in the task — every later decision
keys off it.

**Step 1.2**: Add structured timing logs to `ambuild.rs:flush_build_state`
that bracket each major phase: model training, payload derivation,
`build_and_persist_vamana`, overflow stage, codebook chain stage,
data-page write, metadata page write. Land these as part of the
profiling work even if the rest of the task doesn't ship — the logs
are diagnostic gold for any future build-perf work.

**Step 1.3**: Cross-reference the flamegraph against the structured
phase timings. Identify the top 1–2 contributors that together
account for ≥ 50% of the overhead vs in-memory replay.

**Decision gate after Phase 1**: Do the top contributors look
addressable in single-process code (page-batching, decode caching,
encoder pipelining), or do they look fundamentally bound by I/O /
algorithmic cost? Decide whether to proceed to Phase 2 attack or
skip directly to Phase 3 reference comparison.

## Phase 2 — Attack the largest fixable contributor

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

We do not pre-commit to landing without the reference comparison.
The whole point of this task is to make the landing decision
defensible.

## Phase 4 — (Conditional) Parallel build scope

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

- A flamegraph artifact attached to the Phase 1 packet.
- Structured per-phase build timings shipped as part of the
  ambuild logging (these are also useful in production
  observability, so they stay in the codebase regardless).
- Each Phase 2 optimization has its own packet with before/after
  measurement on real-10k.
- Phase 3 reference table comparing ec_diskann (post-tuning),
  ec_hnsw, and pgvectorscale (if installable) on the same hardware
  and corpus.
- A single landing-decision packet with the reference comparison
  table, the residual cost, and an explicit recommendation:
  "land single-process" or "open 29d for parallel build before
  landing".

## Acceptance criteria

- Build cost on real-10k after Phase 2 has either landed at parity
  (within 2× of the strongest reference number) or is documented
  with a clean explanation of where the remaining cost lives and
  why it's not single-process-tunable.
- The reference comparison is published as a packet artifact, with
  reproducibility metadata sufficient for outside review.
- The Task 29 landing slice ships either with the post-Phase-2
  numbers, or with an explicit dependency on 29d.

## Estimated size

Roughly 1–3 weeks depending on what Phase 1 finds. Phase 1 alone is
~3 days (profile setup + structured logging + cross-reference).
Phase 2 is the variable: could be one optimization that wins big
(~3–5 days) or a sequence of smaller wins (~2 weeks). Phase 3 is
~2–3 days assuming pgvectorscale installs cleanly.
