# Artifact Manifest: 30742 SPIRE Local-Cancel Remote-Cancel Primitive

Packet: `30742-spire-local-cancel-remote-cancel-primitive`
Head SHA: `be5be41b2768ff560652dc1b647d5cbdf45d6f80`
Timestamp: `2026-05-10T05:08:37-07:00`

## cargo-fmt-check.log

- Command:
  `script -q -c "cargo fmt --check" review/30742-spire-local-cancel-remote-cancel-primitive/artifacts/cargo-fmt-check.log`
- Lane / fixture / storage format / rerank mode:
  static formatting / none / n/a / n/a
- Surface isolation:
  n/a
- Key result lines:
  script exited with `COMMAND_EXIT_CODE="0"`; rustfmt emitted the repository's
  existing stable-toolchain warnings for unstable import grouping options.

## cargo-check-pg18.log

- Command:
  `script -q -c "cargo check --no-default-features --features pg18" review/30742-spire-local-cancel-remote-cancel-primitive/artifacts/cargo-check-pg18.log`
- Lane / fixture / storage format / rerank mode:
  PG18 static compile / none / n/a / n/a
- Surface isolation:
  n/a
- Key result lines:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 4.32s`

## cargo-test-production-executor.log

- Command:
  `script -q -c "cargo test --no-default-features --features pg18 production_executor_ --lib" review/30742-spire-local-cancel-remote-cancel-primitive/artifacts/cargo-test-production-executor.log`
- Lane / fixture / storage format / rerank mode:
  PG18 Rust state tests / executor state fixtures / n/a / n/a
- Surface isolation:
  in-memory executor-state fixtures; no shared table/index surface
- Key result lines:
  `test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1538 filtered out`
  and the new local-cancel executor state tests passed.

## cargo-pgrx-test-local-cancel.log

- Command:
  `script -q -c "cargo pgrx test pg18 local_cancel" review/30742-spire-local-cancel-remote-cancel-primitive/artifacts/cargo-pgrx-test-local-cancel.log`
- Lane / fixture / storage format / rerank mode:
  PG18 pgrx / loopback transport plus compact-candidate receive local-cancel
  fixtures / RaBitQ for the candidate-receive index / strict receive mode, no
  heap rerank
- Surface isolation:
  candidate-receive fixture creates one table and one SPIRE index dedicated to
  the test; transport fixture uses no index surface
- Key result lines:
  `test tests::pg_test_ec_spire_prod_transport_local_cancel_remote_cancel ... ok`
  `test tests::pg_test_ec_spire_prod_receive_local_cancel_remote_cancel ... ok`
  `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 1551 filtered out`

## git-diff-check.log

- Command:
  `script -q -c "git diff --check -- src/am/ec_spire/root/remote_candidates.rs src/am/mod.rs src/lib.rs plan/tasks/task30-phase11-spire-distributed-production-parity.md plan/design/spire-production-coordinator-executor.md" review/30742-spire-local-cancel-remote-cancel-primitive/artifacts/git-diff-check.log`
- Lane / fixture / storage format / rerank mode:
  whitespace check / changed code and docs / n/a / n/a
- Surface isolation:
  excludes unrelated local `handoff.md`
- Key result lines:
  script exited with `COMMAND_EXIT_CODE="0"` and no whitespace errors.

