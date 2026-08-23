# Task 167 packet 052 artifact manifest

- Head under review: `58dfae5686ca2ba562c56e1d497e771018c6262d` — reject
  the append-when-room default after packet 051's measured-negative 50k result.
- Owning packet: `reviews/task-167/052-reject-append-default/`.
- Timestamp: `2026-08-22`.
- Scope: restore the pre-candidate default and align Task 167 benchmark labels,
  excluded-arm provenance, and diagnostic ordering with that disposition.
- Decision evidence: `reviews/task-167/051-batch-consistent-50k-gate/`.
- No benchmark result is claimed in this code-review packet.

## Validation

- Command:
  `cargo test -p ecaz --no-default-features --features pg18 distann_default_backlink_strategy_retains_measured_robust_prune`.
- Result: passed, 1/1. Artifact: `cargo-test-default.log`, LF-normalized
  SHA-256
  `a317729d069335598b9bbce257ef77eb0f6bb6fb41f2109e5eb31a2ff6c42be4`.
- Command:
  `cargo test -p ecaz-cli --no-default-features commands::bench::suite::tests::distann_task167_quality_and_insert_metrics_are_structured -- --exact`.
- Result: passed, 1/1. Artifact: `cargo-test-suite-parser.log`, LF-normalized
  SHA-256
  `8574ed25f10ae2214a620bc0795eedaf98688d96ae920e4356e2fb07523e8410`.
- Command:
  `cargo test -p ecaz-cli --no-default-features task167_quality_gate`.
- Result: passed, 2/2. Artifact: `cargo-test-quality-gate.log`, LF-normalized
  SHA-256
  `346dbec45b84ba37babb7a4c28ff05425b470916ae8e939d8be15cff579b5f94`.
- `cargo fmt --all -- --check` passed before the code checkpoint, with only
  the repository's stable-toolchain warnings about nightly-only import
  grouping options.
- `git diff --check` passed before the code checkpoint and review packet.
