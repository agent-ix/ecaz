# Review Request: SRHT Padded Query-Prep Path

Commit: `3f463ad`

Scope:
- `src/bin/simd_bench.rs`
- `src/quant/prod.rs`
- `src/quant/qjl.rs`
- `src/quant/rotation.rs`

Summary:
- add direct `prepare_ip_query/{1024,1536,2048}` coverage to `simd_bench`
- add `rotation::srht_padded(input, signs)` to fuse the old `pad_input(...); srht(...)` path for
  non-power-of-two query-prep inputs
- switch the padded SRHT call sites in:
  - `ProdQuantizer::encode`
  - `ProdQuantizer::prepare_ip_query`
  - `qjl::qjl_project`
- keep exact-size behavior unchanged by falling back to the existing `srht` path when
  `input.len() == signs.len()`
- add direct equivalence coverage that `srht_padded` matches `pad_input + srht`

Matched benchmark snapshot on this machine (`10000` iterations, auto `avx2+fma`,
`warmup_iterations=256`):
- same-harness baseline before this code change:
  - `srht/d1024_td1024` `636.7 ns`
  - `srht/d1536_td2048` `1334.8 ns`
  - `srht/d2048_td2048` `1313.7 ns`
  - `prepare_ip_query/d1024_b4` `10348.8 ns`
  - `prepare_ip_query/d1536_b4` `16628.5 ns`
  - `prepare_ip_query/d2048_b4` `20790.1 ns`
- kept code, clean rerun after the change:
  - `srht/d1024_td1024` `641.0 ns`
  - `srht/d1536_td2048` `1334.0 ns`
  - `srht/d2048_td2048` `1321.9 ns`
  - `prepare_ip_query/d1024_b4` `10182.6 ns`
  - `prepare_ip_query/d1536_b4` `16323.7 ns`
  - `prepare_ip_query/d2048_b4` `20613.3 ns`
- extra rerun on the kept code to check the padded case:
  - `prepare_ip_query/d1536_b4` `16246.1 ns`

Why this slice is worth keeping:
- ADR-020 makes `1536` the current baseline, and that path always pays the `2048`-lane
  transform/query-prep cost
- the old code allocated and populated an intermediate padded vector before every padded SRHT call
- the fused helper removes that extra pad/copy step without changing the FWHT/runtime-dispatch
  structure underneath
- the measured win is small, but it is consistent on the padded `1536 -> 2048` query-prep case:
  about `1.8%` on the first matched rerun and about `2.3%` on the follow-up rerun

Validation:
- `cargo test srht -- --nocapture`
- `cargo run --bin simd_bench --release --no-default-features --features pg17 -- 10000`
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Please review:
- whether `srht_padded` is the right abstraction for keeping padded SRHT call sites explicit but
  avoiding the extra temporary padded vector
- whether the new `prepare_ip_query` harness coverage is sufficient for continued query-prep tuning
- whether this is the right level of “small but real” improvement to keep for the `1536` baseline
  path
