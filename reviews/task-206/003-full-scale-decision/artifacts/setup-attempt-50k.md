# 50k setup attempt

- code head: `59aeb6c58fa3e2f0db1774a6c3c8a5ab62308e78`
- command: `target/debug/ecaz bench suite run --config reviews/task-206/003-full-scale-decision/artifacts/task206-50k-diagnostic.json`
- fixture: `/home/peter/.ecaz/clusters/task206-50k-bw32-h8`
- extension preflight: passed; 3 nodes unanimous; release profile
- observed log: `physical_setup_start rows=2000 nodes=3`
- observed state: node1 grew to approximately 2.4 GB; nodes 2 and 3 remained
  initialized at approximately 42 MB; one 925 MB PostgreSQL temporary file
  stopped changing; no topology, recall, latency, or storage rows were
  emitted.
- disposition: stopped after the final observation window and removed the
  exact external run directory.
- result: setup incomplete; not benchmark evidence.
