# Task 112 Review Request — Packet 001: Lazy Heap-F32 Rerank (Contract + Gate + Instrumentation)

Branch: `task-112-ivf-lazy-heap-f32-rerank`
Code checkpoint: `d7021c181` ("Task 112: lazy heap-f32 rerank driver, contract,
gate, instrumentation"), off base `61fd84f95`.

This packet covers Task 112 Phases 1–4. Phase 5 (the bench gate) is
**env-blocked on this dev/review box** (no `ecaz` binary, no staged corpora) and
is deferred to the Intel bench desktop; a ready-to-run `ecaz bench suite` A/B
config is included under `artifacts/`.

## Scope (matches the task)

IVF only, `rerank = 'heap_f32'` only. No change to candidate generation, posting
layout, quantizer math, or index storage. Default `f32`/`table` rerank stays
exact; the lazy path is recall-neutral today (see contract below).

## The correctness crux — what was settled

The `ec_ivf` AM is an **ordered index scan**: it returns the whole
`rerank_width` frontier sorted by *exact* score and the executor pulls `k` rows
via `amgettuple`. The AM does **not** know `k`. So the set the AM is responsible
for is the entire `rerank_width` frontier; in the worst case the executor pulls
all of it.

A lazy stop is recall-safe only if every skipped (un-exact-scored) candidate is
provably worse, by exact score, than every candidate the executor could pull.
Scores are negative inner product (lower is better), so "candidate j cannot beat
worst-kept" is `e_j >= worst_kept`, which can only be asserted from a **sound
lower bound** `lower_bound(e_j) >= worst_kept`. The quantized approximate score
is **not** a sound lower bound on the exact neg-IP (quantization error is
two-sided), so using it (or a guessed slack) as an early stop would drop recall —
the explicit Non-Goal. A calibrated lower bound is exactly what **Task 113**
produces (its Phase 4 "Rerank Frontier Integration").

**Decision (the honest, provably-safe form the task prescribes):** implement the
lazy driver + stop predicate + gate + instrumentation now, with the only *sound*
bound available today — `NoBound`, `lower_bound = -inf` — under which the stop
**provably never fires early**, so the lazy path reranks the full width and is
**byte-identical** to fixed-width (proven by the equivalence test). The bound is
a trait seam (`LazyRerankBound`) that Task 113 plugs a calibrated implementor
into with **no change to the stop logic**. A real early stop additionally needs a
second prerequisite — a `k`-cap or on-demand fetch of the skipped suffix, since
without `k` pushdown the kept floor equals the full width — documented in
`lazy.rs`. This is the legitimate "no safe non-trivial stop with today's
frontier" finding; surfacing it for your decision rather than forcing an unsafe
stop.

**Coordination with Task 113:** 113 owns the calibrated lower bound for the IVF
candidate frontier. When it lands, an implementor of `LazyRerankBound` returning
finite sound bounds (plus a `k`-cap) turns this seam into a live latency win. The
included bench A/B becomes the measurement at that point.

## What landed

- `src/am/ec_ivf/lazy.rs` — `drive_lazy_rerank` (best-approx-first incremental
  rerank with the sound stop predicate), `LazyRerankBound` seam, `NoBound`
  default, `LazyRerankPlan` (considered / reranked_prefix_len / skipped). Full
  proof in module + fn docs. 6 unit tests including a property test that every
  skipped candidate is provably not better than the worst kept.
- `src/am/ec_ivf/scan.rs` — drive the plan inside `rerank_probe_candidates`;
  exact-score only the reranked prefix (== full width under NoBound); record
  considered/skipped; gate by `ec_ivf.lazy_heap_rerank`. Debug snapshot exposes
  the two new counters to pg_test.
- `src/am/ec_ivf/options.rs` — `ec_ivf.lazy_heap_rerank` Userset GUC (default
  on) + accessor; disable for a deterministic fixed-width A/B.
- `src/am/common/explain.rs` — `stats_rerank_candidates_considered` /
  `stats_rerank_candidates_skipped` counters, recorder, EXPLAIN rows. Reuses the
  existing 111e/g counters (`stats_rerank_rows`, `stats_heap_blocks_fetched`,
  `stats_exact_rerank_elapsed_us`, `stats_approximate_scan_elapsed_us`,
  `stats_rerank_source_bytes_read`, `stats_heap_tids_scored`) — only the
  genuinely-missing considered/skipped were added.
- `src/tests/ec_ivf.rs` — 7 pg_test acceptance cases.

## Phase 1 note (instrumentation stop condition)

The Phase-1 "close if heap rerank isn't a meaningful share of latency" gate
needs a bench host. Prior 111e packet-005 evidence already established
tid-sorted heap rerank reads as a dominant IO lever, so the premise holds; that
rationale is recorded rather than re-measured here.

## Validation (all on this box; logs under `artifacts/`)

- `cargo check --no-default-features --features pg18` — clean.
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
  — clean.
- `cargo test --no-default-features --features pg18 --lib lazy` — **13 passed**
  (6 lazy unit + 7 Task-112 pg_test cases).
- `cargo test --no-default-features --features pg18 --lib 'am::common::explain::tests'`
  — **9 passed**.
- The acceptance-criterion tests are all green: early stop / counters
  (`..._counters_no_skip_under_no_bound`), ties
  (`..._handles_score_ties`), duplicate heap TIDs
  (`..._dedupes_duplicate_heap_tids`), empty frontier
  (`..._empty_frontier_is_a_noop`), `rerank_width` boundaries
  (`..._width_boundaries`), and the **equivalence test**
  (`..._equals_fixed_width`) proving lazy top-k == fixed-width top-k
  (identical outputs, scores, heap rows, and bytes).

Pre-existing unrelated `pg_test_ec_spire_*` failure on this checkout is not
touched (Task 112 changes nothing under `ec_spire`).

## Deferred to the bench host (Phase 5)

`artifacts/task-112-lazy-rerank-ab.intel-local.json`: a justified non-standard
`ecaz bench suite` config running the standard ec_ivf nprobe sweep
`[8,16,24,32,48,64]` at 100k on a heap-f32 coarse_rerank index, A/B'd by the
`ec_ivf.lazy_heap_rerank` session GUC (on vs off), emitting every Evidence
Requirements field. Expected outcome under the current contract: byte-identical
recall/ordering and equal heap rows/blocks/latency between arms (skipped == 0) —
recall-neutrality confirmation now, live win measurement once Task 113 lands a
bound. See `artifacts/manifest.md` for the run command.

## Recommendation

Iterate. Land Phases 1–4 (gate + contract + instrumentation + provably-safe
stop). The lazy stop is recall-neutral today by construction; the latency win is
gated on Task 113's calibrated bound + a `k`-cap/on-demand-suffix follow-up.
Requesting a reviewer decision on: (a) the safe-stop contract and NoBound
framing, (b) the `LazyRerankBound`/`k`-cap seam as the right shape for 113, and
(c) the deferred bench config.
