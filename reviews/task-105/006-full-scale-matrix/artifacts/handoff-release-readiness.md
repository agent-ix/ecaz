# Handoff: measured foundation for the safety/cleanup/release-readiness track

Task 105 closes the performance-evidence phase. This note is the
contract between that evidence and the next track.

## What is now measured and standing (do not re-run casually)

- **Full-scale matrix**, 10k/50k/100k/1M × 16 families × kernel on/off
  × G4/Intel: `full-scale-matrix.md` (this packet), raw evidence in
  `reviews/task-105/004-g4-lane/` + `005-intel-lane/`, 100k full
  profile in `reviews/task-99/008+009`.
- **Production defaults are measured-in**: aarch64 NEON-first dispatch
  and IVF batch decode default-on (`main=1345ca603`, ADR-077 §4/§6),
  confirmed on-instance (G4 gate 32/32, kernel rates within 0.7% of
  the NEON-cap reference, recall byte-identical).
- **Recall parity**: kernel on/off recall equal at every cell — the
  block-kernel surface does not change results anywhere it engages.
- **Baseline position**: ivf-rabitq1 @1M beats the May gate −18% p50 at
  matched ~25% scan fraction and clears the pinned vchord bar at 1M
  (5.0× / 1.4× at the two operating points). `baseline-comparison.md`.
- **Snapshots** (us-west-2): G4 `snap-0f546929f70d60fb5`, Intel
  `snap-0338adc6455257604` (all t105 fixtures at all scales,
  `main=1345ca603` installed), corpus base `snap-0e9c7743263e61d70`.
  Both stacks destroyed 2026-06-12; zero instances/volumes left.
  Re-running any cell = restore snapshot + `cloud install` at the head
  under test; fixtures need no rebuild unless the on-disk format
  changes.

## Known gaps the next track must respect (none are release blockers by themselves, all are documented)

1. **Task 106 (proposed)** — four unified-driver coverage gaps
   (ADR-077 §9): SPIRE×RaBitQ batch migration (toggle currently
   inert), HNSW×grouped-PQ engagement decision, IVF×TQ-QJL engagement
   diagnosis, SPIRE×pq_fastscan product gap (index unbuildable).
   Scheduled as a targeted pass after Task 105 review closes.
2. **IVF kernel-off baselines at t105 scales** do not exist post-flip
   (the sweep's off arm became same-config after the default flip).
   Task 99's 100k A/B is the differential record. If the release
   narrative needs fresh IVF differentials at 1M, it is one
   snapshot-restore + a small explicit-`off` suite away.
3. **Variance-flagged cells** (do not cite as trends): Intel 1M HNSW
   mixed-direction deltas; Intel diskann-pqfs-binary @64; Intel 50k
   ivf-rabitq1 same-config gap. See honest markers in
   `full-scale-matrix.md`.
4. **diskann-pqfs-grouped-pq at 1M** (~310–324 ms) is not
   production-competitive; if it ships as a supported config it needs
   either tuning guidance or a documented "small-scale only" position.
5. **QJL-1024** remains a 100k dispatch-correctness column, not a
   production recall configuration.

## Recommended framing for release material

- Headline families per AM at 1M (G4, kernel-on, p50 @ low point):
  diskann-tq 4.66 ms (recall 0.961), hnsw-tq-default 8.4 ms (0.867),
  ivf-rabitq1 16.1 ms (0.926) / 56.8 ms (0.980), spire-tq 62.3 ms
  (0.952).
- The kernel program's e2e value is concentrated where scoring share
  is high (SPIRE TQ, DiskANN TQ, IVF families); families with <5%
  scoring share get parity-not-speed from the kernels — say so rather
  than implying universal speedups.
- Comparator claims must cite the pinned packet and its caveats
  (latency-only vchord row, untuned others). No comparator re-runs
  without a version/hardware change per the standing rule.
