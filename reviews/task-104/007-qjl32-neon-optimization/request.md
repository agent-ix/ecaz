# Task 104 packet 007 — candidate-parallel qjl32 NEON kernel (review request)

- Task: `plan/tasks/104-apple-silicon-m5-bench-optimization-lane.md` scope item 3
- Branch: `task-104-m5-bench-optimization`
- **Code commit under review: `f88c640d3`** (src/quant/qjl32/neon.rs,
  aarch64-only). Suite-runner `retired` marker extension `d1235077c` also
  awaits review (crates/ecaz-cli, own commit per the FR-038 rule).
- Evidence: `task104-qjl32-neon-postfix-suite.json`,
  `task104-ivf-qjl-batch-cells-suite.json`, `artifacts/manifest.md`,
  suite manifests, `results.jsonl` + `results-ivf-qjl.jsonl`, install +
  build-profile logs, per-cell logs.

The packet 006 floor-gate failure root cause: the old NEON kernel scored
one candidate at a time and round-tripped every 4-lane product through
memory followed by serial scalar adds. The rewrite mirrors the AVX2 octet
design — candidates in vector lanes, dims sequential (preserving the
scalar accumulation order the 4-ulp pre-slice tolerance contract
requires), codebook gathers via a vqtbl2q byte-shuffle on a 32-byte
register pair (NEON analogue of `_mm256_permutevar8x32_ps`).

Results: kernel 667-684 -> 167-185 ns/c (~4x); floor 0.83x -> 3.2-3.5x;
IVF QJL e2e p50 -46.3%/-53.8%; recall identical to pre-fix at every cell.
Parity: `quant::qjl32` 10/10 on M5 (packet 001 covers the family unit
gates); clippy pg18 -D warnings clean. No shared/x86 code touched — Task
103 Intel cells do not require a re-run.

Review focus: (1) the accumulation-order argument for the tolerance
contract; (2) the byte-index construction for vqtbl2q ((idx<<2)*0x01010101
+ 0x03020100); (3) the scalar tail keeping per-lane dim order.
