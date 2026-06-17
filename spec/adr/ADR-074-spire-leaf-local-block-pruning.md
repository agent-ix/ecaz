---
type: ADR
id: ADR-074
title: "SPIRE Leaf-Local Block Pruning"
status: ACCEPTED
impact: Affects Task 79, SPIRE leaf V2/V3 storage, RaBitQ scan latency, and NFR-016 on-disk format evolution.
date: 2026-06-04
---
# ADR-074: SPIRE Leaf-Local Block Pruning

## Context

Task 79 measured SPIRE's high-recall local RaBitQ path and found that the
problem is not the final heap frontier. The scan routes too many row payloads
into candidate scoring:

- Task 78-style `nlists=128`, `nprobe=96` scores `15,506,227` candidate rows
  over 200 queries at recall@10 `0.9975`.
- Increasing global leaf density reduces candidates but loses recall:
  `nlists=512`, `nprobe=128` scores `5,337,119` rows at recall `0.9645`;
  `nlists=1024`, `nprobe=96` scores `2,152,562` rows at recall `0.9265`.
- Boundary replicas recover some recall only by adding rows back into the
  scored surface.
- Route-time row budgeting gets the best near-frontier to `5,231,408`
  candidates at recall `0.9940`, but p50 remains `58.153 ms` and the task
  candidate gate is still missed.

The accepted Task 79 packet 004 conclusion is therefore that the remaining
granularity problem is inside selected leaves: once a high-quality leaf is
selected, the current scan reads and scores every visible row in that leaf.

## Decision

Pursue a **query-aware leaf-local block pruning** design for SPIRE. The
implementation target is a new leaf storage version that stores compact,
scoreable block summaries separately from row payload segments:

1. During build, partition each leaf's rows into deterministic subleaf blocks.
   The natural first block unit is the existing leaf V2 physical column segment
   size, or a small multiple of it if measurement shows segment summaries are
   too fine.
2. For each block, derive a summary vector from the source vectors in that
   block and encode it with the same assignment payload format as the leaf
   (`rabitq` primary; `turboquant` only for comparison).
3. Store the block summary table in a metadata-reachable side chain, not inside
   the row segment payload. The scan must be able to score summaries before
   reading row payload segments.
4. At scan time, route leaves exactly as packet 004 does, score block summaries
   inside each selected leaf, then read and score only selected row blocks.
5. Preserve a full-leaf fallback for old leaf versions, missing summaries,
   disabled session settings, or diagnostics.

This is intentionally different from a fixed leaf-prefix cap. A fixed prefix is
not query-aware; it assumes the best rows for all future queries live near one
leaf centroid ordering. The block-pruning path scores per-query block summaries
before selecting rows.

## Storage Contract

The implementation uses a reject-unknown format bump consistent with ADR-070
Option A:

- SPIRE leaf V2 remains the row-payload-only fallback format;
- SPIRE leaf V3 is the single-representative block-summary leaf format;
- SPIRE leaf V4 is the multi-representative block-summary leaf format used by
  RaBitQ summaries that need more than one representative payload per block;
- summary metadata is encoded with explicit little-endian fields and exact length
  checks;
- summary payloads are stored in a chain reachable from the leaf meta object,
  separate from row payload segments;
- the summarized leaf meta records block count, rows per block, payload format,
  payload stride, summary representative count, summary bytes, and the first
  summary-chain locator;
- malformed summary metadata or summary chains are rejected before row-segment
  pruning is enabled.

The scan path must not require reading all row segments to choose blocks. If an
early prototype stores summaries inside row segments, it may be used only as a
negative measurement or scoring-kernel probe, not as the Task 79 closing design.

## Runtime Control

The feature remains disabled by default until Task 79 evidence clears the
candidate and latency gates. The scan control surface should expose:

- a session GUC and suite field for a per-leaf or per-query selected block
  budget;
- diagnostics for selected leaves, available blocks, selected blocks, skipped
  row blocks, scored row candidates, summary-score time, and row-score time;
- a fallback mode that forces full selected-leaf scans for A/B validation.

## Alternatives Considered

### Increase `nlists`

Rejected as the primary closing direction for Task 79. Packet 001 shows larger
global partitions reduce candidates, but recall falls well below the high-recall
floor and `nlists=2048` increases p50 despite scoring only `1,148,089` rows.

### Boundary Replicas

Rejected for this fixture. Packet 002 shows replicas spend candidates to recover
recall. They move opposite the Task 79 candidate-surface gate.

### Route-Time Row Budget Only

Useful and already accepted in packet 004, but not sufficient. Whole-leaf
budgeting can choose better high-quality leaves at a constant row budget, but
the selected row unit is still too coarse.

### Fixed Leaf Prefix or Build-Time Row Ordering

Rejected as a closing design. It can reduce candidate counts mechanically but
does not score the query against row groups before pruning and therefore has no
accuracy-preserving argument.

### RaBitQ Early Cutoff Only

Useful as a scorer micro-optimization, but it does not reduce selected row
surface. The task gate is about avoiding millions of row candidates before the
final heap frontier.

## Consequences

### Positive

- Directly attacks the remaining selected-leaf row surface.
- Preserves the high-quality leaf selection behavior from packet 004.
- Makes candidate reduction query-aware instead of relying on global row order.
- Gives TurboQuant a clear comparison point after RaBitQ has a candidate recipe.

### Negative

- Requires a SPIRE leaf format bump and upgrade/fallback discipline.
- Adds build cost and storage overhead for block summaries.
- Adds a new tuning surface for block budget and block size.
- Must prove summary scoring plus selective row reads beats simply increasing
  global leaf density.

## Acceptance Criteria

The Task 79 implementation packet for this ADR must show, on the 100k real
corpus / 200-query RaBitQ lane:

- recall@10 at or above `0.9925`, or a clearly better Pareto point;
- scored row candidates `<=5.2M`, with `<=4.0M` as the strong target;
- p50 `<=45 ms` or at least 25% better than the `60.256 ms` Task 78 baseline;
- retained and returned counts comparable to the baseline unless the packet
  explicitly changes rerank width and proves the tradeoff;
- summary object bytes, row object bytes, summary-score time, row-score time,
  selected block counts, and skipped row-block counts;
- a TurboQuant comparison only after a RaBitQ row is close enough to defend.
