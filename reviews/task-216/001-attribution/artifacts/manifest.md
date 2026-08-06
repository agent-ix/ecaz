# Task 216 attribution manifest

- Task bucket: `reviews/task-216/001-attribution/`
- Packet status: pre-registration; no candidate implementation
- Control: normal PG18 release, sharded owners, no coordinator full-graph
  replica, current materialization/schema-cache behavior
- Required first scale: fresh physical 100k generation
- Runner: `ecaz bench suite` with a checked-in SuiteConfig
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
