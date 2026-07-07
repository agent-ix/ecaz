# Task 146 Packet 006: Matched Release Anchor Results

Please review the matched release anchor results for Task 146.

This packet runs the packet 004 anchor suite on local Intel PG18 against a
release `ecaz.so`. The suite completed 24/24 steps with recall, latency, and
storage for IVF and HNSW controls at 10k, 50k, and 100k.

Key provenance:

- Head SHA: `f18aac406176e31dc2b384d50637d6fe1118ba4e`
- Suite config: `artifacts/suite-task146-release-anchors.json`
- Suite manifest: `artifacts/suite-manifest.json`
- Raw results: `artifacts/results.jsonl`
- Compact summaries:
  - `artifacts/anchor-recall-latency.txt`
  - `artifacts/anchor-storage-index.txt`
- Successful run log: `artifacts/suite-run-r4.log`
- Status: `completed=24 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`
- Backend profile: `coordinator:28818:release`
- Library SHA256:
  `b261b873f3db494f7c56a3894cda5b4344f078447c1ccce6bff7530fb013d27a`

Highlighted anchor rows:

- 50k IVF nlists=64 reaches 0.9975 at nprobe=48 with p50 33.6 ms, and
  1.0000 at nprobe=64 with p50 37.2 ms.
- 100k IVF nlists=128 reaches 0.9980 at nprobe=96 with p50 37.6 ms, and
  1.0000 at nprobe=128 with p50 42.2 ms.
- HNSW control remains below the Task 146 recall threshold at these sweeps:
  100k ef_search=400 gives 0.9795 recall@10 with p50 20.4 ms.

Scope boundary:

- This is anchor-only evidence, not SPIRE S1-S6 matrix evidence.
- This packet does not make a Task 146 promote/do-not-promote decision.
- Task 145 packet 008 bound-prune remains null/faulty evidence because its
  mechanism did not engage; it is not used here as support for any conclusion.
