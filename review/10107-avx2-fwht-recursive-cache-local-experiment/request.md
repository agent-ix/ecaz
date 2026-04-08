# Review Request: AVX2 FWHT Recursive Cache-Local Experiment

Base state:
- branch head before and after the experiment: `2bff69b`
- last landed FWHT code checkpoint still under evaluation: `d276990`

Scope:
- temporary experiment in `src/quant/hadamard.rs` only
- no code changes kept

Summary:
- tried replacing the remaining AVX2 full-array outer-stage passes with a recursive split/combine
  structure
- reused the existing 8/16/32/64-lane AVX2 bootstrap helpers at the leaves
- kept scalar behavior, NEON behavior, and runtime dispatch boundaries unchanged during the
  experiment
- reverted the experiment before commit because it regressed FWHT throughput on this host

Before/after AVX2 FWHT snapshot on this machine (same harness at `20000` iterations, auto
`avx2+fma`, `warmup_iterations=256`):
- before (`2bff69b`, current branch baseline):
  `fwht/2048` `1167.9 ns`, `fwht/4096` `2611.1 ns`
- temporary recursive experiment (reverted before commit):
  `fwht/2048` `1248.0 ns`, `fwht/4096` `2832.6 ns`

Experiment log:
- tried: recurse until tile size `<= 64`, use the existing AVX2 bootstrap blocks as leaves, then
  combine transformed halves with the existing AVX2 butterfly loop over one half-width
- focused correctness check during the experiment: `cargo test fwht -- --nocapture` passed,
  including a temporary direct x86_64 test for sizes above `64`
- observed on this host: the recursive shape was about `6.9%` slower at `2048` and about `8.5%`
  slower at `4096` versus the current 64-lane-bootstrap implementation
- outcome: reverted before commit; no checkpoint, no full validation sweep, and no review packet
  tied to landed code was created for the recursive variant itself
- current interpretation: a naive recursive split/combine shape does not automatically beat the
  existing iterative outer-stage loop here, even when it reuses the 64-lane AVX2 leaf helpers
- likely next direction: try a more explicit tiled iterative structure that fuses multiple outer
  stages inside a bounded working set, rather than pure recursion over the whole transform tree

Validation:
- benchmark-only decision; no landed code changes
- focused temporary validation before revert:
  - `cargo test fwht -- --nocapture`

Please review:
- whether this negative-result packet records the failed recursive experiment clearly enough for
  later B1 FWHT work
- whether reverting immediately on this measured regression was the right call
- whether the suggested next step should be iterative stage fusion / tiling rather than another
  recursive variant
