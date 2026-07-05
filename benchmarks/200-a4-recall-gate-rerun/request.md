# Review Request: A4 Recall Gate Rerun

Commit: `52974ab`

## Summary

- rerun A4 on repaired regular-table `10k x 1536-dim x 4-bit` fixtures after tightening the graph
  build/runtime path and the in-tree recall harness
- keep the structural fixes that materially improved the `1k` diagnostic lane:
  - fixed-width per-layer neighbor slot persistence
  - layer-aware neighbor slicing at runtime
  - upper-layer greedy descent before layer-0 search
  - staged ordered graph results instead of emitting directly from the evolving traversal frontier
- leave behind faster repeatable recall fixture loading plus SQL-callable summary/probe surfaces for
  future A4 reruns
- result: A4 still fails at the required gate, and the repaired `10k` evidence shows the exact
  quantized path on this corpus is also far below the intended `89%` threshold

## Experiment Log

- `1k` fixture loader iteration:
  - row-by-row SPI inserts made the fixture too slow to use interactively
  - batching corpus inserts in groups of `32` reduced `1k` fixture reset time from roughly `92s`
    to roughly `6-7s`
- `1k` runtime diagnostics after the structural fixes:
  - graph Recall@10 over `100` fixed queries at `m=8`:
    - `ef=40`: `0.394`
    - `ef=128`: `0.489`
    - `ef=200`: `0.504`
    - `ef=400`: `0.513`
    - `ef=800`: `0.517`
  - exact quantized overlap on the same fixture stayed at `0.523`
  - build-code proxy overlap stayed at `0.465`
  - takeaway: the runtime fixes helped materially, and pure build-vs-search score divergence was no
    longer the primary blocker on `1k`
- `10k` harness regression check:
  - an `UNLOGGED` table optimization attempt produced `0` graph recall and zero emitted results
  - metadata snapshots on that path showed `dimensions=0` and `tree_height=0`
  - that path was reverted before accepting any A4 result
- repaired `10k` regular-table rerun:
  - graph Recall@10 over `100` fixed queries:
    - `(m=8, ef=40)`: `0.084`
    - `(m=8, ef=128)`: `0.218`
    - `(m=8, ef=200)`: `0.268`
    - `(m=16, ef=200)`: `0.353`
  - diagnostic `m=8, ef=800`: graph reaches `0.391`
  - exact quantized overlap on the same `10k` corpus: `0.431`
  - build-code proxy overlap on the same `10k` corpus: `0.394`
  - takeaway: there is still a real graph-runtime budget gap at the required `ef_search` values,
    but the larger finding is that the exact quantized path on this corpus is also nowhere near the
    intended `0.89` gate

## What Worked

- batched fixture inserts made repeated A4 reruns practical enough to use during debugging
- fixed-slot neighbor persistence plus layer-aware reads repaired the on-disk/runtime agreement and
  flushed out stale compact-neighbor test assumptions
- upper-layer greedy descent and staged ordered-result draining removed the worst early graph-first
  pathologies that were showing up on `1k`
- the new SQL-callable recall helpers in `src/lib.rs` make it cheap to compare graph overlap, exact
  quantized overlap, and build-code overlap on the same fixture

## What Did Not Work

- the `UNLOGGED`-table fixture path was invalid for A4 and produced unusable index metadata
- the current graph runtime still misses the A4 gate badly at the required budgets
- additional graph-runtime repairs alone cannot clear the stated `89%` gate on this exact corpus
  and quantized path, because the exact quantized overlap itself is only `43.1%`

## Current Recommendation

- keep A4 open and keep planner activation, insert, vacuum, and SIMD merge blocked behind it
- treat the next A4 step as a project decision, not just more local scan refactoring:
  - either change the measurement path / dataset assumptions behind the `89%` gate
  - or improve the underlying quantized path enough that the exact ceiling moves materially upward
- if more runtime work is chosen before changing the gate assumptions, it should be justified
  against the remaining gap between graph and exact on the repaired `10k` fixture, not against the
  original `89%` target directly

## Three A4 Hypotheses

These are the three high-level hypotheses to reference in follow-up work:

1. `H1: graph-runtime gap`
   - The live graph path is still leaving recall on the table relative to exact quantized search at
     the required `ef_search` budgets.
   - Current evidence: graph reaches `21.8%` at `(m=8, ef=128)` while exact quantized is `43.1%`
     on the same repaired `10k` corpus.

2. `H2: quantized-objective mismatch`
   - The exact `tqvector` scorer may simply not match the fp32 Recall@10 gate closely enough on the
     current `10k x 1536 x 4-bit` synthetic corpus.
   - Current evidence: even exact quantized search is only `43.1%` against fp32 truth.

3. `H3: quantized-path implementation defect`
   - The exact quantized ceiling may be low because of a real implementation bug or mismatch in the
     quantized path itself, rather than because `4-bit` quantization on this corpus is inherently
     limited.
   - Existing sub-hypotheses to test under this bucket are the review probes in `195-198`
     (score-function divergence, neighbor-slot packing, gamma/build mismatch, visited/refill state).

## Validation

- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

## Review Focus

- whether the experiment trail above is a fair reading of what the `1k` and repaired `10k`
  evidence actually says
- whether the new recall summary/probe surfaces are the right durable A4 debugging seam to keep
- whether the current recommendation is the right one: A4 still blocks downstream work, but the
  next decision is likely about the gate/measurement path, not only more graph traversal churn
