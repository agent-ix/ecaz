# Review request: Task 105 Intel lane — full-scale sweep evidence

- Task: `plan/tasks/105-production-optimization-full-scale-sweep.md`
- Packet: `reviews/task-105/005-intel-lane/`
- Code under measurement: `main=1345ca603` (Phase 1 optimization slices,
  PR #31), release backend
- Host: AWS m7i.2xlarge (Sapphire Rapids), profile `10k-intel`, us-west-2

## Summary

The Intel lane of the Task 105 full-scale sweep is complete and green at
every scale: 10k 71/71, 50k 71/71, 1M 71/71 (3.8 h, 12:24–16:14 UTC
2026-06-12). The 100k optimization-confirmation gate for this host was the
Task 99 Intel profile run (clean IVF TQ −70.3/−68.9% deltas), per the
operator's confirm-before-expensive-work instruction.

- 14/14 recall on/off pairs equal at 1M; `isa=avx2`/`scalar` attribution
  only across the lane.
- Headline 1M kernel on/off p50: SPIRE TQ −37% @16 / −48% @64; DiskANN TQ
  −11% @64; HNSW rabitq / TQ full_lut −19%/−21% @ef=160. IVF on/off pairs
  are same-config noise-floor pairs post-ADR-077 §4 default flip (off arm
  omits the flag instead of forcing the GUC off); the IVF differential
  evidence is Task 99's explicit A/B at 100k.
- One flagged anomaly (not concluded): diskann-pqfs-binary @64 +26% vs
  @128 −7%, single noisy point — carried to the Phase 3 analysis.

Lane teardown is complete: stack destroyed 2026-06-12, end-state snapshot
`snap-0338adc6455257604` (completed). Bucket emptied — packet-local copies
are canonical. Full per-directory provenance in `artifacts/manifest.md`.

This packet was retro-filled with `request.md` + `artifacts/manifest.md`
in response to feedback `reviews/task-105/004-g4-lane/feedback/
2026-06-12-01-reviewer.md` (blocking #2); the same contamination scan that
rebuilt the G4 1M directory was run over this lane and found no
foreign-lane rows.

## Artifacts

- `artifacts/manifest.md` — per-directory provenance (source of truth)
- `artifacts/sweep-10k-clean/`, `artifacts/sweep-50k-quiet/`,
  `artifacts/sweep-1m/` — full sweeps (71/71 each)
- `artifacts/day1-smoke.log`, `artifacts/fixtures-10k-50k.log`,
  `artifacts/fixtures-1m-stage-a.log` — host preflight and staged fixture
  builds
