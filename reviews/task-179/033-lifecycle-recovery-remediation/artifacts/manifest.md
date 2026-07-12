# Artifact manifest

- Head SHA: `822949d6d16135d784c57ac57d010f38474f8f88`
- Task / packet: `task-179` / `reviews/task-179/033-lifecycle-recovery-remediation`
- Captured: `2026-07-12T07:43:39-07:00`
- Lane: PG18 lifecycle and static validation
- Fixture / storage / rerank: lifecycle fixture; physical generation storage; rerank not applicable
- Isolation shape: focused single-local lifecycle test with real autocommit boundaries; not a benchmark and not a shared-table measurement

## `pg18-multi-epoch.log`

- Command: `cargo pgrx test pg18 test_distann_multi_epoch_publish`
- Result: pass
- Key line: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2495 filtered out`
- Covered assertions include Pending cancellation, exact predecessor preservation,
  audit fields, durable gate clearing, cancelled-recovery rejection,
  `EC_TRANSACTION_ISOLATION` under Repeatable Read, and subtransaction-abort
  cleanup of a scan token whose Rust guard is bypassed by PostgreSQL ERROR.

## `clippy-pg18.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: pass
- Key line: `Finished dev profile [unoptimized + debuginfo] target(s)`

## `cli-check.log`

- Command: `cargo check -p ecaz-cli`
- Result: pass with the pre-existing unused `LoadedDistributedPlacementConfig.path` warning
- Key line: `Finished dev profile [unoptimized + debuginfo] target(s)`

No benchmark results are claimed by this packet. The accepted immutable Task
172 physical matrix remains under
`reviews/task-172/002-physical-multinode-benchmark/`; Task 179 remains open for
the follow-up performance and lifecycle work named in `request.md`.
