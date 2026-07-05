# Task 28 IVF Bound-Prune Cleanup

## Scope

This packet records commit `526971ca`, which handles the small round-2
reviewer cleanups around the A7 PQ-FastScan bound-pruning path.

Changes:

- Gate the per-query `CandidateTopK` running-bound heap to quantizers that can
  actually use score-bound pruning. Today that is PQ-FastScan only.
- Add the missing invariant comment for the `heap_tid_count == 0` guard in
  `consume_live_tid_budget`.
- Remove two empty placeholder packet directories from the local tree with
  `rmdir`; they contained no files and therefore do not appear in this commit.

## Validation

- `cargo test -p ecaz --lib am::ec_ivf::scan::tests`
- `cargo test -p ecaz --lib am::ec_ivf::quantizer::tests`
- `cargo fmt`
- `git diff --check`

## Notes

This cleanup does not change PQ-FastScan scoring behavior. It removes dead
running-heap maintenance from TurboQuant and RaBitQ scans, where
`score_ip_from_parts_with_min_bound` cannot use the bound.
