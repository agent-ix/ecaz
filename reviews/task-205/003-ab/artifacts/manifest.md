# Task 205 A/B packet

- Candidate head SHA: `615fd72b2d6d31d7bec9020eabcfa8fa34d39a68`.
- Baseline source for the no-pushdown comparison: parent of the bundled
  implementation, `350736f62`.
- Task bucket/packet: `reviews/task-205/003-ab/`.
- Suite config: `artifacts/task205-ab-suite.json`.
- Lane/fixture: PG18 `distann-local-multinode`, three nodes, rabitq neighbor
  codes, default exact co-located rerank; 10k/50k/100k.
- Fixed parameters: BW=4, H=100, top-k=10, head search width=32, head seed
  count=32, 50 benchmark iterations after 10 warmups.
- Control arm: `owner-traversal-control`, `traversal_replica=false`; it is the
  distributed owner path and is the only decision-bearing control.
- Candidate arm: `algorithm1-pushdown`, same distributed owner path on the
  candidate build; the A/B attribution axis is the baseline versus candidate
  build, not a replica arm.
- Required metrics: recall, end-to-end latency, storage, request/response
  bytes, and per-round transport wait.
- Run directories: `/home/peter/.ecaz/clusters/task205-{control-owner,candidate}-{10k,50k,100k}`;
  all are outside the repository and `target/`.
- Timestamp: 2026-07-29 America/Los_Angeles.

## NFR-021 pre-registration verdict

The owner-traversal control and the candidate are admissible in architecture:
both use physically sharded owner state and no coordinator-resident traversal
replica. The verdict remains **pending measurement evidence** until the suite
emits per-node rows and the 100k/10k growth ratio. A replica arm is excluded
from the decision-bearing A/B.

The matrix is not run in this packet because the staged corpus inputs are absent
on this host. No recall, latency, storage, byte, or transport number is claimed.
