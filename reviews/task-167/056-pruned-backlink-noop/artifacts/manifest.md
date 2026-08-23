# Task 167 packet 056 artifact manifest

- Head under review: `3da6df06cd8f2428212e492535987e993a4658cf`.
- Candidate code checkpoints:
  - `5e32a1dfb2e5d35ffe365c8bb013f43cc3bdbb34` implements the full-target
    pruned-backlink no-op, attributed counter, harness labels, and focused
    regressions.
  - `3da6df06cd8f2428212e492535987e993a4658cf` explicitly scopes the physical
    no-op to targets already at `graph_degree`, keeping spare-capacity policy
    unchanged and matching the local planner precedent.
- Owning packet: `reviews/task-167/056-pruned-backlink-noop/`.
- Timestamp: `2026-08-22`.
- Scope: preserve a full target when robust-prune rejects the proposed
  backlink, retain stale-neighbor cleanup, add one attributed insert-work
  counter, and align candidate/control harness labels.
- Source precedent:
  `src/am/ec_diskann/insert.rs::plan_backlink_mutation` returns no mutation
  when the selected full-target set excludes the new backlink.
- Decision context: packets 047, 051, and 054 varied free-capacity behavior;
  none changed this full-target rejection behavior.
- No benchmark result is claimed. A separately preregistered clean 50k gate
  must disposition the candidate before any final scale matrix.

## Validation

Focused validation logs are recorded at exact head
`3da6df06cd8f2428212e492535987e993a4658cf`.

- `product-test.log`
  - Command: `cargo test -p ecaz --no-default-features --features pg18 backlink_prune_rejection_preserves_full_target_order --lib`
  - Result: PASS, 1 passed, 0 failed, 2577 filtered out.
  - SHA-256: `f7f90207a1ba2bdb8dade0bba11fa6a5f0bb1fda035d8d26aa47aa733997d92a`.
- `counter-test.log`
  - Command: `cargo test -p ecaz --no-default-features --features pg18 stage_and_insert_resets_are_independent --lib`
  - Result: PASS, 1 passed, 0 failed, 2577 filtered out.
  - SHA-256: `df1bfe826e7f75ce8da877b86f839e690d45bb4c0800a13c68486c58ecb66ff8`.
- `parser-test.log`
  - Command: `cargo test -p ecaz-cli --no-default-features commands::bench::suite::tests::distann_task167_quality_and_insert_metrics_are_structured -- --exact`
  - Result: PASS, 1 passed, 0 failed, 510 filtered out.
  - SHA-256: `3000fb18c355780fcd7056b4cdefc58122a4e9a140d44f41215d8b389556d3e7`.
- `gate-control-test.log`
  - Command: `cargo test -p ecaz-cli --no-default-features task167_quality_gate`
  - Result: PASS, 2 passed, 0 failed, 509 filtered out.
  - SHA-256: `9e29ef3ef178ba53c16c07b22203018d3e101b7f8d9ef08010d6eb2ae82524d2`.
- `fmt-check.log`
  - Command: `cargo fmt --all -- --check`
  - Result: PASS (exit 0); stable rustfmt emitted only the repository's known
    nightly-option warnings.
  - SHA-256: `f59e08ebbd63f2f05789739143aa906d592fe984cce588bf6389a6cd4091a810`.
- `diff-check.log`
  - Command: `git diff --check 5e32a1dfb^..HEAD`
  - Result: PASS (exit 0).
  - SHA-256: `a7d693d9437dd00dfeb93ae4fa077eae4489d83393868f52c54612063a7db3cd`.
