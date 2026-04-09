# Review Request: Prepared Query Prefix Copies

Commit: `aaf552d`

Scope:
- `src/quant/prod.rs`

Summary:
- keep the existing `srht_padded` query-prep path from `10115`, but stop copying the SRHT outputs
  again when building `PreparedQuery`
- `prepare_ip_query` already owns both full SRHT result vectors:
  - query rotation via `rotation::srht_padded`
  - QJL projection via `qjl::qjl_project`
- instead of allocating fresh prefix vectors with `[..self.original_dim].to_vec()`, truncate those
  owned vectors in place and return them directly

Matched benchmark on this machine (`40000` iterations, auto `avx2+fma`,
`warmup_iterations=256`):
- reverted-copy baseline:
  - `prepare_ip_query/d1024_b4` `10350.2 ns`
  - `prepare_ip_query/d1536_b4` `16266.9 ns`
  - `prepare_ip_query/d2048_b4` `20756.0 ns`
- kept code:
  - `prepare_ip_query/d1024_b4` `10169.7 ns`
  - `prepare_ip_query/d1536_b4` `15632.5 ns`
  - `prepare_ip_query/d2048_b4` `19911.4 ns`

Observed deltas from the matched `40000`-iteration runs:
- `1024`: about `1.7%` faster
- `1536`: about `3.9%` faster
- `2048`: about `4.1%` faster

Why this slice is worth keeping:
- the old code paid two extra allocations plus two full-vector copies on every `prepare_ip_query`
  call, even when `original_dim == transform_dim`
- this is a narrow ownership cleanup with no algorithm change and no new SIMD/runtime-dispatch
  surface
- the win shows up at all three query-prep operating points now exposed in `simd_bench`

Validation:
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 40000`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether truncating the owned SRHT result vectors in place is the right ownership boundary for
  `PreparedQuery`
- whether the matched `40000`-iteration benchmark evidence is strong enough for keeping this as a
  query-prep checkpoint
