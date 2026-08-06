# Task 215 review request: release A/B matrix and decision

Decision: STOP. BW64/H8 is not promoted; shipped defaults remain BW4/H100.

The complete normal PG18 release matrix passed all six arm harnesses, but the
candidate changed recall and was slower at every staged scale. See
`artifacts/decision.md` for the paired recall, latency-tail, storage, and
rollback evidence.

Please review `artifacts/task215-release-ab.json`, the generated suite
manifest/results, and the packet-local decision artifacts. The cited decision
run is under `artifacts/run-r2/`; the earlier stale-schema `artifacts/run/`
attempt is superseded and is not decision evidence.
