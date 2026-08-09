# Task 219 packet 001 artifact manifest

- Task bucket: `reviews/task-219/`
- Packet: `001-frontier-assembly/`
- Source head: `ea51a9c8bdce1f412652ac743ae0d055af8daa76`
- Source packet: `reviews/task-215/003-release-matrix-and-decision/`
- Source artifact: `artifacts/run-r2/results.jsonl`
- Source config: `artifacts/task215-release-ab.json`
- Lane: normal PG18 release A/B, three sharded owners, no coordinator replica
- Scales: 10k / 50k / 100k
- Storage format: BW4/H100/L32 control versus BW64/H8/L64-effective candidate
- Rerank mode: normal RaBitQ neighbor scoring; no candidate mechanism added
- Surface: shared three-owner release fixture; not isolated one-index-per-table
- Command: `ecaz bench suite run --config reviews/task-215/003-release-matrix-and-decision/artifacts/task215-release-ab.json --artifact-dir reviews/task-215/003-release-matrix-and-decision/artifacts/run-r2 --manifest-output reviews/task-215/003-release-matrix-and-decision/artifacts/run-r2/suite-manifest.json --results-output reviews/task-215/003-release-matrix-and-decision/artifacts/run-r2/results.jsonl --continue-on-error`
- Captured: `2026-08-06T17:16:49Z` manifest/report audit
- Result: six release arms succeeded; candidate was slower and not recall-equivalent

The frontier table in `frontier.md` is transcribed from the cited
packet-local `results.jsonl`; the source packet's accepted reviewer feedback
independently verifies the paired rows.
