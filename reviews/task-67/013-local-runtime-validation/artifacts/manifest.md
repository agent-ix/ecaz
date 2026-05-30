# Task 67 Packet 013 Artifact Manifest

- Head SHA: `903ee4d0212a5517584b5d0c90a7b1b11f37e663`
- Task bucket: `reviews/task-67/`
- Packet path: `reviews/task-67/013-local-runtime-validation/`
- Timestamp: `2026-05-30T02:22:05Z`
- Lane: local runtime validation attempt for Task 67 x86 scaffold
- Fixture: focused Rust unit test binary startup
- Storage format: not applicable
- Rerank mode: not applicable
- Surface isolation: not applicable; no benchmark or SQL surface was run

## Artifacts

### `runtime-attempt.log`

- Command: `cargo test -p ecaz task67_sum_query_dequant_for_test_scaffold_matches_scalar_when_available -- --nocapture`
- Result: failed before test body execution.
- Key line:
  - `undefined symbol: pg_re_throw`

### `cpu-features.log`

- Command: `lscpu`
- Result: host is Intel x86_64 with AVX2+FMA and no AVX-512 flags.
- Key line:
  - `Model name: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz`
  - `Flags: ... fma ... avx2 ...`

## Limitations

- This packet is evidence of local runtime and hardware limitations, not a
  successful validation packet.
- Full Task 67 completion still requires an Intel validation lane where the
  PostgreSQL-linked tests can start and where AVX-512 runtime paths can execute.
