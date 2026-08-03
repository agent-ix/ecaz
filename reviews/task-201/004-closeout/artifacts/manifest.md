# Task 201 packet 004: closeout audit

- Head SHA: `c830b184fe4c750936ab13eab2891f63f06ba3d0`.
- Task bucket: `reviews/task-201/004-closeout/`.
- Scope: measurement-only closeout; no source or runtime-default changes.
- Required packet sequence is complete: 001 attribution, 002 isolated candidate, 003 release matrix, 004 closeout.
- PG18 release provenance is identical across the cited measurements: extension SHA `c830b184fe4c750936ab13eab2891f63f06ba3d0`, release profile, three physical nodes, shared physical-table surface.
- Memory diagnostic config: `artifacts/task201-normal-replica-memory-100k.json` (SHA256 `a6eecc8cda469440a69071e66b94e3b5b0160e9021dfd0edc0426d870403740d`).
- Memory diagnostic results: `artifacts/memory-run/results.jsonl` (SHA256 `fe619d871ade25df25d9e20880ee5552e386e2e74e24a4d610af3b8dafef40a1`); suite manifest (SHA256 `da44bdec5b35f7a7e1778a8e34b3b0133238d5853c3b0ded5b5fc1fcbb010f10`).
- Memory command: `/home/peter/dev/ecaz/.worktrees/task201/target/release/ecaz bench suite run --config reviews/task-201/004-closeout/artifacts/task201-normal-replica-memory-100k.json --artifact-dir reviews/task-201/004-closeout/artifacts/memory-run`.
- The memory run used the fixed normal-replica 100k physical generation, persisted trained 4096 head, BW4/H100, RaBitQ, lazy10, 300 measured queries, 10 warmups, and 250 ms backend RSS/HWM sampling for plan-cache-off control and MAT-40 plan-cache-on candidate. The suite step succeeded with exit code 0.

## Evidence map

| requirement | evidence |
| --- | --- |
| Fresh post-replica attribution with fallback and replica labeled | `../001-post-replica-attribution/request.md`, `../001-post-replica-attribution/artifacts/manifest.md`, structured results and attribution key lines |
| At most three preregistered candidates; at most one advanced | `../001-post-replica-attribution/request.md` preregisters MAT-40, MAT-21, MAT-26; `../002-isolated-latency-candidate/request.md` advances MAT-40 only |
| Same-generation isolated candidate A/B | `../002-isolated-latency-candidate/artifacts/run-fresh/results.jsonl` and manifest; recall 0.9625 in both arms, 3.0% mean improvement in the fresh 100k screen |
| Required 10k/50k/100k release evidence | `../003-release-matrix-and-decision/artifacts/run-v2/results.jsonl`, `suite-manifest.json`, and `release-key-lines.log`; all three suite steps succeeded |
| Recall, latency, storage, topology, and correctness | packet 003 structured results and compact scale summaries |

## Final disposition

MAT-40 is not promoted. It preserves recall and storage at all measured scales, but the 10k arm is 3.2% slower and the 50k/100k gains are only 1.2% mean latency. The candidate is therefore closed without a productionization task, ADR, default change, or new benchmark selector. The accepted Task 199 normal-replica path and its owner fallback remain unchanged.

The task is ready for outside closeout review. The coder does not mark the review request accepted; reviewer feedback belongs under this packet’s `feedback/` directory.
