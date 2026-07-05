# Review Request: Subleaf Block Pruning Design

## Scope

This is the required Phase 4 design packet for Task 79. It does not change
production code. It records the storage and scan direction before implementing
leaf-local candidate pruning.

New design artifact:

- `spec/adr/ADR-074-spire-leaf-local-block-pruning.md`

## Decision

Proceed with query-aware leaf-local block pruning as the next implementation
slice.

The intended implementation stores scoreable subleaf/block summaries in a leaf
metadata-reachable chain, then scores those summaries per query before reading
and scoring row payload blocks. The feature stays disabled by default and falls
back to full-leaf scans for old leaf versions, missing summaries, diagnostics,
or explicit session settings.

## Why This Is the Right Next Slice

The accepted Task 79 evidence has already bounded the alternatives:

| direction | result |
| --- | --- |
| Increase `nlists` | Candidate count falls, recall misses badly; `nlists=2048` also regresses p50. |
| Boundary replicas | Recover recall by adding candidates back, so they move opposite the gate. |
| Route-time row budget | Correctly caps whole-leaf routing, but best high-recall row is still `5,231,408` candidates at p50 `58.153 ms`. |
| Fixed leaf prefix | Not query-aware; it can reduce rows mechanically but has no recall-preserving argument. |

The remaining problem is the whole-leaf unit: once a leaf is selected, current
scan still reads and scores every visible row in that leaf. Subleaf block
summaries are the direct mechanism that reduces rows inside selected high-recall
leaves.

## Design Commitments

- RaBitQ remains the primary/default lane.
- TurboQuant is comparison-only after RaBitQ has a defensible row.
- The first implementation should add a reject-unknown SPIRE leaf V3 format
  rather than overloading leaf V2 reserved bytes.
- Summary metadata must be reachable before row segment reads; storing summaries
  inside row segments is not enough for the latency gate.
- The suite must report selected leaves, available blocks, selected blocks,
  skipped row blocks, scored row candidates, summary-score time, row-score time,
  object bytes, recall, and latency.

## Review Focus

Please review whether ADR-074 satisfies Task 79's Phase 4 prerequisite and
whether the proposed storage contract is the right implementation target before
code starts.

## Validation

Docs-only checkpoint. No tests run for this packet.

