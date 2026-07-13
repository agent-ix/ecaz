# Artifact manifest

- Task bucket / packet: `reviews/task-179/061-post-closeout-hardening`
- Branch: `task-179-ec-distann-physical-shards`
- Source SHA: `7bb215458823908397cb76ee2b4630da8d989d65`
- Host / PostgreSQL: local Intel x86_64 / PostgreSQL 18.3
- Created: `2026-07-13` (America/Los_Angeles)
- Fixture / storage / rerank: focused safety/resource validation; not a
  benchmark measurement and no corpus is used
- Isolation: each focused PG18 command runs one named pg_test

## Decision-grade artifacts

| Artifact | Command | Result | SHA-256 |
| --- | --- | --- | --- |
| `interrupt-unit.log` | `cargo test --lib --no-default-features --features pg18 interrupt_poll_accepts_cancel_and_backend_termination` | 1 passed, 0 failed, 2508 filtered | `583d714b43dbf1de2d1250590b1dfe490891c49127f108716a9012d900394c66` |
| `custom-scan-rollback-pg18.log` | `cargo pgrx test pg18 test_distann_control_metadata_and_fail_closed` | 1 passed, 0 failed, 2508 filtered; asserts exactly one query-context cleanup on rollback | `38c512caf1348189344e6470422058303f36583f1a26fe3648bb0c481957a459` |
| `cancel-reuse-pg18.log` | `cargo pgrx test pg18 test_ec_distann_remote_transport_cancel_then_reuse` | 1 passed, 0 failed, 2508 filtered; prompt live cancellation and reuse | `68f78e7a6096abd964ad64eeef75143bd45c8a126f6875880e0a4739c31971ec` |
| `clippy-pg18.log` | `cargo clippy --lib --no-default-features --features pg18 -- -D warnings` | pass | `8c90f918aee0c672bb6ba66734441d1646b57ed4348cf294c81bd596c93bff73` |

The logs were captured with `script -q -e -c`, retain their command exit code,
and were normalized to LF before hashing. No corpus, benchmark output,
PostgreSQL operational log, polling snapshot, or generated cache is included.
