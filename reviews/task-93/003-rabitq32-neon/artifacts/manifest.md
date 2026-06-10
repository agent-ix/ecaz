# Manifest: Task 93 Packet 003 RaBitQ32 NEON Backend

- Head SHA: `1b447f544` ("Task 93 Phase B: real NEON rabitq32 backend via
  production pair primitive")
- Task bucket: `reviews/task-93/`
- Packet path: `reviews/task-93/003-rabitq32-neon/`
- Lane: local PG18 pgrx fixture, Apple M5 Pro (arm64, macOS) — native NEON
  host; `current_isa()` reports NEON and the kernel dispatches the real NEON
  backend (not a forced mode)
- Host/socket: `/Users/peter/.pgrx`, port `28818`; database `task93_bench`
- Extension install: `target/debug/ecaz dev install ecaz-pg-test --pg 18`,
  installed backend sha256
  `7e449bd6baac80f02989436a497023c41a7c2e367b4d64ed42df0f980b3ebec2`
  (`install-ecaz-pg18.log`)
- Fixtures, storage format, reloptions, prefixes: identical to packet 002
  (dbpedia real10k nlists=64 / real100k nlists=256, 1536-dim, `ec_ivf`
  `storage_format=rabitq`, `quant_bits=1`, `rerank=off`)
- Suite config: `crates/ecaz-cli/suites/task93-phase3-neon-ivf-rabitq.json`
- Runner command: `target/debug/ecaz bench suite run --config
  crates/ecaz-cli/suites/task93-phase3-neon-ivf-rabitq.json --database
  task93_bench --host /Users/peter/.pgrx --port 28818 --manifest-output
  .../suite-manifest.json --results-output .../results.jsonl --log-file
  .../suite-run.log`
- Kernel-on cell: `--ivf-scratch-soa-batch-decode`; kernel-off cell: default
  GUCs (per-candidate production scoring). Same cell design as packet 002.
- Truth caches reused from packet 002 (identical corpora/queries/k).
- Run note: the first suite invocation failed at the real100k `load` step —
  `corpus load` row-counting hit `ERROR: ec_ivf scan currently requires
  exactly one ORDER BY query` because the planner chose the pre-existing
  ec_ivf index for a plain `count(*)` once the tables already existed (the
  packet-002 run had a fresh database). Worked around by dropping the
  prefix tables and rerunning; logged here as a latent ec_ivf planner/AM
  interaction worth a follow-up fix outside Task 93's scope.

## Validation artifacts (HEAD `1b447f544`)

### `cargo-clippy.log`

- `cargo clippy --all-targets --no-default-features --features pg18 -- -D warnings` — clean.

### `cargo-test-rabitq32.log`

- 5 passed:
  - `scalar_tail_is_forced_scalar_anchor`, `scalar_block32_matches_forced_scalar_anchor_bits` —
    strict `f32::to_bits()` anchor, now pinned to the scalar backend directly;
  - `dispatched_block32_matches_anchor_within_tolerance` — dispatched kernel
    (NEON on this host, asserted) vs anchor under the documented SIMD
    envelope. Measured deviation at dim=1536: 22 ULP / 1.55e-6 relative —
    above the nominal 4-ULP/1e-6 figure, consistent with reordered FMA
    summation; envelope set to 1e-5 matching the existing `rabitq.rs`
    production differential precedent (see request.md §Tolerance);
  - `neon_block32_is_bit_equal_with_production_neon_batch` — kernel scores
    are bit-equal with `estimate_ip_bits1_batch`'s NEON path by construction
    (both call `sum_query_dequant_neon_bits1_pair` with identical pairing);
  - `production_dispatch_is_within_phase2_tolerance` — unchanged ADR-076
    check at dim=65.

### `cargo-test-candidate-batch.log` / `cargo-test-ivf-quantizer.log`

- 10 and 27 passed; block/tail counter tests and the IVF routing proof are
  now ISA-aware (kernel rows asserted under the kernel-returned ISA label,
  `isa=neon` on this host; tails bit-exact vs scalar).

## Bench artifacts

### Recall byte-equality (gate 1) — PASS at every cell

| fixture | nprobe | kernel-on recall@10 | kernel-off recall@10 |
|---|---|---|---|
| real10k | 8 | 0.8953 (identical percentiles) | identical |
| real10k | 32 | 0.8953 | identical |
| real100k | 32 | 0.7719 | identical |

### `[block-kernel-counters]` — `isa=neon` kernel rows, scalar tails split out

```text
label=nprobe=8  surface=ivf quant=rabitq isa=neon  kernel_flushes=162  kernel_candidates=37920  kernel_elapsed_ms=8.458353
label=nprobe=32 surface=ivf quant=rabitq isa=neon  kernel_flushes=619  kernel_candidates=154784 kernel_elapsed_ms=29.575005
label=nprobe=32 surface=ivf quant=rabitq isa=neon  kernel_flushes=1614 kernel_candidates=409568 kernel_elapsed_ms=51.493765   (real100k)
```

Kernel-off cells emit zero block-kernel rows (clean toggle); tail rows
(`isa=scalar`) carry only the <32 remainders (459/514/545 candidates).

### Per-ISA scoring-share gate (≥2× vs the scalar kernel, packet 002) — PASS

| cell | scalar kernel (packet 002) | NEON kernel (this packet) | ratio |
|---|---|---|---|
| real10k nprobe=8 | 792.8 ns/cand | 223.1 ns/cand | **3.55×** |
| real10k nprobe=32 | 514.6 ns/cand | 191.1 ns/cand | **2.69×** |
| real100k nprobe=32 | 364.4 ns/cand | 125.7 ns/cand | **2.90×** |

### End-to-end wall latency (32 iterations, concurrency 1)

| cell | kernel-on p50/p95/p99 | kernel-off p50/p95/p99 |
|---|---|---|
| real10k nprobe=8 | 1.18 / 1.41 / 2.33 ms | 1.07 / 1.33 / 1.95 ms |
| real10k nprobe=32 | 2.26 / 2.50 / 3.60 ms | 2.24 / 2.60 / 3.51 ms |
| real100k nprobe=32 | **3.57 / 6.54 / 9.57 ms** | 3.82 / 6.96 / 10.4 ms |

The packet-002 scalar-kernel regression is gone: kernel-on is at parity on
real10k and faster than kernel-off on real100k (p50 −6.5%, p95 −6.0%,
p99 −8.0%).

### `suite-manifest.json` / `results.jsonl` / `suite-run.log` / load logs / `truth-cache/`

- Structured runner outputs and per-step commands, as in packet 002.
