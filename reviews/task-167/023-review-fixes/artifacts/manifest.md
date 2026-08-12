head_sha: 28f2d8213
task_bucket: reviews/task-167
packet: 023-review-fixes
timestamp: 2026-08-12 (America/Los_Angeles)
fixture: ecaz dev distann-multicluster local-multinode-pg18 --pg 18 --nodes 3 --physical-benchmark
database: tqvector_bench; PostgreSQL 18.3
suite_config: artifacts/task167-physical-suite.json
suite_manifest: artifacts/suite-manifest.json
results: artifacts/results.jsonl
validation: artifacts/validation.log

This packet uses a bespoke three-owner physical suite because the canonical
intel-local lane is a single-index HNSW/IVF matrix and cannot express the
published-generation owner-routing, coordinator-outside-roster, physical
DML, two-writer, routed DELETE/VACUUM, UPDATE, and mid-insert-fault surfaces.
The config therefore preserves the prior Task 167 physical matrix while
capturing the corrected counters, isolated inserted-neighborhood parity, and
the new concurrency/delete drills.

The suite's throughput criterion is explicitly relative A/B only: FR-083 has
no absolute insert-latency gate, so the packet reports absolute rows/s and
physical/control ratio without treating `pass=true` as an absolute SLA. The
`beam_width=4, hop_rounds=100` setting is deliberate for this packet's
incremental-write attribution; read-side regime correction remains owned by
Task 203 follow-up work and is not presented as a corrected read benchmark.

Build validation completed from head `28f2d8213` with `cargo check -p ecaz
--features pg18` and `cargo check -p ecaz-cli`. A release install/build was
attempted with `cargo pgrx install --release --pg-config
/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features
--features pg18` and the equivalent `cargo build --release` command, but the
shared-host optimized build did not complete in this session; no stale
release `.so` is cited as current-head evidence. The focused pgrx ACL runner
was likewise stopped after it became silent without returning a test result.
The packet therefore remains review-open and does not claim refreshed
benchmark numbers. No corpus, cluster directory, polling exhaust, or
transient cache belongs in this packet.
