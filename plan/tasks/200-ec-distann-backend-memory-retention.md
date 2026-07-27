# Task 200: ec_distann Backend Memory Retention

Status: **proposed** (2026-07-27). Priority: P1 — severity is unknown and the
plausible worst case is a production defect.

## Why

During the Task 188 packet-006 rerun, a PostgreSQL backend serving the repeated
physical query phase grew to approximately **52 GB RSS** and was still climbing
when the run was terminated to protect the host
(`reviews/task-188/006-batch10-stage-counters/artifacts/run/rerun-20260727/outcome.md`).
Over at most 60 queries that is on the order of **a gigabyte retained per
query** — unbounded per-query growth, not a large working set.

Task 188 worked around it by reconnecting the benchmark backend every five
queries, which was the right call to finish the diagnostic but leaves the defect
unexamined. The workaround also means nothing in the current benchmark path will
surface a recurrence.

The growth occurred while executing the ordinary semantic ANN SQL through the
coordinator's CustomScan — the production read path — not a benchmark-only
`pg_extern`. It is therefore **not** the same mechanism as the Task 185
diagnostic-memory fixes (`df89b5726`, `c83ea6ea8`), which bounded
`#[cfg]`-gated functions that do not exist in a production build.

The measurement was taken with `--distann-stage-counters` enabled and the same
loop has never been run with counters off, so the observation is confounded.
Resolving that confound is the first job.

## Why it matters more than a benchmark nuisance

If the retention is on the read path, the benchmark did not create the problem —
it found it faster by hammering one backend. Production connections are
long-lived, so a pooled connection serving thousands of queries is the real
exposure, and it would present as gradual backend bloat that nobody attributes to
`ec_distann`.

These subsystems also hold large state in Rust-allocated `static` and
`thread_local` structures, **outside PostgreSQL's memory contexts**. Statement and
transaction cleanup cannot reclaim any of it and `pg_backend_memory_contexts`
will not show it, so ordinary PostgreSQL memory diagnostics are blind to this
class of growth.

## Goal

Determine whether the growth is on the production physical read path or confined
to benchmark instrumentation, then bound it, with a regression test that would
catch a recurrence.

## Phase 1: the decisive experiment

One 100k three-owner physical generation, one backend, no reconnect
(`benchmark_backend_batch_size=0` / `worker_batch_size=0`), several hundred
repeated queries, RSS sampled at a fixed interval:

1. `distann_stage_counters=false` — the production configuration;
2. `distann_stage_counters=true` — the Task 188 configuration.

Report peak and slope of RSS for each. Flat with counters off scopes the issue to
instrumentation; growth with counters off makes it a production defect and
promotes this task's priority.

Report the query count and sampling interval; do not report a single peak without
the series.

## Phase 2: attribution

Only if Phase 1 shows growth. Attribute the allocation to a call site.
PostgreSQL's own memory reporting will not help for Rust-side allocations —
expect to need RSS plus allocator-level attribution (heaptrack, massif, or
jemalloc statistics), and record which tool produced the attribution.

**Already ruled out by source inspection at `c1c43a9bf`** — do not re-derive
these:

- `stage_counters.rs`: 37 atomics plus a fixed-size `BufferedAttribution` array;
  cannot produce GB regardless of scan count.
- `head_cache.rs`: `HashMap` keyed by `index_oid`, insert replaces the entry.
- `PHYSICAL_EPOCH_CACHE` (capacity 2) and `RETAINED_EPOCH_CACHE` (capacity 4) in
  `generation_read.rs`: correct LRU. The `push_back` without eviction is the
  *lookup* path, which does `remove(position)` first, so size is unchanged.
- `PHYSICAL_PREPARED_QUERY_CACHE` (capacity 4) and the owner payload plan cache:
  both evict.
- `scan_registry.rs`: fixed-capacity shared-memory slot arrays.

Since every deliberate cache is bounded, the likely shape is a per-scan
allocation that is never released. Suggested starting points: remote transport
buffers (`remote_transport.rs`), materialization maps, and CustomScan per-scan
state.

## Decision and closeout

- If the retention is on the production read path: bound it, and add a regression
  test that runs a few hundred queries on a single backend and asserts bounded
  RSS. Because the fix touches the read path, closeout requires a **10k / 50k /
  100k A/B** showing recall exact against the current numbers and no latency
  regression, per the repository closeout rule and NFR-007 provenance.
- If it is confined to instrumentation: bound it anyway, document the mechanism,
  and record why the production path is unaffected — with the counters-off series
  from Phase 1 as the evidence, not an argument from code reading.
- If Phase 1 cannot reproduce the growth at all: say so explicitly, retain the
  series, and close as unreproduced rather than fixed. Do not close on the
  reconnect workaround.

## Required review packets

1. `reviews/task-200/001-reproduction/`: Phase 1 series for both counter
   configurations, with the suite config checked in;
2. `reviews/task-200/002-attribution/`: conditional call-site attribution;
3. `reviews/task-200/003-fix-and-regression/`: conditional fix, regression test,
   and the A/B matrix if the read path changed.

## Non-goals

- BW8 promotion or any Task 188 candidate decision.
- The Task 188 benchmark-harness refactor and its pre-merge gates, which stay in
  that task.
- Recall, graph, head, or codec work.
- Removing the Task 188 reconnect workaround before Phase 1 answers the question;
  it is currently the only thing keeping long benchmark runs alive.

## References

- `reviews/task-188/006-batch10-stage-counters/` — the rerun outcome, the
  efficient rerun, and reviewer feedback `2026-07-27-02-reviewer.md` and
  `2026-07-27-03-reviewer.md`.
- Task 185 `df89b5726` / `c83ea6ea8` — the *different*, diagnostic-only
  statement-context fixes.
- NFR-007 benchmark provenance; FR-080, FR-082.
