# Review Request: Prepared Query LUT Row Fill Deferred

Commit context:
- attempted code checkpoint: `4c47b26`
- reverted on active branch by: `c5810ac`

Scope explored:
- `src/quant/prod.rs`

What was tried:
- replace `prepare_ip_query`'s nested `Vec::push` LUT builder with a pre-sized row-fill helper
- keep the LUT layout unchanged
- specialize the hot `4-bit` path with a fixed-width `8`-centroid row fill

Why it is not kept in the current baseline:
- the measured win was large, but this changes a central query-prep construction path enough that
  it should be isolated and reviewed on its own instead of riding along with the current narrower
  scoring/query-prep cleanup line
- current direction is to defer it, not to discard the result

Measured on this machine (`40000` iterations, auto `avx2+fma`, `warmup_iterations=256`):
- baseline from the prior kept checkpoint (`10116`):
  - `prepare_ip_query/d1024_b4` `10169.7 ns`
  - `prepare_ip_query/d1536_b4` `15632.5 ns`
  - `prepare_ip_query/d2048_b4` `19911.4 ns`
- LUT-row-fill experiment, first long run:
  - `prepare_ip_query/d1024_b4` `3772.7 ns`
  - `prepare_ip_query/d1536_b4` `6088.8 ns`
  - `prepare_ip_query/d2048_b4` `7781.2 ns`
- LUT-row-fill experiment, confirmatory hot rerun:
  - `prepare_ip_query/d1024_b4` `3884.2 ns`
  - `prepare_ip_query/d1536_b4` `6235.3 ns`
  - `prepare_ip_query/d2048_b4` `7680.2 ns`

Observed deltas from the confirmatory rerun versus the `10116` baseline:
- `1024`: about `61.8%` faster
- `1536`: about `60.1%` faster
- `2048`: about `61.4%` faster

Interpretation:
- this does look promising
- the reason it is deferred is scope/isolation, not bad numbers
- if revived later, it should come back as its own explicitly-reviewed slice with focused before/after
  evidence and no other concurrent query-prep changes

Current branch state after deferral:
- the active code path is back to the `10116` baseline
- the experiment remains documented here so it is not lost or retried blindly

Validation after the revert back to the active baseline:
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether deferring this larger LUT-construction rewrite is the right call for the current branch
- whether the packet preserves enough evidence to bring the idea back later without rediscovery
