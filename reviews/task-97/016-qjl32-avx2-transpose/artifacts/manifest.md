# Task 97 Packet 016 Artifact Manifest

- Head SHA: `a6b0bfd8b0d37efc3559960e18ec01e40f9bb13b`
- Task bucket: `reviews/task-97/016-qjl32-avx2-transpose/`
- Lane: coder-1 / Task 97 TurboQuant QJL block kernel
- Fixture: local x86_64 AVX2, local PG18, deterministic synthetic corpus,
  `dim=1024`, `bits=4`, `seed=42`, `queries_seed=43`
- Suite config: `reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json`
- Storage format / rerank mode: `turboquant`, production QJL (`MseLutQjl`)
- Installed backend SHA256:
  `e8dd061ef58acd1700781bf8d769589cc645f74fc42b2ff813d4be2ca27e5818`
- Timestamp: `2026-06-10T04:59:59Z`
- AWS / GitHub CI: not run

## Commands

- Format:
  `cargo fmt --check`
- Diff whitespace:
  `git diff --check`
- Focused tests:
  `cargo test qjl32_ --lib -- --nocapture --color never`
- Micro-bench:
  `cargo bench --features bench --bench quant_score 'quant/qjl32_block32' -- --sample-size 10 --warm-up-time 1 --measurement-time 2`
- Local PG18 install:
  `target/debug/ecaz --log-file reviews/task-97/016-qjl32-avx2-transpose/artifacts/local-ecaz-pg18-install.log dev install ecaz-pg-test --pg 18`
- Kernel-on suite:
  `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/016-qjl32-avx2-transpose/artifacts/suite-kernel-on-cli.log bench suite run --config reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json --artifact-dir reviews/task-97/016-qjl32-avx2-transpose/artifacts --only-tag kernel_on --manifest-output reviews/task-97/016-qjl32-avx2-transpose/artifacts/suite-kernel-on-manifest.json --results-output reviews/task-97/016-qjl32-avx2-transpose/artifacts/results-kernel-on.jsonl`
- Kernel-off suite:
  `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/016-qjl32-avx2-transpose/artifacts/suite-kernel-off-cli.log bench suite run --config reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json --artifact-dir reviews/task-97/016-qjl32-avx2-transpose/artifacts --only-tag kernel_off --manifest-output reviews/task-97/016-qjl32-avx2-transpose/artifacts/suite-kernel-off-manifest.json --results-output reviews/task-97/016-qjl32-avx2-transpose/artifacts/results-kernel-off.jsonl`

## Primary Artifacts

- `local-cargo-fmt-check.log`: passed; stable rustfmt emitted the repository's
  usual unstable-option warnings.
- `local-git-diff-check.log`: passed.
- `local-cargo-test-qjl32.log`: `10 passed; 0 failed`.
- `local-cargo-bench-qjl32-block32.log`: scalar median `35.383 us`, dispatch
  median `8.0926 us`, median speedup `4.37x`.
- `local-ecaz-pg18-install.log`: backend assertion passed; installed SHA256
  `e8dd061ef58acd1700781bf8d769589cc645f74fc42b2ff813d4be2ca27e5818`.
- `suite-kernel-on-cli.log`, `suite-kernel-on-manifest.json`,
  `results-kernel-on.jsonl`.
- `suite-kernel-off-cli.log`, `suite-kernel-off-manifest.json`,
  `results-kernel-off.jsonl`.
- Recall logs: `recall-ivf-turboquant-qjl32-batch-on.log`,
  `recall-spire-turboquant-qjl32-batch-on.log`,
  `recall-hnsw-turboquant-qjl32-batch-on.log`.
- Latency logs: `latency-ivf-turboquant-qjl32-batch-{on,off}.log`,
  `latency-spire-turboquant-qjl32-batch-{on,off}.log`,
  `latency-hnsw-turboquant-qjl32-batch-{on,off}.log`.

## Key Lines

Micro-bench:

- `quant/qjl32_block32/scalar/d1024_b4`: `[34.613 us 35.383 us 36.611 us]`
- `quant/qjl32_block32/dispatch/d1024_b4`: `[7.9057 us 8.0926 us 8.3355 us]`

IVF kernel-on direct counters:

- `nprobe=8`: `surface=ivf quant=turboquant_qjl isa=avx2 kernel_candidates=24096 kernel_elapsed_ms=6.014459`; scalar tail `scalar_candidates=1263 scalar_elapsed_ms=1.392794`.
- `nprobe=16`: `surface=ivf quant=turboquant_qjl isa=avx2 kernel_candidates=51200 kernel_elapsed_ms=12.809578`.

SPIRE direct counter comparison:

- `nprobe=8`: kernel-off scalar `21.905986 ms`; kernel-on AVX2 `3.438826 ms`; kernel-on scalar tail `12.719015 ms`; kernel-on total `16.157841 ms`; direct scoring speedup `1.36x`.
- `nprobe=16`: kernel-off scalar `44.298653 ms`; kernel-on AVX2 `7.532742 ms`; kernel-on scalar tail `24.671362 ms`; kernel-on total `32.204104 ms`; direct scoring speedup `1.38x`.

HNSW:

- `ef_search=32`: kernel-on direct row remains `isa=scalar`, `scalar_candidates=29763`, `scalar_elapsed_ms=33.124347`.

## Interpretation

The candidate-parallel AVX2 kernel fixes the packet 011 inner-loop shape and
is fast in isolation. The local suite still misses the Task 97 `1.5x`
scoring-share floor when scalar tails are included, so this packet supports
the packet 011 AVX2 stop-condition decision path rather than another broad
optimization pass.
