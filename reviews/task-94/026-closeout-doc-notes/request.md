# Task 94 Packet 026: Closeout Doc Notes

This packet handles the local closeout-scope documentation items from packet
024 feedback F1/F2. It does not change block-kernel behavior, benchmark
defaults, or runtime routing defaults.

Code checkpoint: `53ebf0ca1d7f97cde6d8c3212cefc62adf3a3cf8`

## Change

- `docs/usage.md` now documents that IVF PqFastScan grouped-PQ block-kernel
  scoring is opt-in through `ec_ivf.scratch_soa_batch_decode = on`.
- The same section explains the pruning-vs-throughput trade: with batch
  scoring enabled, eligible PqFastScan postings are scored from scratch SoA
  batches and do not use the per-posting `suffix_max` cutoff, so
  `posting_pruned_by_bound` is expected to read 0 for those postings.
- The `ec_ivf.scratch_soa_batch_decode` GUC long description now names the
  Task 94 IVF PqFastScan grouped-PQ block-kernel path and retains
  default-off wording.
- Task 94 status text now points through this packet and keeps final Graviton
  4 / full closeout evidence pending approval.

## Validation

- `cargo fmt --check`
- `git diff --check`

No tests, benchmarks, GitHub CI, or AWS runs were used because this is a
documentation/help-text cleanup.

## Request

Please review this as closing the packet 024 F1/F2 closeout documentation
items. Task 94 should still remain in review pending the approved Graviton 4
lane and final closeout benches.
