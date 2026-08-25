# Task 236 current-main integration validation

- Head SHA: `48ea5d506c781ec92cfa91b0b756540f3b8cd8cd`
- Task / packet: `236`, `reviews/task-236/005-main-integration/`
- Date: `2026-08-24`
- Target: PG18, shared external Cargo target (`/home/peter/.cargo-target`)
- Formatting: not run

| Artifact | Command | Result |
| --- | --- | --- |
| `cargo-check-pg18.log` | `cargo check --lib --no-default-features --features pg18` | passed, no warnings |
| `remote-postgres-tls-tests.log` | `cargo test --lib --no-default-features --features pg18 remote_postgres_tls` | 8 passed |
| `secure-suite-contract-test.log` | `cargo test -p ecaz-cli distann_local_multinode_expands_secure_remote_transport` | 1 passed |
| `task167-regression-preservation-test.log` | `cargo test -p ecaz-cli distann_task167_heldout_regression_gate_is_step_local_and_paired` | 1 passed |

The authoritative integration benchmark is
`benchmarks/task236-distann-secure-transport-main-integration-ab-r2/`, produced
from this exact clean SHA. Its own manifest records the fixture, commands, and
10k/50k/100k results.
