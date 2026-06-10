# Task 97 Packet 018 Artifact Manifest

- Head SHA: `fa501976ca16925d4eac101af5c0662a9aca971b`
- Task bucket: `reviews/task-97/018-qjl32-octet-batch/`
- Lane: coder-1 / Task 97 TurboQuant QJL block kernel
- Fixture: local x86_64 AVX2, local PG18, deterministic synthetic corpus,
  `dim=1024`, `bits=4`, `seed=42`, `queries_seed=43`
- Suite config: `reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json`
- Storage format / rerank mode: `turboquant`, production QJL (`MseLutQjl`)
- Installed backend SHA256:
  `c47b05a8654ad59ca8db0f0e0bb01af9cf09acf678ddb0a2928d44c36a4b04eb`
- Timestamp: `2026-06-09T23:55:00-07:00`
- AWS / GitHub CI: not run

## Commands

- Format:
  `cargo fmt --check`
- Diff whitespace:
  `git diff --check`
- Focused qjl32 tests:
  `cargo test qjl32_ --lib -- --nocapture --color never`
- HNSW under-octet bypass test:
  `cargo test turboquant_exact_payload_flush_bypasses_qjl_batch_under_octet --lib -- --nocapture --color never`
- CandidateBatch counter test:
  `cargo test turboquant_qjl_batch_matches_pre_slice_scalar_reference_and_records_counters --lib -- --nocapture --color never`
- Local PG18 install:
  `target/debug/ecaz --log-file reviews/task-97/018-qjl32-octet-batch/artifacts/local-ecaz-pg18-install-after-bypass.log dev install ecaz-pg-test --pg 18`
- Kernel-on suite:
  `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/018-qjl32-octet-batch/artifacts/suite-kernel-on-cli-after-bypass.log bench suite run --config reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json --artifact-dir reviews/task-97/018-qjl32-octet-batch/artifacts --only-tag kernel_on --manifest-output reviews/task-97/018-qjl32-octet-batch/artifacts/suite-kernel-on-manifest-after-bypass.json --results-output reviews/task-97/018-qjl32-octet-batch/artifacts/results-kernel-on-after-bypass.jsonl`
- Kernel-off suite:
  `target/debug/ecaz --database postgres --host /home/peter/.pgrx --port 28818 --log-file reviews/task-97/018-qjl32-octet-batch/artifacts/suite-kernel-off-cli-after-bypass.log bench suite run --config reviews/task-97/009-local-qjl32-suite/artifacts/task97-local-qjl32-suite.json --artifact-dir reviews/task-97/018-qjl32-octet-batch/artifacts --only-tag kernel_off --manifest-output reviews/task-97/018-qjl32-octet-batch/artifacts/suite-kernel-off-manifest-after-bypass.json --results-output reviews/task-97/018-qjl32-octet-batch/artifacts/results-kernel-off-after-bypass.jsonl`

## Primary Artifacts

- `cargo-fmt-check-after-bypass.log`: passed; stable rustfmt emitted the
  repository's usual unstable-option warnings.
- `git-diff-check-after-bypass.log`: passed.
- `cargo-test-qjl32-after-hnsw-bypass.log`: `11 passed; 0 failed`.
- `cargo-test-hnsw-under-octet-bypass.log`: `1 passed; 0 failed`.
- `cargo-test-qjl32-candidate-batch-after-bypass.log`: `1 passed; 0 failed`.
- `git-diff-check-packet.log`: passed after packet files were written.
- `local-ecaz-pg18-install-after-bypass.log`: backend assertion passed;
  installed SHA256
  `c47b05a8654ad59ca8db0f0e0bb01af9cf09acf678ddb0a2928d44c36a4b04eb`.
- `suite-kernel-on-cli-after-bypass.log`,
  `suite-kernel-on-manifest-after-bypass.json`,
  `results-kernel-on-after-bypass.jsonl`.
- `suite-kernel-off-cli-after-bypass.log`,
  `suite-kernel-off-manifest-after-bypass.json`,
  `results-kernel-off-after-bypass.jsonl`.
- Recall logs: `recall-ivf-turboquant-qjl32-batch-on.log`,
  `recall-spire-turboquant-qjl32-batch-on.log`,
  `recall-hnsw-turboquant-qjl32-batch-on.log`.
- Latency logs: `latency-ivf-turboquant-qjl32-batch-{on,off}.log`,
  `latency-spire-turboquant-qjl32-batch-{on,off}.log`,
  `latency-hnsw-turboquant-qjl32-batch-{on,off}.log`.

## Key Lines

IVF kernel-on direct counters:

- `nprobe=8`: `surface=ivf quant=turboquant_qjl isa=avx2 kernel_candidates=25080 kernel_elapsed_ms=6.157508`; scalar tail `scalar_candidates=279 scalar_elapsed_ms=0.330106`.
- `nprobe=16`: `surface=ivf quant=turboquant_qjl isa=avx2 kernel_candidates=51200 kernel_elapsed_ms=12.712645`.

SPIRE direct scoring comparison:

- `nprobe=8`: kernel-off scalar `22.422690 ms`; kernel-on AVX2 `5.691617 ms`; kernel-on scalar tail `3.220812 ms`; kernel-on total `8.912429 ms`; direct scoring speedup `2.52x`.
- `nprobe=16`: kernel-off scalar `45.124169 ms`; kernel-on AVX2 `11.419457 ms`; kernel-on scalar tail `6.426059 ms`; kernel-on total `17.845516 ms`; direct scoring speedup `2.53x`.

HNSW:

- End-to-end latency: kernel-off `1.73 ms`; kernel-on `1.70 ms`; speedup `1.02x`.
- Direct counters: `surface=hnsw quant=turboquant_qjl isa=avx2 kernel_candidates=3824 kernel_elapsed_ms=0.950083`; scalar batch row `scalar_candidates=623 scalar_elapsed_ms=0.697507`.
- Under-octet scalar bypass reduced HNSW CandidateBatch-accounted candidates from packet 016's `29763` scalar-only candidates to `4447` total candidates while leaving 1-7 candidate groups on the pre-batch scalar scorer.

End-to-end latency ratios:

- IVF `nprobe=8`: off `1.19 ms`, on `1.03 ms`, speedup `1.16x`.
- IVF `nprobe=16`: off `1.51 ms`, on `1.20 ms`, speedup `1.26x`.
- SPIRE `nprobe=8`: off `8.91 ms`, on `8.65 ms`, speedup `1.03x`.
- SPIRE `nprobe=16`: off `17.4 ms`, on `16.8 ms`, speedup `1.04x`.
- HNSW `ef_search=32`: off `1.73 ms`, on `1.70 ms`, speedup `1.02x`.

## Interpretation

This packet closes the packet 016 follow-up without broad optimization. The
qjl32 batch route still uses 32-candidate block kernels first, but AVX2 now
scores any full 8-candidate remainder before scalar fallback. Octet-scored
candidates are recorded as kernel rows under `isa=avx2`; true tails remain
scalar rows. HNSW additionally bypasses qjl32 CandidateBatch for groups below
8 candidates, because those groups cannot use the octet kernel and were the
remaining local parity regression.
