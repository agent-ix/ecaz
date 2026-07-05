# Task 97 Packet 010 Artifact Manifest

- head SHA: `48be35a5656341d99ea45f3fa09788f2b298d589`
- task bucket: `reviews/task-97/010-status-through-packet-009`
- lane: Task 97 TurboQuant QJL block kernel
- fixture: status-only packet; no new benchmark run
- storage format: `turboquant`
- rerank / exact mode: production QJL (`MseLutQjl`) is represented by the packet 009 local PG18 suite fixture `dim=1024,bits=4,seed=42`; standard `1536d/4-bit` is the no-QJL LUT32 lane and is not Task 97 QJL evidence
- AWS / CI: not run

## Scope

This packet records the reviewer/operator clarification that Task 97 evidence must use the current production QJL-active TurboQuant configuration:

- in scope: `dim=1024,bits=4,seed=42`, exact mode `MseLutQjl`
- out of scope for Task 97 QJL evidence: standard `1536d/4-bit`, because `qjl_enabled(dim,bits)` makes that cell the no-QJL LUT32 lane

It updates:

- `plan/tasks/97-tq-qjl-block-kernel-family.md`
- `plan/tasks/README.md`

## Evidence Carried Forward

- Packet 009 request: `reviews/task-97/009-local-qjl32-suite/request.md`
- Packet 009 manifest: `reviews/task-97/009-local-qjl32-suite/artifacts/manifest.md`
- Packet 009 code checkpoint: `70f6f2cf3c2f3c06a67139754242ce2c465d1f3e`

Packet 009 contains local PG18 direct `[block-kernel-counters]` evidence for IVF and SPIRE AVX2 block rows under `quant=turboquant_qjl`, and HNSW scalar-tail rows under `quant=turboquant_qjl` for the `m=8` fixture whose graph expansions do not reach block width 32.

## Remaining Gates

- Packet 009 review.
- Scoring-share closeout ladder against the Task 97 `1.5x / 1.8x / 2x` thresholds.
- Graviton 4 runtime dispatch evidence, including measured SVE vector length and `isa=sve2` counter rows, when AWS testing is approved.
- Final Task 97 closeout matrix.

## Validation

- Status/documentation-only packet.
- No code changed.
- No tests, CI, or AWS runs were used.
