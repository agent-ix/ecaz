# Task 226 packet 002 artifact manifest

- Snapshot-lifetime code correction SHA:
  `c85196ce841c1cbcea187dbefb3c10430fb611be`
- Suite runner binary SHA: `b54f321a579ccdac1535aedc4e3387f78811b0af`
  (the post-preregistration corrections touch extension/runtime code, not the
  suite command expansion)
- Clean release extension execution-head SHA:
  `a1f1584966011ca7c16175fe91f8efc302c8cf25`, unanimous on all three owners.
- Task bucket / packet: `reviews/task-226/002-current-head-100k/`
- Lane: three-owner physical PG18 release extension, fixed 4,096 persisted
  sharded head, `ec_real_100k`, 200 held-out queries, top-k 10, L32, H100,
  RaBitQ scoring, and production lazy-10 materialization
- Isolation: fresh production and attribution fixtures outside the repository;
  every fixture shares one immutable generation across its runtime variants
- SuiteConfig: `task226-current-head-bw8-100k.json`, SHA-256
  `faec8932e937ea12c85e201fa6a3601dc561572cbc687887b72ec0194a5f11f3`
- Production A/A: `aa-control` / `aa-candidate`, both BW4/H100
- Production A/B: `bw4-control` / `bw8-candidate`; only beam width changes
- Attribution A/B: the same BW4/BW8 delta on a separate full-metrics fixture;
  its instrumented latency is diagnostic only
- Run directories: `/home/peter/.ecaz/clusters/task226-bw8-production-100k`
  and `/home/peter/.ecaz/clusters/task226-bw8-attribution-100k`; both were
  stopped and removed after their cited evidence was captured
- Storage format / rerank: unchanged physical RaBitQ generation; no format or
  rerank-mode change

## Preregistration evidence

- `suite-audit.log`: config audit passes with two steps.
- `suite-dry-run.log` and `suite-dry-run-manifest.json`: expanded commands from
  the clean pre-Task-222 runner. Variant strings end at the existing
  traversal/locator fields and contain no payload-projection axis.
- `request.md` and the Task 226 file at `d42d01e32`: numerical
  ADVANCE/TRADE/STOP rule recorded before measurement.
- `pre-guard-failure.md`: decisive pre-measurement baseline failure and the
  already-landed production guard cherry-picked as `c51e74c5e`; no fixture
  workaround or measurement claim.
- `pre-snapshot-guard-failure.md`: published-topology, pre-measurement
  PostgreSQL assertion diagnosis and the existing snapshot-ownership fix
  cherry-picked as `c85196ce8`; no arm measurement or gate claim.

## Successful execution artifacts

Production command:

```text
/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-226/002-current-head-100k/artifacts/task226-current-head-bw8-100k.json --only current-head-bw8-production-100k --manifest-output reviews/task-226/002-current-head-100k/artifacts/run/production-suite-manifest.json --results-output reviews/task-226/002-current-head-100k/artifacts/run/production-results.jsonl --log-file reviews/task-226/002-current-head-100k/artifacts/production-suite.log
```

Attribution command:

```text
/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-226/002-current-head-100k/artifacts/task226-current-head-bw8-100k.json --only current-head-bw8-attribution-100k --manifest-output reviews/task-226/002-current-head-100k/artifacts/run/attribution-suite-manifest.json --results-output reviews/task-226/002-current-head-100k/artifacts/run/attribution-results.jsonl --log-file reviews/task-226/002-current-head-100k/artifacts/attribution-suite.log
```

- `run/production-suite-manifest.json` and `run/production-results.jsonl`:
  immutable expanded production command and normalized result rows. Generated
  at Unix ms `1787559874848`; status is one succeeded selected step and one
  intentionally skipped step.
- `production-suite.log`: suite-level command/status log.
- `run/production-100k/distann-multinode-summary.log`: direct production
  topology, generation, recall, latency, storage, and conformance rows.
- `run/production-100k/physical-*-{recall,latency}.log` and
  `physical-*-predictions.json`: direct per-arm rows and per-query predictions.
  The two A/A prediction files are byte-identical at SHA-256
  `84f3ee959c59b8541cb7347cb5b9525624d4bab9b77b440c6d3dabb24a6308db`.
- `run/production-100k/physical-head-membership.json`: persisted-head
  membership evidence for the shared generation.
- `run/attribution-suite-manifest.json` and `run/attribution-results.jsonl`:
  immutable expanded full-metrics command and normalized attribution rows.
  Generated at Unix ms `1787561473525`; status is one intentionally skipped
  step and one succeeded selected step.
- `attribution-suite.log` and
  `run/attribution-100k/distann-multinode-summary.log`: suite/direct
  full-metrics evidence. Instrumented latency is diagnostic only.
- `decision-summary.md`: compact source-indexed arithmetic and disposition.

## Key result lines

- Production A/A: byte-identical predictions, recall 0.9285 / 0.9285.
- Production A/B: BW4 recall 0.9285, mean 16.40 ms, p95 19.00 ms; BW8
  recall 0.9450, mean 16.20 ms, p95 19.80 ms.
- Paired production recall: +0.016500, 95% CI
  `[+0.008000, +0.026500]`, 20 candidate wins / 2 control wins / 178 ties.
- Storage: 2,498,281,472 physical generation bytes in both arms;
  831,782,912 graph-side bytes, 1,666,498,560 owner row-tier bytes, and zero
  coordinator-resident unsharded bytes.
- Published topology: owner rows 33,195 / 33,432 / 33,373, zero non-owned,
  zero orphans.
- Attribution reproduction: BW4/BW8 recall 0.9275/0.9460; paired delta
  +0.018500 with 95% CI `[+0.009500, +0.029000]`.
- Attribution transport wait: BW4/BW8 3.259058/2.795992 ms mean; total scan
  14.917298/15.549734 ms mean.

Disposition: `ADVANCE` to the preregistered fresh 10k/50k confirmation matrix.
This packet does not authorize a default-policy change and remains review-open.
