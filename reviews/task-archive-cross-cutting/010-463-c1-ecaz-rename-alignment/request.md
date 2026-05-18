# Review Request: Ecaz Rename Alignment On Main

Current head: `00441d1`

Scope:
- `spec/adr/ADR-036-opq-rotation-successor-to-srht.md`
- `spec/adr/ADR-037-additive-residual-quantization.md`
- `spec/adr/ADR-038-lsq-codebook-refinement.md`
- `spec/adr/ADR-045-symphonyqg-quantized-graph-access-method.md`
- `spec/adr/ADR-046-gpu-accelerated-offline-build-trainer.md`

Problem:
- `main` already carried the Ecaz rename checkpoint in `56cd6ce`, but this checkout was still
  sitting on `pg18-shared-infra-merge` because `main` had been left bound to a stray worktree.
- The old branch also still carried a newer ADR wording pass for the OPQ / AQ / LSQ / SymphonyQG /
  GPU-trainer ADRs.
- That branch-only delta needed to be preserved onto `main` without undoing the Ecaz rename or
  leaving the work product stranded on the old branch.

What changed:
- Removed the stray `/home/peter/dev/tqvector-main-merge` worktree binding so `main` could be
  checked out directly in `/home/peter/dev/tqvector`.
- Verified the old branch-only commit (`7980c10`) and compared it against `main`.
- Brought the remaining ADR wording delta onto `main` directly:
  - updated the OPQ, AQ, and LSQ ADR GPU-speedup wording to the newer branch text
  - aligned the SymphonyQG ADR wording with the newer branch text for reported speedups,
    `alpha`-pruning notation, and latency units
  - aligned the GPU trainer ADR wording with the newer branch text for reported speedups and
    latency units
  - preserved the Ecaz rename on `main` where the old branch still said `tqvector`
- Left the already-merged Ecaz rename checkpoint itself untouched; this follow-up only aligns the
  remaining ADR text on top of that rename.

Validation:
- Passed:
  - `cargo test`
  - `bash scripts/run_pgrx_pg17_test.sh`
  - `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Review focus:
- Whether the remaining ADR wording now matches the intended post-rename `main` state
- Whether any stale pre-rename `tqvector` identity wording should still be updated in these ADRs
- Whether this packet stayed strictly doc/ADR scope and did not change runtime behavior
