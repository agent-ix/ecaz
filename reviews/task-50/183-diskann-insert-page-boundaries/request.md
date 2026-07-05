# Review Request: DiskANN Insert Page Boundaries

## Scope

Reviews code commit `f101ba384001674caf3abee29f91b82340dfaca0`.

This slice consolidates DiskANN insert unsafe blocks around two page-level
boundaries:

- metadata special-area reads and rewrites in `read_metadata_page` /
  `with_locked_metadata_page`
- backlink tuple line-pointer validation and mutable tuple-byte access in
  `page_tuple_location` / `with_page_tuple_bytes_mut`

The backlink tuple locator now mirrors the previously reviewed vacuum tuple
locator shape: one audited page block validates the offset, line pointer, slot
flags, and tuple bounds, then derives the tuple byte pointer within the locked
page. The byte-slice helper keeps the mutable borrow confined to the visitor.

## Unsafe Movement

- Previous packet 182 ledger: `1816` direct unsafe rows under `src/`
- Packet 183 ledger: `1808` direct unsafe rows under `src/`
- Net reduction: `8`
- `src/am/ec_diskann/insert.rs`: `39 -> 31` direct unsafe rows

## Validation

Artifacts are under `artifacts/`.

- `cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with existing `src/am/mod.rs` unused import warnings.
- `cargo-check-pg18-pg-test.log`: `cargo check --all-targets --no-default-features --features pg18,pg_test` passed with existing Hadamard test-helper dead-code warnings.
- `rustfmt-diskann-insert-check.log`: touched-file rustfmt check passed; stable rustfmt emitted the known unstable option warnings.
- `git-diff-check.log`: `git diff --check HEAD~1..HEAD` passed.
- `unsafe-block-count.log`: records remaining direct unsafe rows in `src/am/ec_diskann/insert.rs`.
- `unsafe-ledger-generate.log`: regenerated Task 50 ledger with `1808` rows.
- `unsafe-ledger-check.log`: ledger covers current `src/` unsafe rows.

