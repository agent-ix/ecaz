# Task 185 suite arbitrary-head trace manifest

- Task bucket / packet: `reviews/task-185/005-suite-arbitrary-head-trace/`
- Implementation head: `9627d36c2`
- Scope: CLI and suite-runner wiring; no runtime benchmark result
- Fixture: none in this packet
- Storage / lane: physical sharded path intended; no run performed
- Required future runner: `ecaz bench suite`

## Validation artifacts

- `pg18-extension-check.log` — PG18 feature check with the attribution
  endpoint enabled; pass, exit 0; SHA-256
  `5041062555b15d3ec941f6f89d80238761883557d39c71ef57dbd38a8f3d4436`.
- `pg18-cli-check.log` — PG18 `ecaz-cli` check; pass, exit 0; SHA-256
  `34cc9c1ed5e77eb160378b3380f51cb674e278c0294a49dcb2adef17b168f16f`.
- `git-diff-check.log` — source whitespace check; pass, exit 0; SHA-256
  `c84a5896727b37cc104d131f6a9533f9d71337817312bf53eecb62e7c4d08d2c`.

No corpus, truth cache, cluster directory, polling output, or benchmark result
is included. Packet 003 remains the immutable source for the 100k control and
returned-seed attribution result.

## Provenance

- PG18 config: `/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config`
- Extension command: `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check -q -p ecaz --no-default-features --features pg18,distann-head-attribution-benchmark`
- CLI command: `PGRX_PG_CONFIG_PATH=/home/peter/.pgrx/18.3/pgrx-install/bin/pg_config cargo check -q -p ecaz-cli`
- Source validation command: `git diff --check`
