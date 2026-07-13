# Artifact manifest

- Task bucket / packet: `reviews/task-179/060-recovery-state-closeout`
- Branch: `task-179-ec-distann-physical-shards`
- Production remediation SHA: `772728b2379f441e38c53e4b071f28b1b390be6b`
- Final source/test SHA: `87ea7a42753df547a11eec96d8bced90738ac66c`
- Host / PostgreSQL: local Intel x86_64 / PostgreSQL 18.3
- Created: `2026-07-13` (America/Los_Angeles)
- Fixture / storage / rerank: aggregate Task 179 correctness validation; not a
  benchmark measurement and no corpus is used
- Isolation: serial libtest execution (`RUST_TEST_THREADS=1`) because PG fixture
  processes and global GUC/cache tests are not parallel-safe

## Decision-grade artifacts

| Artifact | Source SHA | Command | Result | SHA-256 |
| --- | --- | --- | --- | --- |
| `focused-pg18.log` | `772728b23` | `cargo pgrx test pg18 test_distann_multi_epoch_publish` | 1 passed, 0 failed, 2507 filtered; includes the new registration-skew drill | `ac332584a2aa1433dc0eb2aabf8bfe96bbdec31d051f452563b95e391eff31e6` |
| `distann-pg18-green.log` | `87ea7a427` | `RUST_TEST_THREADS=1 cargo pgrx test pg18 distann` | 238 passed, 0 failed, 3 ignored, 2267 filtered; 21/21 DistANN on-disk fixtures also pass | `a23634b0cec8f8a9aad2e55ece4dcccc8cca5627afa8f0c7718f2a21b51c47c4` |
| `clippy-pg18-final.log` | `87ea7a427` | `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` | pass | `adbe6f5c4a2b93785746e00db493044918cc222469d6c4aed1290c15a7a0c358` |
| `full-crate-diagnostic.md` | `87ea7a427` | diagnostic summary of the non-gate full-crate probe | three reproducible unrelated TurboQuant failures; no Task 179 failure | n/a (text added with this packet) |

The raw failed/interrupted exploratory captures were pruned rather than
committed. They are not acceptance evidence. No corpus, PostgreSQL operational
log, polling snapshot, or generated cache is included.
