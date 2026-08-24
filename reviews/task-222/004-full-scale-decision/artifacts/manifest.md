# Task 222 packet 004 artifact manifest

- Extension/source behavior SHA: `c9f79be4a756031b3f8301960fc0f57b77ae60d1`
- CLI reuse-attestation correction: `f1351d2db`
- Task bucket / packet: `reviews/task-222/004-full-scale-decision/`
- Timestamp: `2026-08-23T23:03:55-07:00`
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

The final suite manifest's runner provenance is
`82b475be7d357523dc681244c859342852f6474a-dirty`. The benchmarked extension
itself is independently attested clean and unanimous at `c9f79be4a` on all
three nodes. The runner's dirty state included the packet/config bookkeeping
and the then-uncommitted reuse-attestation source correction subsequently
committed unchanged as `f1351d2db`; that correction changes fixture validation,
not extension scan behavior.

## Commands and split execution

The matrix was driven only by `ecaz bench suite` with
`task222-payload-projection-matrix.json`.

The initial invocation completed the 10k and 50k steps, then rejected a
proposed reuse of packet 003's already-drilled 100k fixture before measurement:

`ecaz bench suite run --config reviews/task-222/004-full-scale-decision/artifacts/task222-payload-projection-matrix.json --log-file reviews/task-222/004-full-scale-decision/artifacts/suite-final.log`

The pending 100k row was then rebuilt fresh and run alone:

`ecaz bench suite run --config reviews/task-222/004-full-scale-decision/artifacts/task222-payload-projection-matrix.json --only payload-projection-ab-100k --manifest-output reviews/task-222/004-full-scale-decision/artifacts/run/suite-100k-manifest.json --results-output reviews/task-222/004-full-scale-decision/artifacts/run/results-100k.jsonl --log-file reviews/task-222/004-full-scale-decision/artifacts/suite-100k-final.log`

The final 100k manifest reports one succeeded, two intentionally skipped,
zero failed/missing/stale steps and duration 1,320,377 ms. The first wrapper
log's transition from 10k to 50k to 100k, plus both scales' complete compact
summaries, records successful 10k/50k completion.

## Durable artifacts

- `task222-payload-projection-matrix.json`: final SuiteConfig.
- `suite-final.log`: initial suite orchestration for completed 10k/50k and the
  rejected pre-measurement 100k reuse attempt.
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

No corpus TSV, truth cache, PGDATA, PostgreSQL operational log, polling
snapshot, or failed reuse exhaust is committed.
