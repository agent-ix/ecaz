# Review Request: AVX2 FWHT Post-1024 Retune Failures

Base state:
- branch head before and after these experiments: `b3cc8d4`
- last landed FWHT code checkpoint still under evaluation: `a04592f`

Scope:
- temporary experiments in `src/quant/hadamard.rs` only
- no code changes kept

Summary:
- after landing the exact-size `1024` checkpoint in `review/10112`, tried two follow-on AVX2 FWHT
  retunes
- both experiments preserved scalar behavior, NEON behavior, and runtime dispatch boundaries during
  testing
- both were reverted before commit because they regressed the padded-query-prep sizes that matter
  more than the small local win they showed elsewhere

Current AVX2 FWHT baseline on this machine before the temporary retunes (same harness at `20000`
iterations, auto `avx2+fma`, `warmup_iterations=256`):
- baseline (`b3cc8d4`, validated `10112` state):
  `fwht/1024` `489.8 ns`, `fwht/2048` `1054.2 ns`, `fwht/4096` `2456.7 ns`

Experiment log:
- tried and reverted: explicit exact-size chunked stage schedules for `1024`, `2048`, and `4096`
  - shape:
    - bootstrap `256`-lane chunks first
    - then apply one explicit stage width inside `512`-lane chunks
    - then one explicit stage width inside `1024`-lane chunks
    - then continue with the next full-array stage
  - temporary result:
    - `fwht/1024` `481.8 ns`
    - `fwht/2048` `1071.7 ns`
    - `fwht/4096` `2632.9 ns`
  - interpretation:
    - slight `1024` win was not worth the `2048` regression and especially not the large `4096`
      regression
    - explicitly chunking later stages appears to over-localize the work and gives back too much at
      larger exact sizes on this host

- tried and reverted: reopen exact-size `2048` / `4096` tuning with `inner_tile_width = 512`
  instead of `256`
  - shape:
    - keep `1024 -> (512, 256)` from `review/10112`
    - change `2048 -> (1024, 512)`
    - change `4096 -> (2048, 512)`
  - temporary result:
    - `fwht/1024` `514.4 ns`
    - `fwht/2048` `1127.1 ns`
    - `fwht/4096` `2530.6 ns`
  - interpretation:
    - this was a clean regression across all three measured FWHT sizes
    - the earlier `inner_tile_width = 256` choice remains the better exact-size boundary on this
      machine even after the faster stage-combiner work in `review/10111`

Outcome:
- both experiments were reverted before commit
- no new code checkpoint was created from either retune
- current working interpretation:
  - the kept `10111` stage-combiner change and `10112` exact-size `1024` specialization already
    captured the easy follow-on wins from the current AVX2 FWHT structure
  - further gains likely need a different layer of change than more post-`256` exact-size schedule
    retuning

Validation:
- benchmark-only decision; no landed code changes
- focused temporary validation before each revert:
  - `cargo test fwht -- --nocapture`

Please review:
- whether this packet records the two discarded post-`10112` FWHT retunes clearly enough to avoid
  retracing them later
- whether reverting immediately on the `2048` / `4096` regressions was the right call
- whether the current interpretation is reasonable that the next FWHT win probably lies outside
  more exact-size tile-width retuning
