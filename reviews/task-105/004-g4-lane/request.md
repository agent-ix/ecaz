# Review request: Task 105 G4 lane — full-scale sweep evidence

- Task: `plan/tasks/105-production-optimization-full-scale-sweep.md`
- Packet: `reviews/task-105/004-g4-lane/`
- Code under measurement: `main=1345ca603` (Phase 1 optimization slices,
  PR #31), release backend
- Host: AWS m8g.2xlarge (Graviton 4, Neoverse V2 sve2-128), profile
  `10k-medium`, us-west-2

## Summary

The Graviton 4 lane of the Task 105 full-scale sweep is complete and
green at every scale:

- **100k dispatch-confirm gate (clean re-run): 32/32** — kernel cells
  attribute `isa=neon` only (NEON-first dispatch per ADR-077 §6), kernel
  rates within 0.7% of the Task 99 NEON-cap reference.
- **10k sweep: 71/71**, **50k sweep: 71/71** (warm citable run + cold-cache
  datum kept per quiet-host protocol), **1M sweep: 71/71** (3.5 h,
  13:37–17:08 UTC 2026-06-12).
- 14/14 recall on/off pairs equal at 1M (28/28 across smaller scales in
  their own manifests); zero `avx2`/`sve` attribution anywhere in the lane.
- Headline 1M kernel on/off p50: SPIRE TQ −22.6%, DiskANN TQ −13.1%.
  IVF on/off pairs are same-config noise-floor pairs post-ADR-077 §4
  default flip (off arm omits the flag instead of forcing the GUC off);
  the IVF differential evidence is Task 99's explicit A/B. Full numbers
  and per-directory provenance in `artifacts/manifest.md`.

## Response to feedback 2026-06-12-01 (codex)

1. **Lane-clean artifacts (blocking #1): fixed.** The committed
   `sweep-1m/` directory had been contaminated by the local artifact sync
   (23/30 latency logs were stale Intel copies, some `isa=avx2`). The
   remote run itself was verified sound (per-step manifest timestamps;
   all 287 `results.jsonl` rows distinct from Intel). The directory was
   rebuilt from a fresh `aws s3 cp --recursive` of the S3 run prefix and
   re-verified: 71/71, `isa=neon`/`scalar` only, no Intel-identical
   non-deterministic files. Method and gates in `artifacts/manifest.md`
   ("sweep-1m rebuild provenance"). The same scan over gate-clean/10k/50k
   directories found no contamination.
2. **Packet provenance (blocking #2): fixed.** This `request.md` and
   `artifacts/manifest.md` added; the manifest maps every sweep dir to its
   suite, config sha, S3 run prefix, step count, and window, and documents
   that `results.jsonl` `artifact` fields are remote config-relative paths
   with the packet-local copies canonical.
3. **Task-level completion (blocking #3): in progress, not claimed.** The
   full-scale matrix document, prior-baseline comparison, and
   release-readiness handoff land in the Phase 3 packet; Task 105 stays
   open until then. G4 stack teardown: snapshot
   `snap-0f546929f70d60fb5` is now `completed`; bucket empty + destroy
   executed immediately after this packet commit (per the standing
   operator instruction "full sweep up to 1m, then shut down the
   instance").

## Artifacts

- `artifacts/manifest.md` — per-directory provenance (source of truth)
- `artifacts/gate-clean/` — 100k dispatch-confirm gate (32/32)
- `artifacts/sweep-10k/`, `artifacts/sweep-50k-warm/`,
  `artifacts/sweep-50k-coldcache-datum/`, `artifacts/sweep-1m/` — full
  sweeps (71/71 each)
- `artifacts/day1-smoke.log`, `artifacts/fixtures-10k-50k.log` — host
  preflight and staged fixture builds
