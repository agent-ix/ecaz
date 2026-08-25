# Task 223 packet 002 artifact manifest

Date: 2026-08-25 (America/Los_Angeles)

- Task bucket / packet: `reviews/task-223/002-owner-substage-attribution/`
- Campaign base SHA: `ed5ac814c05350ca695533fcd54d0df11faa876b`
- Production behavior measured: Task 222 extension SHA
  `c9f79be4a756031b3f8301960fc0f57b77ae60d1`
- Lane: local Intel host, three-owner physical PG18 release extension,
  `ec_real_100k`, 200 held-out queries, top-k 10, 50 timed warm iterations plus
  10 warmups, persisted head 4096, BW4/H100, production lazy-10
- Isolation: one fresh physical generation shared by the Task 222 all-column
  control and projected candidate; Task 223 reuses the accepted projected
  production result and creates no new runtime arm
- Decision source: `artifacts/gate-calculation.md`

## Source suite command

```text
ecaz bench suite run \
  --config reviews/task-222/004-full-scale-decision/artifacts/task222-payload-projection-matrix.json \
  --only payload-projection-ab-100k \
  --manifest-output reviews/task-222/004-full-scale-decision/artifacts/run/suite-100k-manifest.json \
  --results-output reviews/task-222/004-full-scale-decision/artifacts/run/results-100k.jsonl \
  --log-file reviews/task-222/004-full-scale-decision/artifacts/suite-100k-final.log
```

The suite manifest reports the selected step succeeded in 1,320,377 ms, with
the other two scales intentionally skipped and zero failed/missing/stale
steps. Task 222's artifact manifest records full provenance and cleanup.

## Durable source identities

- Task 222 suite config: SHA-256
  `00cdc3738db20530c56ff4160b541ece98abe4d883945d69a2528bb20e17f178`.
- 100k suite manifest: SHA-256
  `57776404b2874b0d053b0acab7a6ce4227afcbdbdfd09b247e87772e2f443ffe`.
- 100k normalized results: SHA-256
  `b249ce4c1f56beae1dffbb2f6edcc18e8ec48076bed0f39a4083780c8242ab90`.
- 100k compact summary: SHA-256
  `69ccdb50264bbe383f4026bf0b484c795fab7b07e39a2da4daf8427dad3172c4`.
- 100k projected latency log: SHA-256
  `73664b26d82c02ea1a7ac2115b89423936c6bc15b4996d9273ab59dd53834b34`.
- 100k projected predictions: SHA-256
  `228e17fbe4fa7480dced302f5b650721e6833d271503ad90b5d35b99d663eb0d`.

No corpus, truth cache, PGDATA, polling state, or new benchmark output is
duplicated into this packet. The immutable Task 222 packet is the
machine-readable source of truth; `gate-calculation.md` copies only the cited
result lines and arithmetic needed for Task 223's decision.

