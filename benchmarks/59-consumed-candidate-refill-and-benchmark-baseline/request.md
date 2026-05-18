# Review: Consumed-Candidate Refill and Benchmark Baseline

Commit: `60cfbfa`

Scope:
- `src/am/scan.rs`
- `src/lib.rs`
- `tests/proptest_quant.rs`
- `benches/criterion/quant_score.rs`
- `benches/iai/quant_score.rs`

Summary:
- The bootstrap frontier refill path now expands from the consumed candidate's adjacency instead of always reusing the entry adjacency.
- The pg regression was tightened to the actual contract: if the consumed candidate exposes an unseen neighbor, refill restores frontier width; otherwise the frontier simply shrinks by the consumed slot.
- The branch's new benchmark/integration baseline now relies on the narrow always-available `bench_api` surface in `src/lib.rs`.
- Added first-pass benchmark coverage for hot-path scoring and decode operations:
  - criterion: `score_ip_from_parts`, `score_ip_encoded_lite`, `decode_approximate`
  - iai-callgrind: `score_ip_from_parts`
- Added regression coverage for:
  - SRHT roundtrip on real-world padded dimensions (`384`, `768`, `1024`, `1536`)
  - `score_code_inner_product` equivalence with `ProdQuantizer::score_ip_codes_lite`

Validation:
- `cargo test`
- `PGRX_HOME=/tmp/tqvector_pgrx_home cargo pgrx test pg17`
- `cargo clippy --all-targets --no-default-features --features pg17 -- -D warnings`

Review focus:
- Scan correctness: does the consumed-candidate refill contract match the current traversal groundwork, and are there remaining edge cases around frontier compaction or visited-set interaction?
- Benchmark surface: is the benchmark/integration API still narrow enough, and are these first added hot-path benches measuring the right entry points without hiding extra setup costs?
- Coverage quality: are the new SRHT real-world-dimension property and wrapper-equivalence test the right low-risk additions, or is there a more useful next benchmark/test slice before broader recall-data realism work?
