# Review Request: HNSW Debug Relation Page Helper Safe Surface

## Summary

This checkpoint centralizes remaining HNSW debug relation page inspection
helpers in `src/am/ec_hnsw/scan_debug.rs`.

The change adds safe debug wrappers for main-fork block counts, shared-lock
buffer reads, and scan opaque null checks. The three graph collector loops now
use those wrappers instead of repeating PostgreSQL relation and buffer unsafe
blocks at call sites.

## Completion Audit Note

Per the closeout gate in `030-comprehensive-unsafe-burndown-plan`, Task 50 is
not complete yet. Current evidence still shows direct unsafe under `src/` and
non-`src`, and the final residual registry / zero-reducible report has not been
produced.

## Code Commit

- `ea4a65ef62f915d9fe53f573361526932b5719c2` - `Centralize HNSW debug relation page reads`

## Unsafe Count

- Previous packet baseline after packet 294: `2081`
- After this checkpoint: `2076`
- Net change: `-5`

## Validation

- `git diff --check HEAD~1..HEAD`
- `cargo check --all-targets --no-default-features --features pg18,bench`
- `cargo test --lib --no-default-features --features pg18,pg_test --no-run`
- `rg -n unsafe src --count-matches`
- `rg -n unsafe hardening crates vendor -g '*.rs' --count-matches`

The cargo commands pass. The logs include the known pre-existing SPIRE unused-import
warning and Hadamard test-only dead-code warnings.

## Artifacts

- `artifacts/git-diff-check.log`
- `artifacts/cargo-check-pg18-bench.log`
- `artifacts/cargo-test-lib-pg18-pg-test-no-run.log`
- `artifacts/unsafe-count-by-file.log`
- `artifacts/non-src-unsafe-count-by-file.log`
