# Task 97 Packet 012 Artifact Manifest

- head SHA: `fccc4e0cfe3b0921dc7956b836b53db777b211f9`
- task bucket: `reviews/task-97/012-status-through-packet-011`
- lane: Task 97 TurboQuant QJL block kernel
- fixture: status-only packet; no new benchmark run
- storage format: `turboquant`
- rerank / exact mode: production QJL (`MseLutQjl`)
- AWS / CI: not run

## Scope

This packet updates canonical Task 97 status text after packet 011's local scoring-ladder evidence.

Updated files:

- `plan/tasks/97-tq-qjl-block-kernel-family.md`
- `plan/tasks/README.md`

## Evidence Carried Forward

- Packet 011 request: `reviews/task-97/011-local-scoring-share-ladder/request.md`
- Packet 011 manifest: `reviews/task-97/011-local-scoring-share-ladder/artifacts/manifest.md`
- Packet 011 head SHA: `c07590302f2467cc2b52f84fb856acd3c612688c`

Packet 011 is measurement-only and reports same-head local evidence for the corrected Task 97 QJL fixture (`dim=1024,bits=4,seed=42`). Its current local scoring-ladder result is below the Task 97 performance floor:

- SPIRE direct counter scoring: `0.93x` at `nprobe=8`, `0.92x` at `nprobe=16`
- IVF end-to-end: `0.97x` at `nprobe=8` and `nprobe=16`
- SPIRE end-to-end: `1.01x` at `nprobe=8` and `nprobe=16`
- HNSW end-to-end: `0.93x` at `ef_search=32`, with only scalar-tail direct rows under the local `m=8` fixture

## Remaining Gates

- Packet 011 review.
- Project/reviewer disposition on whether Task 97 proceeds with a separate qjl32 AVX2 optimization slice or accepts a stop condition for the current QJL performance state.
- Graviton 4 runtime dispatch/vector-length/counter evidence when AWS testing is approved.
- Final Task 97 closeout matrix.

## Validation

- Status/documentation-only packet.
- No code changed.
- No tests, GitHub CI, or AWS runs were used.
