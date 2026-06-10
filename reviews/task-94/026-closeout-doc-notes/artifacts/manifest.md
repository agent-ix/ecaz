# Task 94 Packet 026 Artifact Manifest

- Head SHA: `53ebf0ca1d7f97cde6d8c3212cefc62adf3a3cf8`
- Task bucket: `reviews/task-94/026-closeout-doc-notes/`
- Lane: coder-1 / Task 94 grouped-PQ block kernel
- Fixture: not applicable; closeout-scope documentation/help-text cleanup
- Storage format / quant: IVF PqFastScan / grouped-PQ
- Timestamp: `2026-06-10T00:40:00-07:00`
- AWS / GitHub CI: not run

## Commands

- Format:
  `cargo fmt --check`
- Diff whitespace:
  `git diff --check`

## Primary Artifacts

- `cargo-fmt-check.log`: passed; stable rustfmt emitted the repository's usual
  unstable-option warnings.
- `git-diff-check.log`: passed.

## Key Lines

Changed files:

- `docs/usage.md`
- `plan/tasks/94-grouped-pq-block-kernel-family.md`
- `plan/tasks/README.md`
- `src/am/ec_ivf/options.rs`

The change documents the packet 024 F1/F2 closeout-scope items:

- IVF PqFastScan batch scoring bypasses legacy per-posting `suffix_max`
  pruning, so `posting_pruned_by_bound` is expected to read 0 for postings
  scored by the batch path.
- The IVF PqFastScan grouped-PQ block-kernel path remains opt-in behind
  `ec_ivf.scratch_soa_batch_decode`; Task 94 does not flip the default without
  final Graviton 4 / full closeout evidence.
