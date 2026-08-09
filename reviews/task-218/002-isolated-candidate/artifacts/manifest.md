# Task 218 packet 002 artifact manifest

Isolated 100k MAT-21 run completed with STOP disposition; outside review is
still pending.

- Task bucket: `reviews/task-218/002-isolated-candidate/`
- Config: `artifacts/task218-mat21-100k.json`
- Candidate: typed/binary `tid[]` locator binding versus textual `ctid`
- Required runner: `ecaz bench suite`
- Required scale: 100k, real corpus `ec_real_100k`
- Required surfaces: recall, warm latency, storage, stage counters, same-generation A/A
- Head SHA: `877cebdbe1694260b15de1d0f6182c7ceb17cb0e` (corrected typed-locator
  SQL); extension preflight records the packet-created `-dirty` suffix.
- Command: `ecaz bench suite run --config artifacts/task218-mat21-100k.json
  --results-output artifacts/run/results.jsonl --manifest-output
  artifacts/run/suite-manifest.json`
- Timestamp: 2026-08-08; completed=1, failed=0, skipped=0,
  missing_artifacts=0, stale=0.
- Generation identity: both arms `02007a9e813ebb35aa679d31e9e3ba754b97f7c5a1898b21941e7e9ec60b48ec5da8`.
- A/A: `physical_benchmark_same_generation_recall byte_identical=true`.
- Recall: control/candidate `0.9280/0.9280`; warm latency `19.60/19.50`
  ms; custom scan total `17.578195/17.423344` ms/scan.
- Owner payload SQL: `8.555023/8.455005` ms/scan; executor remote rows
  `6.660000/6.660000` per scan; storage `2,496,626,688` bytes and
  amplification `1.351147` for both arms.
- Evidence: `artifacts/run/100k/mat21-evidence.log`; structured source:
  `artifacts/run/results.jsonl`.
