# Task 120 Packet 016 Artifact Manifest

- head SHA: `234f35501877854386fc58191f6494b78efe9c7f`
- task bucket: `reviews/task-120/`
- packet path: `reviews/task-120/016-phase6-maintenance-fallback-invariants/`
- timestamp: `2026-06-22T05:20:57Z`
- lane: documentation/invariant record
- fixture: not applicable
- storage format: not applicable
- rerank mode: not applicable
- surfaces: no benchmark surface; no durable format/default promoted
- isolated one-index-per-table or shared-table: not applicable

## Artifacts

- `phase6-invariants.md`
  - command/source: written from the Task 120 Phase 6 acceptance criteria and
    existing Task 120 packet outcomes.
  - key result: records conservative insert/delete/vacuum/split/rebuild,
    mixed-version, stale/malformed summary, and remote worker version-skew
    fallback invariants before any durable SPIRE coarse-rerank behavior is
    promoted.

## Supporting Review Packets

- `reviews/task-120/008-phase2-rabitq-block-pruning/`
  - measured local leaf block-pruning negative result; no product default.
- `reviews/task-120/010-phase3-budget-policy/`
  - measured candidate/rerank budget curves; knobs kept diagnostic-only.
- `reviews/task-120/011-phase4-route-overfetch/`
  - measured local route-overfetch recall recovery; carried to AWS only as a
    hypothesis.
- `reviews/task-120/015-phase5-aws-distributed-rerank/`
  - partial AWS distributed SPIRE evidence; production-read shipping/merge
    metrics still incomplete.

## Packet Notes

- No tests or benchmarks were run for this packet because it changes no code and
  promotes no durable behavior.
- No corpus/query/truth TSVs, truth-cache, SSM/tunnel state, raw per-query
  JSONL, or AWS operational exhaust are committed.
- This packet is not Task 120 closeout; it only satisfies the Phase 6 invariant
  record before any future promotion decision.
