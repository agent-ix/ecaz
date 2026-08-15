# Task 167 packet 026 artifacts

- Packet: `reviews/task-167/026-owner-retry`.
- Production code checkpoint: `cf5ad6761` (parent implementation checkpoint
  `5f56390d1`); the cited matrix extension SHA is:
  `5f56390d18c09a7c3020951d0e0318fab7d9eede`.
- Fixture/harness checkpoints: `4ad02848c`, `234e6925d`, `e0d58ab8e`,
  `6afbe9257`, `8f6dd3621`, and `f75ea993f`. These only provide owner timeout
  propagation, bounded large-scale controls, and packet runner options; the
  release extension provenance is the production SHA above.
- Suite config: `artifacts/task167-owner-retry-suite.json`; runner:
  `ecaz bench suite`.
- Timestamp: 2026-08-14; PG18; three-node physical generation; graph degree 5.
  All cited runs report `release_profile_preflight status=passed`, unanimous
  release profile, and the exact extension SHA.
- Run directories are outside the repository under `/home/peter/.ecaz/clusters/`.
- Compact cited lines are in `artifacts/cited-results-final.log`. The raw
  release logs cited there are the 10k, 50k, and 100k fixture logs in the
  corresponding `bench-suite-final-*` directories.

Reviewer finding coverage:

- `resolve_nodes` now owns the bounded, exact intent-gated retry. The gate is
  scoped to owner, epoch, generation node, vec_id, fresh intent state, and
  freshness; it is fail-closed when the exact current tuple is absent. There
  is no `pg_sleep` inside the locked owner scan.
- Owner-side attribution is sampled/reset on each owner backend. The exact 10k
  churn arm proves `churn_retries=3`; the steady arm proves `steady_retries=0`
  on every owner. The corrected reverse-edge label and shared-target backlink
  gate report `reverse_edge_coverage=15/24` and `back_edge_check=true`.
- Append-when-room is isolated by an A/B arm. Results are 1.062237x, 1.043178x,
  and 1.026743x enabled/disabled at 10k/50k/100k respectively, with amendment
  counts retained in the cited log.
- Recall, latency, storage, insert throughput, and inserted-neighborhood
  fresh-rebuild checks are present at 10k/50k/100k. Large-scale latency is
  intentionally one warmup plus one measured query; exact counts and
  provenance are retained in the raw logs.
- The pinned ANN probe is diagnostic only because ANN selection precedes the
  post-filter; the owner-local exact probe is authoritative. The 100k node-3
  pinned sample's stable-prefix diagnostic is retained verbatim and is not
  misreported as a passing pinned query.

Open disposition:

- The 50k and 100k benchmark metrics pass, but their separate shared-target
  concurrency drills stalled and were terminated. They are not claimed as
  passes; the exact 10k owner-side concurrency gate is the authoritative proof
  that the `resolve_nodes` retry executes on the owner. A fresh large-scale
  concurrency arm is not required to establish that path and is not used as
  closeout evidence.
