# Task 198 packet 004 artifact manifest

- Branch head at measurement: `c2e158042a49406fa934655efba1fb34f5736577`
- Branch head including the final fixture correction: `f5b6653de`
- Installed extension SHA: `2ff72b3e49609c44cec881f72edf183a83554412`
- Task bucket: `reviews/task-198/004-isolated-100k/`
- Lane: Intel local, three independent PG18 owner processes; owner zero is
  also the sole authoritative coordinator
- Fixture: exact/disjoint hash ownership, one index per source table
- Storage and search: RaBitQ neighbor values, exact final score,
  `training_landmarks_exact`, cap 4,096, BW4/H100, owner lazy10 final payload,
  optional faithful coordinator traversal replica
- Corpus: staged `ec_real_100k`; evaluation queries 1--200; training queries
  201--400 from the same immutable query file
- Timestamp: 2026-07-23 America/Los_Angeles

## Commands and artifacts

```text
target/debug/ecaz bench suite audit --config reviews/task-198/004-isolated-100k/artifacts/task198-isolated-100k.json
target/debug/ecaz bench suite run --config reviews/task-198/004-isolated-100k/artifacts/task198-isolated-100k.json --artifact-dir reviews/task-198/004-isolated-100k/artifacts/run
```

The suite config, audit/run logs, `suite-manifest.json`, `results.jsonl`,
compact summary, raw recall/latency logs, and node logs are packet-local. The
suite completed successfully in 2,239,205 ms. All PostgreSQL nodes
unanimously attested the installed extension as a release-profile build at
the extension SHA above. The CLI was only the fixture driver.

## Paired result

- Ordered seed identity: identical digest
  `488caa73ad3f6c22864f9af309569ba4fe6edd72c8d535e71eec7bff78af6d50`.
- Recall: owner `0.9625`, replica `0.9625` over 200 queries / 2,000 top-10
  trials.
- Warm latency over 50 timed samples after 10 warmups:
  - owner: mean `20.60`, p50 `20.40`, p95 `23.80`, p99 `24.70`, max
    `24.90` ms;
  - replica: mean `17.10`, p50 `17.30`, p95 `20.00`, p99 `20.40`, max
    `20.60` ms.
- Replica deltas: mean `-17.0%`, p95 `-16.0%`, p99 `-17.4%`.
- Traversal reconciliation passed in both arms:
  - owner traversal `7.866018` ms, including `6.405357` ms remote expand,
    `4.272619` ms transport wait, and `0.421213` ms straggler spread;
  - replica traversal `3.616845` ms, including `3.386457` ms local
    graph/vector read, `0.140182` ms RaBitQ score, and zero remote expansion,
    transport wait, or straggler spread.
- Final owner payload cost stayed essentially fixed: owner payload SQL
  `8.976717` versus `9.000309` ms, and final materialization wait `7.098875`
  versus `7.329511` ms.
- Replica build/capacity: 100,000 records, 1,315,200,000 bytes copied,
  1,659,518,976 relation bytes, 1,925,866,616 WAL bytes, 51,995 ms build,
  and 3,366,912-byte peak copy batch. The source physical generation was
  2,496,659,456 bytes.
- Cache proxy after the workload: 2,669 heap blocks read and 112,311 hit.
- Owner-outage partial build rollback, digest identity, idempotent replay,
  corrupt-image fallback, durable `Ready -> Stale` invalidation, owner retry,
  and retire/reclaim all passed.

## Fixture finding

The fixed first query used by the mid-scan injection requested only 20 rows
and completed in one expander call, so the line reported `fallback_count=0`;
result identity still matched. An initial offset-selection correction in
`bb922fb9a` revealed that this was true across the first 32 queries and that
packet 003's apparent pass had read a stale counter from its preceding
correctness scenario. A first bounded 64-row correction then exposed the
planner selecting a sequential scan at 10k. Commits `cd47011d1` and
`446f4c581` therefore combine that bounded request with `enable_seqscan=off`
in both semantic arms. That forcing exposed an extra `id` tie-break in the
inner query that bypassed the production CustomScan shape; `f5b6653de`
removes the inner tie-break while retaining deterministic outer aggregation.
The dedicated drill must now enter the same CustomScan path and reach a
second expansion while checking owner-result identity on every attempted
query. Packet 005 reruns the corrected drill as part of the completion matrix.

## Decision

This is a useful candidate under Task 198's factual gate: recall and ordered
semantics are unchanged, while end-to-end mean and tails improve materially.
The added storage, WAL, and build cost are substantial and must remain
explicit in the final decision. The candidate therefore proceeds to the
matched 10k/50k completion suite; this packet supplies the matched 100k row.
