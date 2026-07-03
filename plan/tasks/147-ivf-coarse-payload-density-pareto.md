# Task 147: IVF coarse-payload density pareto at the new defaults (1/2-bit + exact rerank vs 4-bit TQ)

Status: **in progress** (2026-07-03). Owner: Codex (branch
`task-146-outside-scan-profile` lane). Priority: P2.
Reopens the Task 96 premise as a measurement question; informed by the
Task 115/122 rerank-masking insight and the Task 143/145 stage budgets.

## Why

Under the promoted dense+int8 defaults, parse+push and page access are
~44% of the 1m approximate scan, and Task 135 packet 001 proved both
row sub-stages sit at per-unit floors — the recorded lever is posting
COUNT/BYTES. A denser coarse payload halves or quarters both stages'
work. The recall risk is bounded by a measured cross-cutting fact:
**exact heap-f32 rerank masks coarse-quantizer error** (1-bit recall ==
4-bit recall under rerank; the mechanism that nulled Task 115's
residual win).

Phase-0 inventory (this task): TurboQuant's coarse encode is hardwired
to 4-bit (`IvfQuantizer::encode_source` uses `crate::DEFAULT_QUANT_BITS`;
`quant_bits` parameterizes RaBitQ only), and Task 96 (TQ no-QJL 2-bit
block kernels) was deferred for having no consumer. So "2-bit TQ" needs
new encode + scan + kernel surface — NOT justified before the density
hypothesis is proven. The existing surface that tests the same
hypothesis with zero new code: **RaBitQ `quant_bits={1,2}` +
`dense_posting_blocks=1` + `rerank='heap_f32'`** (Task 93 popcount
kernels, Task 111a dense layout, Task 111h source/f32 rerank — all
landed).

## Scope

- A/B/C matrix at 10k/50k/100k (1m for the winner if it beats or
  approaches the champion at 100k):
  - **A (champion)**: `storage_format=turboquant` pure defaults
    (dense + int8/SDOT, no rerank) — cells already measured in the
    Task 145/146 packets on the same binary lineage; reuse.
  - **B**: `storage_format=rabitq, quant_bits=1, dense_posting_blocks=1,
    rerank='heap_f32'` (default rerank_width 50).
  - **C**: same with `quant_bits=2`.
- Compare recall + latency + storage; stage counters on, so the
  pages/parse reduction and the rerank cost are separately visible.
- Decision output: if B or C beats A on the latency/recall pareto,
  file the implementation follow-ups (Task 96 revival for a TQ 2-bit
  scan payload and/or coarse_rerank promotion per the 111e family);
  if A holds, record the source-grounded negative and close the
  density direction at the coarse stage.

## Out of Scope (hard)

- No new quantizer/kernel/format code in this task — measurement only.
- No default changes.

## Gate / Exit Criteria

- The matrix above with per-cell recall/latency/storage evidence in the
  packet, and an explicit pareto verdict vs the champion.
