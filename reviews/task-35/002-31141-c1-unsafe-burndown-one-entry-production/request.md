# Review Request: Unsafe Burndown One-Entry Production Slice

Head: `f6a780887a264e31c17151e63810b27e0aa6c47d`

Scope:
- `scripts/unsafe_comment_baseline.txt`
- `src/am/ec_ivf/quantizer.rs`
- `src/am/ec_spire/coordinator/remote_candidates/resolve.rs`
- `src/am/ec_spire/coordinator/remote_candidates/result_contracts.rs`
- `src/am/ec_spire/scan/types.rs`
- `src/am/ec_spire/update/materialization.rs`
- `src/am/ec_spire/update/routing.rs`
- `src/standalone_pg_backend_stubs.rs`

What changed:
- Reviewed and documented seven one-entry unsafe-comment baseline sites.
- Removed those seven entries from `scripts/unsafe_comment_baseline.txt`.
- Covered live relation/snapshot/slot invariants, backend transaction XID
  access, diagnostic summary relation lifetime, IVF codebook chain TID
  validity, and the standalone C-string panic stub.

Baseline result:
- Start: 4,816 entries across 124 files.
- End: 4,809 entries across 117 files.
- Net reduction: 7 entries and 7 files.

Review focus:
- Whether each new `SAFETY` comment states a real caller/callee invariant
  rather than restating the unsafe operation.
- Whether this small slice is an acceptable first proof of the Task 35 packet
  workflow before taking larger subsystem batches.

Validation:
- `bash scripts/unsafe_baseline_report.sh /private/tmp/tqvector-unsafe-baseline-before.txt`
  - artifact: `artifacts/unsafe-baseline-before.log`
- `bash scripts/unsafe_baseline_report.sh`
  - artifact: `artifacts/unsafe-baseline-after.log`
- `bash scripts/check_unsafe_comments.sh`
  - artifact: `artifacts/audit-unsafe.log`
- `make fmt-check`
  - artifact: `artifacts/fmt-check.log`
- `git diff --check HEAD^ HEAD`
  - artifact: `artifacts/git-diff-check.log`
- `cargo check --all-targets --no-default-features --features pg18,bench`
  - artifact: `artifacts/cargo-check-pg18.log`

Notes:
- `cargo check` passed with existing warnings from PostgreSQL headers and
  currently unused SPIRE re-exports in `src/am/mod.rs`.
