# Task 124 Packet 027 Artifact Manifest

- head SHA: `098d432048bfe56ef2d3d534d753f1c4fb3dc92f`
- task bucket: `reviews/task-124/027-speed-closeout-shelve`
- timestamp: `2026-06-30T03:58:44Z`
- lane: local PG18 evidence cited from prior packet-local artifacts
- runner: `ecaz bench suite` for cited benchmark packets
- quant/index: `ec_ivf`, TurboQuant stage-2 over RaBitQ coarse frontier
- isolation: cited suites used one fresh index per table/prefix where noted in
  their packet manifests
- purpose: close Task 124 as shelved under the corrected TQ speed objective

## Source Packets

This packet does not introduce a new benchmark run. It is a closeout packet
summarizing previously committed Task 124 evidence:

| Packet | Evidence role |
| --- | --- |
| `reviews/task-124/026-f32-vs-tq-nprobe60-discriminator/` | Final 10k / 50k / 100k f32@60 vs TQ@60 discriminator; 24 suite steps passed. |
| `reviews/task-124/025-tq-selected-slab-vector/` | Rejected 10k / 50k / 100k TQ selected-payload lookup experiment; 18 suite steps passed. |
| `reviews/task-124/020-tq-borrowed-score-buffer/` | Rejected TQ borrowed score buffer experiment. |
| `reviews/task-124/018-tq-selected-index-vector/` | Rejected TQ selected index vector experiment. |
| `reviews/task-124/019-phase6-evidence-correction/` | Corrected the invalid local macOS cold-cache interpretation. |
| `reviews/task-124/017-speed-objective-correction/` | Reopened Task 124 under the corrected TQ speed objective. |
| `reviews/task-124/001-tq-stage2-engine-slice/` and `002-tq-stage2-attribution-counters/` | In-engine TQ stage-2 implementation and counters. |

## Key Cited Result Lines

Packet `026` final discriminator:

- 10k f32/source: recall `1.0000`, p50/p95/p99 `1.22 / 1.38 / 1.43 ms`,
  index `2.9 MiB`.
- 10k TQ final15: recall `1.0000`, p50/p95/p99 `1.13 / 1.28 / 1.37 ms`,
  index `10.9 MiB`, scalar candidates `0`.
- 50k f32/source: recall `1.0000`, p50/p95/p99 `4.48 / 5.32 / 5.83 ms`,
  index `11.6 MiB`.
- 50k TQ final15: recall `0.9980`, p50/p95/p99 `4.23 / 4.47 / 4.54 ms`,
  index `50.9 MiB`, scalar candidates `0`.
- 100k f32/source: recall `1.0000`, p50/p95/p99 `9.46 / 9.76 / 9.92 ms`,
  index `22.5 MiB`.
- 100k TQ final15: recall `1.0000`, p50/p95/p99 `8.77 / 9.01 / 9.22 ms`,
  index `100.8 MiB`, scalar candidates `0`.

Packet `025` selected-slab-vector negative result:

- 10k cap-off worsened, 10k cap60 slight tail win.
- 50k cap-off worsened, 50k cap60 improved.
- 100k cap-off worsened, 100k cap60 materially worsened in tail
  (`8.59 / 8.85 / 9.03 ms` baseline vs `8.99 / 10.1 / 12.0 ms` experiment).
- TQ scorer remained full SIMD with `scalar_candidates=0`.

Packets `018` and `020` TQ component speed attempts:

- selected-index vector lookup regressed 100k latency at both nprobe points.
- borrowed score buffer did not improve 100k latency at either nprobe point.

## Closeout Interpretation

The final discriminator is negative for the only remaining possible nprobe-based
TQ claim: f32/source also holds recall at `nprobe=60`. Therefore the speed delta
from that frontier setting is not a TurboQuant optimization. Combined with the
negative TQ-specific speed experiments and the established TQ storage gap, this
supports closing Task 124 as Shelve rather than continuing more narrow
micro-optimization on the same design.
