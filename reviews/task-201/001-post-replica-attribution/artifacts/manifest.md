# Task 201 packet 001: post-replica attribution

- Head SHA: `c830b184fe4c750936ab13eab2891f63f06ba3d0`
- Task bucket: `reviews/task-201/001-post-replica-attribution/`
- Run artifact root: `artifacts/run/`
- Suite config: `artifacts/task201-attribution-100k.json`
- Suite config SHA-256: `71850b1f9f7231981347638859014d039bb358f7fb7706f70d88c7979868a663`
- Results SHA-256: `5a28fe83c67839d93833e36b4fc93c01730c107d89b0b7d4c7db8ea6896dfcf7`
- Suite manifest SHA-256: `0eb03bbd0bad3a32e4a2a4cfe2fbcb75d6e20f9c5d2ef6612124dd4e3de51ba6`
- Extension: isolated PG18 release install; source SHA `c830b184fe4c750936ab13eab2891f63f06ba3d0`; installed `.so` SHA-256 `fb3c49c481338ddd00eade39244b5a7507194f1f78b24f8e4493568bb15f2b80`
- CLI: `/home/peter/dev/ecaz/.worktrees/task201/target/release/ecaz`; SHA-256 `8496663553699d4350755b771d98655d6a9adea75e112c50c5ac5524797497e3`
- Corpus: `ec_real_100k`; staged query SHA-256 `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782`
- Timestamp: `2026-08-03 10:42:16 -0700` (results artifact mtime)
- Command: `CARGO_TARGET_DIR=/home/peter/dev/ecaz/.worktrees/task201/target /home/peter/dev/ecaz/.worktrees/task201/target/release/ecaz bench suite run --config reviews/task-201/001-post-replica-attribution/artifacts/task201-attribution-100k.json --artifact-dir reviews/task-201/001-post-replica-attribution/artifacts/run`
- Fixture: fresh physical 3-owner PG18 fixture under `/home/peter/.ecaz/clusters/task201-attribution-100k`; shared-table physical generation, not isolated one-index-per-table surfaces. The single-index control was setup-only and is not used for the Task 201 A/B decision.
- Storage format: BW4/H100, trained 4096-landmark exact head, graph degree 32, RaBitQ neighbors, exact ranking, lazy10 payload materialization, owner payload plan cache off, 200 queries, 50 warm latency samples, 10 warmups.
- Variants: `owner-fallback-control` (`traversal_replica=false`) and `normal-replica` (`traversal_replica=true`) use the same persisted-head seed digest and all other settings.

The durable key result lines are in `artifacts/attribution-key-lines.log`. The structured source of truth is `artifacts/run/results.jsonl`; the packet-local suite manifest is `artifacts/run/suite-manifest.json`.

## Attribution finding

The normal replica reduced mean latency from 19.70 ms to 17.40 ms (11.7%) with identical 0.9625 recall, identical remote rows (332 over 50 scans), identical payload bytes (8,350,092), and identical physical-generation storage (3,188,056,064 bytes). It replaced the 7.543 ms owner traversal total with a 3.615 ms local replica traversal total; the remaining largest measured residual is owner payload SQL work at 8.922 ms and remote materialization at 7.474 ms. Replica fallback count was zero in the measured arm.

## Candidate screen preregistration

The measured trigger is owner payload/executor residual, not head or graph correctness. The screen was limited to three eligible ledger candidates:

1. **MAT-40** projection-shape payload cache/prepared portal — primary candidate, directly triggered by repeated owner payload SQL work.
2. **MAT-21** typed/binary locators — secondary, triggered only if locator/transport work dominates after MAT-40.
3. **MAT-26** batch detoast/binary-send by physical block — secondary, triggered only if heap/varlena work dominates after MAT-40.

Only MAT-40 is advanced in packet 002. No head, graph, neighbor-codec, or replica-lifecycle change is included.

RSS/allocation follow-up for the fixed normal-replica path is captured in the
closeout packet’s packet-local memory suite:
`../004-closeout/artifacts/memory-key-lines.log` and
`../004-closeout/artifacts/memory-run/`.
