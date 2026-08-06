# Task 216 packet 002 manifest

- Packet: `reviews/task-216/002-isolated-candidate/`
- Status: pre-registered; measurement not yet run
- Candidate: MAT-15 only — packed null bitmap + cumulative `int8` offsets + flat `bytea`
- Control: current per-column `bytea[]` payload representation
- Required first scale: fresh physical 100k A/B
- Runner: `ecaz bench suite` with `artifacts/task216-mat15-100k.json`
- Topology: normal PG18 release, coordinator plus three sharded owners; one
  index per owner table, no coordinator full-graph replica
- Parameters: BW4/H100/L32, graph degree 32, build shards 1, head cap 4096,
  persisted-head seeds 32, hop rounds 100, top-k 10, 50 iterations / 10
  warmups, `ec_real_100k`
- Isolation: control is run from the pre-change release commit; candidate is
  run from the MAT-15 commit. No MAT-21, Task 215 BW64/H8 defaults, or other
  traversal/materialization changes are included.
- Artifact rule: suite manifests, `results.jsonl`, cited summaries, and
  validation logs remain under this packet's `artifacts/`; cluster run
  directories remain outside the repository and are removed after capture.

The final version of this manifest will add the two head SHAs, exact suite
commands, timestamps, artifact hashes, and cited result lines after both runs.

