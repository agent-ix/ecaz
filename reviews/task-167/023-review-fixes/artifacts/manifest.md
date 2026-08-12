head_sha: f8c2af988
task_bucket: reviews/task-167
packet: 023-review-fixes
timestamp: 2026-08-12 (America/Los_Angeles)
fixture: ecaz dev distann-multicluster local-multinode-pg18 --pg 18 --nodes 3 --physical-benchmark
database: tqvector_bench; PostgreSQL 18.3
suite_config: artifacts/task167-physical-suite.json
suite_manifest: artifacts/suite-manifest.json
results: artifacts/results.jsonl

This packet uses a bespoke three-owner physical suite because the canonical
intel-local lane is a single-index HNSW/IVF matrix and cannot express the
published-generation owner-routing, coordinator-outside-roster, physical
DML, two-writer, routed DELETE/VACUUM, UPDATE, and mid-insert-fault surfaces.
The config therefore preserves the prior Task 167 physical matrix while
capturing the corrected counters, isolated inserted-neighborhood parity, and
the new concurrency/delete drills.

The release installation and suite command, exact installed extension `.so`
path/profile, head SHA, topology, commands, and cited result lines will be
recorded below as the run completes. No corpus, cluster directory, polling
exhaust, or transient cache belongs in this packet.
