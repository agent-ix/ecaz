# Task 185 arbitrary-head attribution manifest

- Task bucket / packet: `reviews/task-185/004-arbitrary-head-attribution/`
- Implementation head: `815ff3551`
- Scope: benchmark-only source checkpoint; no runtime benchmark result
- Fixture: none in this packet
- Storage / lane: physical sharded path intended; no run performed
- Runner: not yet wired; next checkpoint must use `ecaz bench suite`

## Validation artifacts

- `pg18-feature-check.log` — PG18 feature build with
  `distann-head-attribution-benchmark`; pass, exit 0; SHA-256
  `c779510e8529f45dacc182621b3f410feab2e01db09f883067344888c46b7ee3`.
- `pg18-featureless-check.log` — PG18 featureless build; pass, exit 0;
  SHA-256 `f46ae876c7eee3537e409d09de912728967717674fc31e3f02405f96aa97a119`.
- `git-diff-check.log` — `git diff --check`; pass, exit 0; SHA-256
  `3c9edd68d0ae313b1a789d0b4f2bbabe72f688946be3d4ca5202495eef0ae764`.

The packet intentionally contains no corpus, truth cache, cluster directory,
polling output, or benchmark result. Packet 003 remains the source of the
100k control and returned-seed attribution evidence.

## Provenance

- PG18 config: `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- Feature command: `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check -q -p ecaz --no-default-features --features pg18,distann-head-attribution-benchmark`
- Featureless command: `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check -q -p ecaz --no-default-features --features pg18`
- Source validation command: `git diff --check`
