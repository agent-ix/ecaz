# Task 216 attribution manifest

- Task bucket: `reviews/task-216/001-attribution/`
- Packet status: pre-registration; no candidate implementation
- Control: normal PG18 release, sharded owners, no coordinator full-graph
  replica, current materialization/schema-cache behavior
- Required first scale: fresh physical 100k generation
- Runner: `ecaz bench suite` with a checked-in SuiteConfig
- Suite config: `artifacts/task216-100k-attribution.json`
- Diagnostic extension: PG18 release with
  `distann-head-attribution-benchmark`; suite uses `skip_install=true`
- Control parameters: BW4/H100, effective L32, persisted-head seeds 32,
  graph degree 32, build shards 1, top-k 10, 50 iterations / 10 warmups
- Candidate rule: pre-register at most three measured-stage candidates and
  advance at most one
- Hard separation: no Task 215 BW64/H8 defaults in the decision control or
  candidate
- Evidence to capture: owner graph read, scoring, response assembly/encoding,
  wire wait, coordinator decode/copy, executor residual, allocations/copies,
  request/response bytes, recall/result identity, tails, topology, and failure
  semantics where measurable

No benchmark artifact is cited yet. The suite config and subsequent logs will
be added under this packet before measurement begins.

Pre-registered candidate families before measurement: `MAT-15`, `MAT-21`, and
`TRAV-05`. The run must select candidates from measured dominant stages; this
list is not an implementation commitment.
