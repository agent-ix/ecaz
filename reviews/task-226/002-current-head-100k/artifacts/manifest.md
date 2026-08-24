# Task 226 packet 002 artifact manifest

- Source / runner SHA: `97fee7ced35bde4d7f6f768520ab9074b9ad37b6`
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
  and `/home/peter/.ecaz/clusters/task226-bw8-attribution-100k`; both will be
  removed immediately after their cited evidence is captured
- Storage format / rerank: unchanged physical RaBitQ generation; no format or
  rerank-mode change

## Preregistration evidence

- `suite-audit.log`: config audit passes with two steps.
- `suite-dry-run.log` and `suite-dry-run-manifest.json`: expanded commands from
  the clean pre-Task-222 runner. Variant strings end at the existing
  traversal/locator fields and contain no payload-projection axis.
- `request.md` and the Task 226 file at `d42d01e32`: numerical
  ADVANCE/TRADE/STOP rule recorded before measurement.

No benchmark result is claimed yet. Successful suite manifests, normalized
results, direct logs, compact summaries, decision lines, commands, timestamps,
and fixture cleanup will be recorded after execution.
