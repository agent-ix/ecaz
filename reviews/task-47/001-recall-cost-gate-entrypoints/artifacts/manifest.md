# Artifact Manifest: Task 47 Recall And Cost Gate Entrypoints

- Head SHA: `80d0fe0c002edd3ba3466d8fe2694b5dbcb59410`
- Task bucket: `reviews/task-47/001-recall-cost-gate-entrypoints`
- Timestamp: `2026-05-18`
- Lane: `ecaz bench suite audit` and Makefile dry-run wiring
- Fixture / storage / rerank: fixture assumptions documented in `docs/recall-floors.md`; no live corpus executed
- Surface isolation: no live PostgreSQL recall or EXPLAIN run in this packet

## `make-n-task47-gates.log`

- Command: `make -n recall-gate recall-gate-full cross-am-gate cost-gate`
- Result: pass.
- Key lines:
  - `cargo run -p ecaz-cli -- bench suite run --config fixtures/gates/recall-gate-small.json`
  - `cargo run -p ecaz-cli -- bench suite run --config fixtures/gates/recall-gate-full.json`
  - `cargo run -p ecaz-cli -- bench suite run --config fixtures/gates/cross-am-gate-small.json`
  - `cargo run -p ecaz-cli -- bench suite run --config fixtures/gates/cost-gate-small.json`

## Suite Audits

- `audit-recall-gate-small.log`
  - Command: `target/debug/ecaz bench suite audit --config fixtures/gates/recall-gate-small.json`
  - Result: `[suite:task47-recall-gate-small] audit passed: 3 steps`
- `audit-recall-gate-full.log`
  - Command: `target/debug/ecaz bench suite audit --config fixtures/gates/recall-gate-full.json`
  - Result: `[suite:task47-recall-gate-full] audit passed: 1 steps`
- `audit-cross-am-gate-small.log`
  - Command: `target/debug/ecaz bench suite audit --config fixtures/gates/cross-am-gate-small.json`
  - Result: `[suite:task47-cross-am-gate-small] audit passed: 2 steps`
- `audit-cost-gate-small.log`
  - Command: `target/debug/ecaz bench suite audit --config fixtures/gates/cost-gate-small.json`
  - Result: `[suite:task47-cost-gate-small] audit passed: 2 steps`
