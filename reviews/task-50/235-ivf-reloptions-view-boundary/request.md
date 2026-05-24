---
task: 50
packet: 235
topic: ivf-reloptions-view-boundary
role: coder
status: ready-for-review
created: 2026-05-21T05:26:56-07:00
head_sha: f9bdd233c27af4b39537dd77d6cb19f06a6380cf
---

# Review Request: IVF Reloptions View Boundary

## Summary

This packet moves IVF relation option parsing behind a typed reloptions view.

Changes:

- Added `EcIvfReloptionsView`, which binds PostgreSQL's relation-owned `rd_options` pointer to the `EcIvfReloptions` layout registered by `ec_ivf_amoptions`.
- Removed the free `unsafe fn read_string_reloption` helper and made string reloption access a view method.
- Made `options::relation_options` safe, matching the local HNSW/DiskANN/SPIRE relation-options pattern.
- Removed the now-unnecessary unsafe call around IVF admin option reads.

## Safety Notes

- The raw `rd_options` pointer is still only dereferenced inside the view boundary.
- The public IVF options reader validates null relations, returns defaults for null reloptions, and borrows PostgreSQL-owned storage only for the duration of option materialization.
- The string reloption offset contract is now tied to the typed `EcIvfReloptionsView` rather than a standalone raw-pointer helper.

## Unsafe Count

- `src/am/ec_ivf/options.rs`: `10 -> 7`
- `src/am/ec_ivf/admin.rs`: `8 -> 7`
- Previous repo count: `2496`
- Current repo count: `2492`
- Delta: `-4`

The packet-local count log is:

- `artifacts/unsafe-counts.log`

## Validation

- `artifacts/rustfmt-check.log`: `rustfmt --check src/am/ec_ivf/options.rs src/am/ec_ivf/admin.rs` passed with only known stable-rustfmt config warnings.
- `artifacts/git-diff-check.log`: `git diff --check HEAD^ HEAD` passed.
- `artifacts/cargo-check-pg18-bench.log`: `cargo check --all-targets --no-default-features --features pg18,bench` passed with the known existing `src/am/mod.rs` unused SPIRE re-export warning.
- `artifacts/cargo-test-lib-ec-ivf-pg18-no-run.log`: `cargo test --lib ec_ivf --no-default-features --features pg18,pg_test --no-run` passed with the known existing Hadamard helper dead-code warnings.
