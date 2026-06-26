---
head_sha: ccafdbc1e922dcfd4881f62af413ae9713f80f84
task_bucket: reviews/task-89
packet: reviews/task-89/007-public-shape-defer-gate
timestamp_utc: 2026-06-26T04:12:05Z
---

# Artifact Manifest

## Scope

Task 89 Phase 6 public-shape gate recommendation. This packet introduces no new
benchmark run. It summarizes the already-committed evidence packets and requests
reviewer approval to defer TQ+ in its current IVF experimental shape.

## Source Evidence

- `reviews/task-89/001-ivf-tqplus-experimental-profile/feedback/2026-06-25-01-reviewer.md`
  - Reviewer approved ADR-081 direction.
  - Reviewer blocked closeout on latency comparability and cross-corpus
    evidence.
- `reviews/task-89/003-ivf-tqplus-dbpedia-suite/`
  - DBPedia 10k/50k/100k IVF no-QJL A/B evidence.
- `reviews/task-89/004-ivf-tqplus-qjl-projected-suite/`
  - Projected DBPedia 10k IVF QJL/gamma A/B evidence.
- `reviews/task-89/005-ivf-tqplus-insert-drift/`
  - DBPedia live-insert drift evidence at 10%, 25%, and 50%.
- `reviews/task-89/006-ivf-tqplus-cross-corpus/`
  - Deterministic synthetic non-DBPedia cross-corpus A/B evidence.

## Key Gate Facts

- DBPedia no-QJL representative recall is mixed:
  - 10k nprobe 48: TQ+ -0.50 pp.
  - 50k nprobe 64: TQ+ +0.30 pp.
  - 100k nprobe 96: TQ+ -0.60 pp.
- Projected QJL/gamma DBPedia 10k at nprobe 48: TQ+ +0.30 pp.
- Insert drift passes measured thresholds:
  - 25% insert: -0.05 pp live-minus-rebuild.
  - 50% insert: +0.25 pp live-minus-rebuild.
- Synthetic non-DBPedia recall regresses systematically:
  - nprobe 16: -0.45 pp.
  - nprobe 32: -2.95 pp.
  - nprobe 48: -5.00 pp.
  - nprobe 64: -7.30 pp.

## Latency Comparability

The gate recommendation does not rely on the existing latency deltas. Packet
001 reviewer feedback identified TQ+ as scalar-only while baseline TurboQuant
can use tiled/SIMD scoring. Until TQ+ has a comparable scorer, latency remains a
diagnostic implementation gap rather than promotion/defer evidence.

## Requested Review Outcome

Reviewer approval for:

- Defer TQ+ in the current IVF experimental form.
- Do not introduce a public `turboquant_tqplus` storage format.
- Do not promote `turboquant_calibration=tqplus_experimental` to a public
  operator option.
- Do not start SPIRE/HNSW/DiskANN ports from this evidence.
