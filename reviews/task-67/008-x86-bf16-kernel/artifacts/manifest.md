# Artifact Manifest

- head SHA: `f909b16ded2b276a10c952aab1516e00c1759b56`
- task bucket: `reviews/task-67/008-x86-bf16-kernel`
- lane: Task 67 x86 AVX-512 BF16 bits=4 scoring kernel
- fixture: none
- storage format: RaBitQ bits=4 scoring path with `rabitq-bf16`
- rerank mode: not applicable
- isolated one-index-per-table or shared-table surfaces: not applicable
- timestamp: `2026-05-30T01:50:36Z`

## Artifacts

- `validation.log`
  - command: `cargo fmt`
  - command: `cargo test -p ecaz quant::rabitq::tests::x86_sum_query_dequant_bits4_matches_scalar_when_available --no-run`
  - command: `rustc /tmp/check_exact_bf16.rs`
  - attempted command: `cargo test -p ecaz --features rabitq-bf16 quant::rabitq::tests::x86_sum_query_dequant_bf16_bits4_matches_bf16_scalar_when_available --no-run`
  - attempted command: `cargo check -p ecaz --features rabitq-bf16 --lib`
  - key lines:
    - default focused bits=4 test build exited 0 and produced test executables
    - standalone BF16 intrinsic probe exited 0
    - feature-enabled cargo validation did not complete because cargo spun without spawning `rustc`
    - runtime execution remains locally blocked by unresolved PostgreSQL symbol `LockBuffer`
