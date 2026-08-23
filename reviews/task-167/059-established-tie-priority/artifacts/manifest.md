# Task 167 packet 059 artifact manifest

- Head under review: `ddea621a61dd19e0c4c946b5a0627a57a5dae4dc`.
- Candidate code checkpoints:
  - `350385ce9fe7158286ce6570383f8f44828fe671` centralizes established-first
    backlink selection, routes the physical path through it, and adds the
    exact-tie regression.
  - `ddea621a61dd19e0c4c946b5a0627a57a5dae4dc` changes only candidate/control
    benchmark labels and the parser fixture for attributable measurement.
- Owning packet: `reviews/task-167/059-established-tie-priority/`.
- Timestamp: `2026-08-22`.
- Scope: align physical backlink tie-breaking with the established-first
  full-target union order used by the pure DistANN planner, mature local
  incremental planner, and batch Vamana; make the candidate explicit in suite
  output.
- Decision context: packet 057's no-op candidate preserved 702 pruned targets
  but did not move heldout recall. It could not affect exact-vector ties because
  proposal-first ordering made the new duplicate win before the no-op guard.
- No benchmark result is claimed. A separately preregistered clean 50k gate is
  required before this candidate can be retained or rejected.

## Validation

Focused validation logs are recorded at exact head
`ddea621a61dd19e0c4c946b5a0627a57a5dae4dc`.

- `product-test.log`
  - Command: `cargo test -p ecaz --no-default-features --features pg18 backlink_exact_tie_prefers_the_established_neighbor --lib`.
  - Result: PASS, 1 passed, 0 failed, 2577 filtered out.
  - SHA-256: `2222f5151966bb00dea045e5618bd06a52bfb4041477db7f9840f27563c730f3`.
- `parser-test.log`
  - Command: `cargo test -p ecaz-cli --no-default-features commands::bench::suite::tests::distann_task167_quality_and_insert_metrics_are_structured -- --exact`.
  - Result: PASS, 1 passed, 0 failed, 510 filtered out.
  - SHA-256: `cd0c36b049b84490087e3bbe4c93cce13acecd0bed96f6ba1c2140ce46da3747`.
- `gate-control-test.log`
  - Command: `cargo test -p ecaz-cli --no-default-features task167_quality_gate`.
  - Result: PASS, 2 passed, 0 failed, 509 filtered out.
  - SHA-256: `36f732e55e593006135e799b519079e54799d3be7fe58364b9642fe6b2a36e35`.
- `fmt-check.log`
  - Command: `cargo fmt --all -- --check`.
  - Result: PASS (exit 0); stable rustfmt emitted only the repository's known
    nightly-option warnings.
  - SHA-256: `30c17fa246da8a43314ac0f629694882b39087d7e9d89b0e20bce60964f350ea`.
- `diff-check.log`
  - Command: `git diff --check 350385ce9^..HEAD`.
  - Result: PASS (exit 0).
  - SHA-256: `3db8230879c0e37f84bd5afbbd773e19658be5c1e75d0f254815a3d1b2954f6a`.
