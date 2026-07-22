# Artifact manifest — Task 194 packet 008

- Task bucket / packet: `reviews/task-194/008-nine-way-completion-audit/`.
- Implementation SHA: `1b5e201a9`.
- Canonical source suite config:
  `reviews/task-194/002-nine-way-attribution/artifacts/suite/task194-suite.json`.
- Immutable-output rerun config: `task194-nine-way-corrected.json`; it changes
  only packet/run paths, base port, and identifying labels from the source
  config so packet-002 evidence is not overwritten.
- Lane: local Intel, three isolated PG18 owner instances; trained exact
  landmark head, RaBitQ stored neighbor values, exact co-located row rerank,
  lazy10, BW=4/H=100.
- Protocol: 200 recall queries / 2,000 trials and 10 warmups + 50 measured
  latency iterations, stage/work attribution enabled.
- Validation:
  - strict normal PG18 clippy with warnings denied: passed;
  - strict PG18 attribution-feature clippy with warnings denied: passed;
  - focused reconciliation parser test: 1 passed.
  - canonical suite audit: passed, one step.
- Installed extension preflight: target and installed PG18 libraries are both
  24,271,128 bytes with SHA-256
  `1f08db214b8ed61e1197307754f343947d02327098a8b83190bea9fc5f21fdb7`.
- Planned command: `target/debug/ecaz bench suite run --config
  reviews/task-194/008-nine-way-completion-audit/artifacts/task194-nine-way-corrected.json
  --database tqvector_bench --log-file
  reviews/task-194/008-nine-way-completion-audit/artifacts/suite-run.log`.
- Run timestamp: 2026-07-22 03:06:25--03:39:50 PDT.
- Runner SHA: `e5f3e5995375bd9de1850200de2189659d326bf7`.
- Extension SHA: `07e38848ad072602026d247e7c23dc98b164b3a0`, unanimous
  `release` profile on all three owners.
- Suite result: one step succeeded in 2,089,063 ms; no failed, missing, or
  stale step.
- Isolation: one physical generation with one index per owner/source table,
  plus the fixture's separate same-data single-index control.

Operational node logs, fixture transcript, single-control raw logs, generated
corpus/truth data, and the runner transcript are not committed.

## Pre-run files

- `release-install.log`: release install transcript.
- `suite-audit.log`: suite shape/input audit.

## Decision artifacts

| Artifact | SHA-256 |
|---|---|
| `task194-nine-way-corrected.json` | `ef93772b302044829cb715a62f63982cecf3321bf8067d87afbfbcd9b4995ab4` |
| `run/suite-manifest.json` | `5bf5836bb8da5e28e21373bb3324236fd254a5d733ffe7b0d86f9a6fca46681b` |
| `run/results.jsonl` | `0b053083cd6e807188b17f7b9732e280ef3e120d5dc19d52e3ffd65114cee75f` |
| `run/nine-way-completion-100k/distann-multinode-summary.log` | `47180335a9ff7823dd4e87f29b3015b3b61ff46def5bda9b9558abcdc4c850fe` |
| `run/nine-way-completion-100k/physical-production-recall.log` | `31d739c1694a29fe229a507a8fcd5ddd899fba5697e2018610d63b01c7fe4ce6` |
| `run/nine-way-completion-100k/physical-production-latency.log` | `d71ad427fc35f253855007d259c377841a10c605df386396fa08149444d2e078` |

## Key results

- Recall `0.9625` (CI95 `0.9532--0.9700`); warm latency `27.70 ms`
  mean, `26.50 ms` p50, `34.10 ms` p95, `39.60 ms` p99.
- Physical generation storage `2,496,626,688` bytes; control indexes `24,576`
  bytes; same-data single index `854,810,624` bytes.
- Traversal `9.065098 ms/scan`: local expansion `1.478851`, remote expansion
  `7.429284`, coordinator partition `0.004965`, placement/decode `0.005831`,
  and frontier insertion `0.026753`.
- Remote decomposition `7.342625 ms/scan`: connection ready `0.040325`, request
  encode `0.004849`, owner service `2.258880`, transport wait `5.012911`, and
  coordinator receive/decode `0.025660`. Relative error is `1.1665%` against
  remote expansion, below the `5%` gate.
- Owner service includes remote graph read `1.200080`, scoring `0.894481`,
  open/validate `0.084687`, and response-row assembly `0.000504 ms/scan`.
  Straggler spread is `0.480184 ms/scan`.
- Traversal decomposition error is `1.3173%`, below the `10%` gate. All 34
  stage rows and 26 work rows are present.
- Per scan: 10 hop rounds, 40 nodes requested/returned, zero repeated nodes,
  14.3 query-cache hits / 2.0 misses, zero connections opened or statements
  prepared after warmup, 13,871.92 logical request bytes, and 10,530.32 logical
  response bytes.
- Decision: accept packet 007's paired fixed-work candidate STOP. The completed
  attribution still selects round-trip reduction, and that candidate already
  failed to produce a useful end-to-end/tail win.
