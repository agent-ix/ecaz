# Task 124 Closeout: Shelve TurboQuant Stage-2 Pipeline

## Decision

Close Task 124 as **Shelve**.

TurboQuant stage-2 is now implemented and measured in-engine, and the task was fully focused on TQ. The result is not promotable:

- recall can match the current RaBitQ + f32 baseline;
- the TQ stage-2 scorer is full SIMD/NEON, not scalar;
- final f32 fetches can be reduced;
- but latency is not durably better at the product matrix;
- persisted TQ sidecar storage remains about 4.5x the f32/source ec_ivf index;
- Phase 6 IO-sensitive validation did not convert the source-read reduction into a clear latency win.

## Requirement Audit

| Task 124 requirement | Evidence | Closeout read |
| --- | --- | --- |
| Phase 0 SIMD/scalar audit | `001-tq-stage2-engine-slice/artifacts/tq-score-surface-audit.md`; packet 003/011/015 counter rows | Complete. The hot path is batch/SIMD; measured TQ rows report `scalar_candidates=0`. |
| Phase 1 engine/API path | `001-tq-stage2-engine-slice/request.md` | Complete. Disabled-by-default IVF path added for RaBitQ frontier -> TQ stage-2 -> bounded exact/source f32 final rerank. |
| Phase 2 persisted/index-side TQ payload | packets 001, 003, 011-014 | Complete enough for closeout. Index-side TQ sidecar worked and was optimized; deeper storage/materialization slices were tested and rejected. |
| Phase 3 counters/attribution | `002-tq-stage2-attribution-counters/request.md` | Complete. Stage-2 candidate/scored/retained rows, compact payload bytes, final exact rows, and final source bytes are exposed. |
| Phase 4 correctness/recall gates | packets 001, 002, 003, 005, 015 | Complete. Focused tests passed; recall matrices identify final10 as invalid and show matched recall for viable TQ configs. |
| Phase 5 10k/50k/100k benchmark matrix | `003-tq-stage2-ab-suite/` and follow-up matrix packets 005-008 | Complete. Recall/latency/storage measured via `ecaz bench suite`; no promotion win. |
| Phase 6 IO-sensitive validation | `015-tq-phase6-local-cache-evict/` | Complete. Local macOS relation `F_NOCACHE` run was negative/mixed, not a product win. |

## Why Not Promote

The original winning claim required matched recall, lower p50/p95/p99 latency, and a clear fetch/materialization/storage rationale against RaBitQ + f32.

The best measured path got recall alignment and full SIMD scoring, but failed the latency/storage bar:

- Packet 003 post-change 10k/50k/100k A/B: TQ stayed near parity but did not beat f32 consistently; 100k TQ p50/p95/p99 remained worse at both nprobe32 and nprobe64.
- Packet 005: final15 helped, but final10 broke recall and 100k/nprobe64 tail latency regressed.
- Packet 011: selected slab improved 100k latency slightly, but storage stayed `100.8 MiB` vs `22.5 MiB`.
- Packet 015: local IO-sensitive validation did not rescue the TQ thesis; TQ was worse at nprobe32 and mixed at nprobe64.

## What Was Explored

This was more than measurement:

- implemented the in-engine TQ stage-2 path;
- added TQ-specific counters;
- optimized selected payload loading;
- swept group width and final exact width;
- tried gamma/header byte reductions;
- tried binary/TQ2 byte-reduction formats;
- tried stage-2 width changes;
- kept the selected-payload slab improvement;
- tested and rejected top-k fusion, compact group headers, and direct slot addressing;
- ran the reviewer-requested Phase 6 local IO-sensitive validation.

The reviewer’s packet 011 directive asked for either a Phase 2 structural slice or Shelve after Phase 6. Packets 012-014 tested structural slices and reverted them on evidence; packet 015 supplied Phase 6 evidence. That resolves the fork.

## Landing Recommendation

Do not promote TurboQuant stage-2 as a product path from Task 124.

The code on this branch remains useful experimental infrastructure and evidence, but the closeout decision is that TQ stage-2 should not displace RaBitQ + f32 unless a future task takes on a larger storage redesign with a new premise. The canonical task file has been updated to `complete / shelved` and points at this packet.
