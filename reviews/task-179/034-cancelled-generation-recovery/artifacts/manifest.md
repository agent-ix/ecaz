# Artifact manifest

- Head SHA: `1b531ce9fcb97268cb027de29dd52dd2441b72a7`
- Task / packet: `task-179` / `reviews/task-179/034-cancelled-generation-recovery`
- Captured: `2026-07-12T08:40:49-07:00`
- Lane: PG18 cancellation lifecycle, TC-050 wire format, layout/upgrade invariants
- Fixture / storage / rerank: committed single-participant Published-before-swap lifecycle fixture; physical generation storage; rerank not applicable
- Isolation shape: one logical control and one local participant generation; the participant publication is committed separately before coordinator cancellation to model a remote partial acknowledgement. This is not benchmark evidence.

## `pg18-cancelled-recovery.log`

- Command: `cargo pgrx test pg18 test_distann_multi_epoch_publish`
- Result: pass
- Key line: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2495 filtered out`
- Key assertions: the participant reaches `Published` while the coordinator
  pointer remains on the predecessor; cancellation remains audited and
  non-activatable; `ec_distann_recover_cancelled_publish` creates a
  `Published`-origin tombstone before physical deletion, records coordinator
  completion only afterward, and exact replay succeeds.

## `cancel-audit-fixture.log`

- Command: `cargo test --no-default-features --features pg18 distann_cancel_publish_audit_v1_fixture_decodes_independently_and_rejects_version_swap`
- Result: pass
- Key line: `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 73 filtered out`
- Fixture: `fixtures/on-disk/distann_cancel_publish_audit_v1.hex`
- Domain digest: `42a9358f6ec4998673293572fffba5db37127c328d5e7fd2141ac34a9dc2bb53`

## `layout-upgrade.log`

- Command: `cargo test --no-default-features --features pg18 --test size_of_assertions --test upgrade_matrix`
- Result: pass
- Key lines: `13 passed; 0 failed` and `2 passed; 0 failed`

## `clippy-pg18.log`

- Command: `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- Result: pass
- Key line: `Finished dev profile [unoptimized + debuginfo] target(s)`

No recall, latency, storage, or benchmark result is claimed by this packet.
