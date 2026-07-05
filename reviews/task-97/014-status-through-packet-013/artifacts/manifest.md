# Task 97 Packet 014 Artifact Manifest

- head SHA: `2b67525ed4152b18a29aa4888f8ffbc6bcdc2d2f`
- task bucket: `reviews/task-97/014-status-through-packet-013`
- lane: Task 97 TurboQuant QJL block kernel
- fixture: status-only packet; no new benchmark run
- storage format: `turboquant`
- rerank / exact mode: production QJL (`MseLutQjl`)
- AWS / CI: not run

## Scope

This packet updates canonical Task 97 status text after packet 013's current-head per-candidate scorer Criterion evidence.

Updated files:

- `plan/tasks/97-tq-qjl-block-kernel-family.md`
- `plan/tasks/README.md`

## Evidence Carried Forward

- Packet 013 request: `reviews/task-97/013-per-candidate-scorer-evidence/request.md`
- Packet 013 manifest: `reviews/task-97/013-per-candidate-scorer-evidence/artifacts/manifest.md`
- Packet 013 head SHA: `8742d7f2bca185262889038628ead2756c120da9`

Packet 013 adds the missing local Criterion row for the current-head production per-candidate QJL scorer at the Task 97 fixture:

- `quant/score_ip_from_parts/d1024_b4/1024`: `[874.53 ns 887.34 ns 904.33 ns]`

## Remaining Gates

- Packet 013 review.
- Packet 004 F1 old-vs-new disposition: packet 013 supplies the current-head row, but not the pre-`b0efa19d9` old multi-accumulator comparison.
- Project/reviewer disposition on whether Task 97 proceeds with a separate qjl32 AVX2 optimization slice or accepts a stop condition for the current QJL performance state.
- Graviton 4 runtime dispatch/vector-length/counter evidence when AWS testing is approved.
- Final Task 97 closeout matrix.

## Validation

- Status/documentation-only packet.
- No code changed.
- No tests, GitHub CI, or AWS runs were used.
