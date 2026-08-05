# 100k setup attempt

- head SHA: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`
- command: `target/debug/ecaz bench suite run --config reviews/task-206/002-100k-sweep/artifacts/task206-100k-sweep.json`
- fixture: `/home/peter/.ecaz/clusters/task206-100k-wide-beam-sweep`
- extension preflight: passed; 3 nodes unanimous; release profile; extension
  SHA `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`
- observed log: `physical_setup_start rows=2000 nodes=3`
- observed state: node1 grew to approximately 4.4 GB and created 1 GiB plus
  779 MiB PostgreSQL temporary files; nodes 2 and 3 remained at approximately
  42 MB. The runner emitted no result rows or benchmark metrics.
- disposition: stopped after the temporary files and directory size stopped
  changing during an additional observation window; the exact external run
  directory was removed afterward.
- result: setup incomplete; this is not recall, latency, or storage evidence.

The per-node PostgreSQL logs are retained beside this file. They show normal
startup and shutdown, with no extension error; node1 reported frequent WAL
checkpoints while the setup operation was active.
