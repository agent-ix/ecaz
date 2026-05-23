# Review Request: IVF Scan Rerank Boundaries

## Scope

Reviews code commit `3331d5408d7b7a472a9de96cf38218580539b4a4`.

This slice consolidates IVF scan unsafe blocks around scan-local allocation,
heap-f32 rerank setup, heap rerank reader/prefetch setup, and debug order-by
reads. It intentionally keeps the existing raw PostgreSQL pointer contracts in
place instead of introducing broad safe wrappers.

Key changes:

- `palloc_copy_slice` now treats PostgreSQL allocation and the non-overlapping
  copy as one scan-local allocation contract.
- `configure_heap_rerank_state` resolves heap relation, snapshot, and indexed
  source attribute inside one audited scan-descriptor boundary.
- Heap-f32 rerank reader construction and relation-block prefetch now share the
  same owned-rerank-state contract.
- Debug order-by helpers check scan descriptor fields and read the first
  order-by null/value slots inside one audited block per helper.

## Unsafe Movement

- Previous packet 183 ledger: `1808` direct unsafe rows under `src/`
- Packet 184 ledger: `1801` direct unsafe rows under `src/`
- Net reduction: `7`
- `src/am/ec_ivf/scan.rs`: `34 -> 27` direct unsafe rows

## Validation

Artifacts are under `artifacts/`.

- `cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with existing `src/am/mod.rs` unused import warnings.
- `cargo-check-pg18-pg-test.log`: `cargo check --all-targets --no-default-features --features pg18,pg_test` passed with existing Hadamard test-helper dead-code warnings.
- `cargo-test-ivf-gettuple-pg18-no-run.log`: targeted IVF gettuple test binary build passed.
- `cargo-pgrx-test-ivf-gettuple-pg18-blocked.log`: targeted PG18 pgrx run was blocked before the test body by the existing local `BufferBlocks` symbol lookup failure.
- `rustfmt-ivf-scan-check.log`: touched-file rustfmt check passed; stable rustfmt emitted the known unstable option warnings.
- `git-diff-check.log`: `git diff --check HEAD~1..HEAD` passed.
- `unsafe-block-count.log`: records remaining direct unsafe rows in `src/am/ec_ivf/scan.rs`.
- `unsafe-ledger-generate.log`: regenerated Task 50 ledger with `1801` rows.
- `unsafe-ledger-check.log`: ledger covers current `src/` unsafe rows.

