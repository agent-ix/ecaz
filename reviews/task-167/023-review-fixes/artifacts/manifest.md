head_sha: c7e2f4a5c7e1c43fd361cc327f6f4995423d601b
task_bucket: reviews/task-167
packet: 023-review-fixes
timestamp: 2026-08-12 (America/Los_Angeles)
fixture: ecaz dev distann-multicluster local-multinode-pg18 --pg 18 --pgbin /home/peter/.pgrx/18.3/pgrx-install/bin --nodes 2 --rows 100 --dim 4 --graph-degree 8 --skip-fault-drills --allow-debug-extension --remote-insert-probe --artifact-dir reviews/task-167/023-review-fixes/artifacts --log-file reviews/task-167/023-review-fixes/artifacts/remote-insert-drill.log
database: tqvector_bench; PostgreSQL 18.3
suite_config: artifacts/task167-physical-suite.json
validation: artifacts/validation.log
remote_insert_probe: artifacts/remote-insert-drill.log
summary: artifacts/distann-multinode-summary.log

This packet uses a bespoke physical suite because the canonical
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

Build validation completed from head `c7e2f4a5c` with PG18 compile/build and
install. The exact two-owner fixture passed remote insert/owner proof,
mid-insert rollback, concurrent insert/query, routed DELETE + VACUUM with an
owner tombstone, and the topology gate. The packet-local log records
`physical_remote_insert_probe pass=true`, `physical_concurrent_insert_query
pass=true`, and `physical_routed_delete_vacuum pass=true`. The final install
was from the committed code head; the runtime provenance line may carry the
fixture's packet-artifact dirty suffix. No refreshed 10k/50k/100k benchmark
results are claimed. No corpus, cluster directory, polling exhaust, or
transient cache belongs in this packet.
