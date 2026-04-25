# Review Request: Task 28 IVF PG18 Validation Gates

Status: open
Owner: coder2
Date: 2026-04-25
Branch: `task28-ivf`
Code checkpoint: `33bfff75f2e3df304886345e5ca92f51b7dcf573`

## Scope

- Close the Phase 8 PG18 unit, extension, and lint validation gates.
- Keep validation PG18-only; no PG17 tests were run.
- Fix PG18 clippy findings surfaced by the validation gate:
  - use the tree-height callback function directly in `pgrx_extern_c_guard`;
  - replace an oversized debug tuple with `EcIvfRescanDebugSnapshot`;
  - simplify the PG18 stats counter assertion.
- Update `plan/tasks/28-ivf-access-method.md` to mark unit, extension, and
  lint gates complete while leaving measurement gates open.

## Files

- `src/am/ec_ivf/cost.rs`
- `src/am/ec_ivf/scan.rs`
- `src/lib.rs`
- `plan/tasks/28-ivf-access-method.md`
- `review/30033-task28-ivf-pg18-validation/artifacts/manifest.md`
- `review/30033-task28-ivf-pg18-validation/artifacts/*.log`

## Validation

- `cargo test --no-default-features --features pg18 --lib -- --skip pg_test`
  - `372 passed; 0 failed; 250 filtered out`
- `cargo pgrx test pg18`
  - main pgrx suite: `618 passed; 0 failed; 4 ignored`
  - proptest and integration/doc-test follow-ons also passed
- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings`
- `git diff --check`

Raw logs and diagnostic failed-attempt logs are stored under
`review/30033-task28-ivf-pg18-validation/artifacts/`; see
`artifacts/manifest.md`.

## Review Focus

- Whether the validation-gate interpretation is appropriate: pure Rust unit
  tests use `--skip pg_test`, and SQL callback behavior is covered by the full
  `cargo pgrx test pg18` suite.
- Whether the `EcIvfRescanDebugSnapshot` debug helper is a reasonable
  replacement for the previous large tuple return.
- Whether the task plan should now move to recall, latency, storage, and WAL
  measurement gates.

## Non-Goals

- Recall/latency/storage/WAL measurement claims.
- PG17 validation.
- New IVF runtime behavior.
