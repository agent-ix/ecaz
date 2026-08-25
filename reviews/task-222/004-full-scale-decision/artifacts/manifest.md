# Task 222 packet 004 artifact manifest

- Extension/source behavior SHA: `c9f79be4a756031b3f8301960fc0f57b77ae60d1`
- CLI reuse-attestation correction: `f1351d2db`
- Reviewer cleanup SHA: `06b59c4c6bf818236b852b2ac6597fbbe92593a2`
- Task bucket / packet: `reviews/task-222/004-full-scale-decision/`
- Original matrix timestamp: `2026-08-23T23:03:55-07:00`; explicit 10k/50k
  normalized-evidence reruns completed later on `2026-08-23`
- Lane: three-owner physical PG18 release extension, `ec_real_10k`,
  `ec_real_50k`, and `ec_real_100k`; 200 held-out queries/scale, top-k 10,
  50 warm timed iterations plus 10 warmups
- Storage / rerank: RaBitQ physical generation, persisted head 4096,
  head-search/head-seed 32/32, beam width 4, hop limit 100, production lazy-10
- Isolation: one fresh physical generation per scale, shared by its control and
  candidate; only `payload_projection` differs
- NFR registrations: `task222-all-columns-control` and
  `task222-projected-candidate`, both preregistered conforming
- Run directories: `/home/peter/.ecaz/clusters/task222-payload-projection-10k`,
  `/home/peter/.ecaz/clusters/task222-payload-projection-50k`, and
  `/home/peter/.ecaz/clusters/task222-payload-projection-matrix-100k`; all are
  external to the repository and removed after capture
- Decision: useful at every scale; see `decision.md`

Runner provenance is `351feb6c4` for 10k, `351feb6c4-dirty` for 50k, and
`82b475be7-dirty` for 100k. The 50k dirty state is solely the preceding
packet-local 10k rerun evidence; the 100k dirty state included packet/config
bookkeeping and the then-uncommitted reuse-attestation correction subsequently
committed unchanged as `f1351d2db`. The benchmarked extension itself is
independently attested clean, release-profile, and unanimous at `c9f79be4a` on
all three nodes at every scale. Runner bookkeeping and fixture validation do
not change extension scan behavior.

## Commands and split execution

The matrix was driven only by `ecaz bench suite` with
`task222-payload-projection-matrix.json`.

Each published row is from a successful fresh `--only` invocation. The 10k and
50k commands differ only in the selected step and output names:

`ecaz bench suite run --config reviews/task-222/004-full-scale-decision/artifacts/task222-payload-projection-matrix.json --only payload-projection-ab-10k --manifest-output reviews/task-222/004-full-scale-decision/artifacts/run/suite-10k-manifest.json --results-output reviews/task-222/004-full-scale-decision/artifacts/run/results-10k.jsonl --log-file reviews/task-222/004-full-scale-decision/artifacts/suite-10k-rerun.log`

`ecaz bench suite run --config reviews/task-222/004-full-scale-decision/artifacts/task222-payload-projection-matrix.json --only payload-projection-ab-50k --manifest-output reviews/task-222/004-full-scale-decision/artifacts/run/suite-50k-manifest.json --results-output reviews/task-222/004-full-scale-decision/artifacts/run/results-50k.jsonl --log-file reviews/task-222/004-full-scale-decision/artifacts/suite-50k-rerun.log`

The 100k command is:

`ecaz bench suite run --config reviews/task-222/004-full-scale-decision/artifacts/task222-payload-projection-matrix.json --only payload-projection-ab-100k --manifest-output reviews/task-222/004-full-scale-decision/artifacts/run/suite-100k-manifest.json --results-output reviews/task-222/004-full-scale-decision/artifacts/run/results-100k.jsonl --log-file reviews/task-222/004-full-scale-decision/artifacts/suite-100k-final.log`

All three manifests report one succeeded, two intentionally skipped, and zero
failed/missing/stale steps. Durations are 141,837 ms, 633,487 ms, and
1,320,377 ms for 10k, 50k, and 100k respectively. Their normalized results
contain 1,463, 1,985, and 1,803 rows.

## Durable artifacts

- `task222-payload-projection-matrix.json`: final SuiteConfig.
- `suite-10k-rerun.log`, `suite-50k-rerun.log`,
  `run/suite-{10k,50k}-manifest.json`, and
  `run/results-{10k,50k}.jsonl`: successful fresh 10k/50k suite evidence.
- `suite-100k-final.log`, `suite-100k-status.log`,
  `run/suite-100k-manifest.json`, and `run/results-100k.jsonl`: successful
  fresh 100k suite evidence.
- `run/{10k,50k,100k}/distann-multinode-summary.log`: compact provenance,
  topology, same-generation, recall, latency, payload/stage, storage, and gate
  lines for every scale.
- `run/{10k,50k,100k}/physical-{all-columns-control,projected-candidate}-{recall,latency}.log`:
  direct metrics cited in `decision.md`.
- `run/{10k,50k,100k}/physical-{all-columns-control,projected-candidate}-predictions.json`:
  byte-identical ordered result evidence.
- `run/{10k,50k,100k}/physical-head-membership.json`: deterministic head
  membership evidence.
- `completion-audit.md`: acceptance-criterion and reviewer-finding closure map.
- `cargo-check-copyobject.log`: PG18 library check at reviewer cleanup SHA;
  finished successfully.
- `pg18-copyobject-contract.log`: focused three-owner payload-projection
  contract at reviewer cleanup SHA; 1 passed, 0 failed, 2,578 filtered out in
  78.10 seconds.

No corpus TSV, truth cache, PGDATA, PostgreSQL operational log, polling
snapshot, or failed reuse exhaust is committed.
