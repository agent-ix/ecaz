head_sha: ae19b8ce0f8c311d6fd8f386d85241592aa95ab3
task_bucket: reviews/task-167
packet: 023-review-fixes
timestamp: 2026-08-12 (America/Los_Angeles)
fixture: ecaz dev distann-multicluster local-multinode-pg18 --pg 18 --pgbin /home/peter/.pgrx/18.3/pgrx-install/bin --nodes 2 --rows 100 --dim 4 --graph-degree 8 --skip-fault-drills --allow-debug-extension --remote-insert-probe
database: tqvector_bench; PostgreSQL 18.3
suite_config: artifacts/task167-physical-suite.json
validation: artifacts/validation.log
remote_insert_probe: artifacts/remote-insert-drill.log

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

Build validation completed from head `ae19b8ce0` with `cargo check -p ecaz
--features pg18`, `cargo check -p ecaz-cli`, and a debug PG18 install. The
packet-local remote-insert drill ran with extension SHA `ae19b8ce0`, committed
candidates 0 and 1 locally, then committed candidate 2 on remote owner ordinal
1, and reported `physical_remote_insert_probe pass=true`. The same short run
later stopped in the existing two-writer backlink drill with
`EC_REMOTE_WRITE: backlink failed: db error`; consequently the new
back-edge assertion did not pass and no concurrency result is claimed. No
refreshed 10k/50k/100k benchmark results are claimed. No corpus, cluster
directory, polling exhaust, or transient cache belongs in this packet.
