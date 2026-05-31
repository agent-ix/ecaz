# Review Request: Task 75 Phase 2 Decision

## Scope

This packet records the Phase 2 decision from the Task 75 local Intel candidate
funnel packet:

- Benchmark packet: `benchmarks/task75-intel-local-routing-envelope/manifest.md`
- Summary: `benchmarks/task75-intel-local-routing-envelope/artifacts/summary.md`
- Diagnostics/code packet: `reviews/task-75/001-candidate-funnel-diagnostics/request.md`
- Evidence head: `f9554974120fba58cf6f9fe22ffde37bb99d5bfa`

## Funnel Readout

At the matched-recall SPIRE point (`tg96/tg128 b0`, nprobe `96`):

- recall@10: `0.9975`
- p50: `131.292 ms` at tg96, `134.271 ms` at tg128
- leaf candidates: `2,784,952` over 200 queries
- retained to heap rerank: `5,000` over 200 queries
- returned to top-k: `2,000` over 200 queries
- retained/candidate ratio: `0.18 %`

The candidate envelope is therefore not tight, but the packet does not prove a
safe routing slice: the rows that fail to reach the heap are only known after
quantized scoring, and the tg64/tg96 comparison shows that equal candidate
counts do not imply equal recall (`0.9825` vs `0.9975`).

## Ranked Phase 2 Decision

1. **Score-bound early termination**: shelved for Task 75. This is the most
   plausible future win because only `0.18 %` of candidates survive to heap
   rerank, but pushing a heap bound into per-leaf quantized scoring needs a
   correctness proof that approximate-score ordering cannot drop exact winners.
   That proof is not present in Phase 1, and landing it here would exceed the
   measurement-first scope.
2. **Adaptive nprobe collapse**: shelved for Task 75. The tg64/tg96/tg128 rows
   show a saturated aggregate candidate count, but recall still improves from
   tg64 to tg96. Collapsing nprobe from the aggregate count alone would regress
   the Task 73 recall floor.
3. **Tighter recursive draft**: shelved for Task 75. The funnel proves many
   selected-leaf rows are discarded, but it does not identify a routing-level
   predicate that can reject those leaves before scoring without changing
   recursion semantics.

## Closeout Direction

Task 75 should close as measurement-only: the funnel exposes the cost source,
but Phase 1 does not surface a clean, semantics-preserving `>=10 %` p50 routing
slice. Task 76 owns the defaults/Pareto question that can choose a lower-latency
point such as tg32/tg64 without pretending to preserve the high-recall envelope.

## Validation

No code changes in this packet. The underlying code/benchmark validation remains:

```bash
cargo test -p ecaz-cli spire_pipeline --no-default-features
cargo build -p ecaz-cli --no-default-features
target/debug/ecaz bench suite run --config benchmarks/task75-intel-local-routing-envelope/suite.json --database task75_spire_gate --host /home/peter/.pgrx --port 28818 --manifest-output benchmarks/task75-intel-local-routing-envelope/artifacts/suite-manifest.json --log-file benchmarks/task75-intel-local-routing-envelope/artifacts/suite-run-rerun-port28818.log
```
