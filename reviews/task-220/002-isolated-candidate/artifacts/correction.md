# Task 220 reviewer correction response

Reviewer feedback `2026-08-09-01-reviewer.md` identified that the benchmark
implementation had accidentally made the regressed packed SQL the featureless
production path. The correction is committed as `c8b5fd9ee`.

Corrected production boundaries:

- `generation_read.rs`: the featureless `materialize_payloads` path selects
  `build_payload_sql`; the benchmark-feature path still selects packed SQL
  only when `ec_distann.benchmark_packed_payload=on`.
- `generation_read.rs`: the non-profile production endpoint passes
  `use_packed_payload=false`.
- `remote_endpoint.rs`: FR-079 owner SQL uses `build_payload_sql`, decodes the
  legacy `bytea[]`, and flattens it into cumulative offsets plus one byte
  buffer for the unchanged packed wire ABI.

Validation completed:

- `cargo check -p ecaz`
- `cargo check -p ecaz --features distann-head-attribution-benchmark`
- `cargo check -p ecaz --tests`
- `cargo check -p ecaz --tests --features distann-head-attribution-benchmark`
