# Review Request: DiskANN Vacuum Pass Boundaries

## Scope

Reviews code commit `08f0d0584e2486b085c84fb299797a0af7f786f4`.

This slice consolidates DiskANN vacuum-path unsafe blocks:

- No-op vacuum stats allocation and update now share one callback-scoped stats
  contract.
- Successful bulkdelete medoid-refresh marking and stats update now share one
  applied-pass contract.
- Bulkdelete pass block-count read and persisted-chain materialization now
  share one index-relation read contract.
- Vacuum neighbor repair heap-relation resolution and fill planning now share
  one repair-fill contract.

The rewrite, tuple validation, callback invocation, and page-locking behavior
remain unchanged.

## Unsafe Movement

- Previous packet 186 ledger: `1793` direct unsafe rows under `src/`
- Packet 187 ledger: `1789` direct unsafe rows under `src/`
- Net reduction: `4`
- `src/am/ec_diskann/routine.rs`: `54 -> 50` direct unsafe rows

## Validation

Artifacts are under `artifacts/`.

- `cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with existing `src/am/mod.rs` unused import warnings.
- `cargo-check-pg18-pg-test.log`: `cargo check --all-targets --no-default-features --features pg18,pg_test` passed with existing Hadamard test-helper dead-code warnings.
- `cargo-test-diskann-vacuum-noop-pg18-no-run.log`: targeted DiskANN vacuum test binary build passed.
- `cargo-pgrx-test-diskann-vacuum-noop-pg18-blocked.log`: targeted PG18 pgrx run was blocked before the test body by the existing local `BufferBlocks` symbol lookup failure.
- `rustfmt-diskann-routine-check.log`: touched-file rustfmt check passed; stable rustfmt emitted the known unstable option warnings.
- `git-diff-check.log`: `git diff --check HEAD~1..HEAD` passed.
- `unsafe-block-count.log`: records remaining direct unsafe rows in `src/am/ec_diskann/routine.rs`.
- `unsafe-ledger-generate.log`: regenerated Task 50 ledger with `1789` rows.
- `unsafe-ledger-check.log`: ledger covers current `src/` unsafe rows.

