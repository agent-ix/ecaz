# Task 227 packet 004 artifact manifest

- Code SHA: `5864619a6298561f3a608aaca90b7226283fc593`
  (`task227 add persisted graph diagnostics`)
- Task bucket / packet: `reviews/task-227/004-graph-diagnostics/`
- Lane: PG18 feature-only diagnostic tooling; no corpus benchmark measurement
- Timestamp: 2026-08-24 05:18 PDT (America/Los_Angeles)
- Fixture: pure CLI graph/suite tests and one isolated one-index-at-a-time,
  two-row PG18 physical-generation plus monolithic-control callback fixture
- Storage format / rerank: RaBitQ persisted graph; no scan, rerank, quantizer,
  posting, or production storage behavior change

Artifacts:

- `graph-diagnostic-cli-tests.log`
  - Head SHA: `5864619a6298561f3a608aaca90b7226283fc593`
  - Command: `cargo test -p ecaz-cli graph_diagnostic`
  - Result: 4 passed, 0 failed
  - Covers deterministic adjacency digests; SCC/weak-component, degree,
    edge-class, articulation/bridge, and seed-reachability summaries; an
    iterative 20,000-node chain; suite validation, expansion, and artifact
    declaration.
- `graph-diagnostic-feature-build.log`
  - Head SHA: `5864619a6298561f3a608aaca90b7226283fc593`
  - Command: `cargo test --lib --no-default-features --features pg18,distann-head-attribution-benchmark graph_diagnostic`
  - Result: feature-gated PG18 extension compiled; 0 selected tests failed.
- `graph-diagnostic-pg18-callback.log`
  - Head SHA: `5864619a6298561f3a608aaca90b7226283fc593`
  - Command: `cargo pgrx test pg18 test_distann_graph_diagnostic_chunks`
  - Result: 1 passed, 0 failed (unrelated targets filtered)
  - Covers physical retained-generation graph streaming, feature-only SQL
    registration, graph-only tuple reads, owner/cardinality fields, and
    monolithic signed-cursor pagination.

No corpus/query TSV, truth cache, PGDATA cluster, polling output, benchmark
measurement, or shared-table fixture is included. This packet implements the
read-only tooling prerequisite; the 100k physical-versus-monolithic diagnostic
run belongs to packet 005 attribution and remains pending.
