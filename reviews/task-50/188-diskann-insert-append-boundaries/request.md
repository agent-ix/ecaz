# Review Request: DiskANN Insert Append Boundaries

## Scope

Reviews code commit `f40cacf560ebde2a8729171dca6a261874a0c6fc`.

This slice consolidates DiskANN insert append unsafe blocks:

- Duplicate overflow append and live-node append now choose the target block
  and call the append helper inside one append contract.
- `append_raw_tuple_payload` now opens the target buffer under one relation/page
  contract.
- New-page initialization, existing-page free-space checks, recursive P_NEW
  retry, and `PageAddItemExtended` now share one WAL-registered append-page
  contract.

The payload validation, page locking, free-space retry, WAL finish, and returned
`ItemPointer` behavior remain unchanged.

## Unsafe Movement

- Previous packet 187 ledger: `1789` direct unsafe rows under `src/`
- Packet 188 ledger: `1783` direct unsafe rows under `src/`
- Net reduction: `6`
- `src/am/ec_diskann/insert.rs`: `31 -> 25` direct unsafe rows

## Validation

Artifacts are under `artifacts/`.

- `cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with existing `src/am/mod.rs` unused import warnings.
- `cargo-check-pg18-pg-test.log`: `cargo check --all-targets --no-default-features --features pg18,pg_test` passed with existing Hadamard test-helper dead-code warnings.
- `cargo-test-diskann-unique-insert-pg18-no-run.log`: targeted DiskANN unique insert test binary build passed.
- `cargo-pgrx-test-diskann-unique-insert-pg18-blocked.log`: targeted PG18 pgrx run was blocked before the test body by the existing local `BufferBlocks` symbol lookup failure.
- `rustfmt-diskann-insert-check.log`: touched-file rustfmt check passed; stable rustfmt emitted the known unstable option warnings.
- `git-diff-check.log`: `git diff --check HEAD~1..HEAD` passed.
- `unsafe-block-count.log`: records remaining direct unsafe rows in `src/am/ec_diskann/insert.rs`.
- `unsafe-ledger-generate.log`: regenerated Task 50 ledger with `1783` rows.
- `unsafe-ledger-check.log`: ledger covers current `src/` unsafe rows.

