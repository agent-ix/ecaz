# Task 167 packet 053 artifact manifest

- Head under review: `4826e96447911d33e915943f591eebdf6a80ce06`.
- Owning packet: `reviews/task-167/053-conservative-backlink-admission/`.
- Timestamp: `2026-08-22`.
- Scope: conservative free-capacity robust-prune admission, one new attributed
  insert-work counter, and truthful candidate/control harness labels.
- Before evidence:
  - packet 047 robust-prune-all heldout deficit `0.008611` against allowance
    `0.007000`;
  - packet 051 unconditional append heldout deficit `0.010611` against the
    same allowance.
- No benchmark result is claimed in this code-review packet. A separately
  preregistered isolated 50k gate owns candidate disposition.

## Validation

- Command:
  `cargo test -p ecaz --no-default-features --features pg18 am::ec_distann::insert::tests --lib`.
- Result: passed, 12/12. This includes positive admission, displacement
  rejection, exact-equivalent entrance, full-target re-prune, idempotence,
  degree, and metric-contract coverage. Artifact: `cargo-test-insert.log`,
  LF-normalized SHA-256
  `f0af532780f6192ac7fbaaf34706f304f6a486beccef8d2ad076f8ec6c0b97d6`.
- Command:
  `cargo test -p ecaz --no-default-features --features pg18 distann_default_backlink_strategy_uses_conservative_prune_admission --lib`.
- Result: passed, 1/1. Artifact: `cargo-test-default.log`, LF-normalized
  SHA-256
  `67f2e970a3c19979a23e2671b3da9c72c0f9a7c603a54e5a30eb83e0453e1968`.
- Command:
  `cargo test -p ecaz --no-default-features --features pg18 stage_and_insert_resets_are_independent --lib`.
- Result: passed, 1/1. Artifact: `cargo-test-counter-reset.log`, LF-normalized
  SHA-256
  `225ed26e55f76402f604d94540f6cadfd4338040d9e85e442bf8dd8b472e319f`.
- Command:
  `cargo test -p ecaz-cli --no-default-features commands::bench::suite::tests::distann_task167_quality_and_insert_metrics_are_structured -- --exact`.
- Result: passed, 1/1. Artifact: `cargo-test-suite-parser.log`, LF-normalized
  SHA-256
  `ce5d757b5563d4e7e2f52f556fe88b145353b12a197ec7bbc27c8a245122f948`.
- Command:
  `cargo test -p ecaz-cli --no-default-features task167_quality_gate`.
- Result: passed, 2/2. Artifact: `cargo-test-quality-gate.log`, LF-normalized
  SHA-256
  `684ef50fe4851bd376294ff0b34a0c621d11fc1f4c19a51163254296ffb56221`.
- `cargo fmt --all -- --check` passed before the code checkpoint, with only
  the repository's stable-toolchain warnings for nightly-only import grouping.
- `git diff --check` passed before both the code and packet checkpoints.
