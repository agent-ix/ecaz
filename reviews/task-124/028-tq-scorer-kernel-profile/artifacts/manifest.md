# Task 124 Packet 028 Artifact Manifest

- head SHA: `541993ff2581e8466b89da05ec673daf24b5aaff`
- task bucket: `reviews/task-124/028-tq-scorer-kernel-profile`
- timestamp: `2026-06-30T06:47:18Z`
- lane: local macOS/aarch64 release lib test, backend `neon`
- runner: focused Cargo lib tests with `--release --features bench`
- quant/index: TurboQuant no-QJL 4-bit LUT32 scorer; no IVF/f32/nprobe
  comparison
- isolation: scorer-only synthetic fixture, 1536D, one 32-candidate block,
  200000 iterations
- purpose: establish TQ-internal scorer profiler and record rejected first
  no-QJL LUT32 kernel attempts

## Commands

```text
cargo fmt --check
```

Result: passed.

```text
cargo test --release --lib --features bench quant::lut32::tests::lut32_neon_backend_matches_scalar_reference_bits_when_available -- --nocapture
```

Result: passed.

```text
ECAZ_TQ_PROFILE_ITERS=200000 ECAZ_TQ_PROFILE_LOG=reviews/task-124/028-tq-scorer-kernel-profile/artifacts/lut32-profile-restored-baseline.log cargo test --release --lib --features bench task124_profile_lut32_block32_and_query_prep -- --ignored --nocapture
```

Result: passed.

## Artifacts

| Artifact | Meaning |
| --- | --- |
| `lut32-profile-baseline.log` | Initial profiler baseline before kernel experiments. |
| `lut32-profile-even-full-chunks.log` | First NEON even/full-chunk fast-path measurement. |
| `lut32-profile-even-full-chunks-shift-or.log` | Measurement after adding shift/OR byte-index replication; rejected regression. |
| `lut32-profile-restored-baseline.log` | Final profiler run after reverting rejected kernel changes; current code state. |
| `simd-bench-baseline-existing.log` | Pre-existing standalone `simd_bench` per-candidate context; not the Task 124 block-kernel surface. |

Additional profiler logs from intermediate reruns are present in the packet
artifact directory and show the same high variance; they are not primary
decision artifacts.

## Key Lines

Initial baseline:

```text
task124_lut32_profile backend=neon dim=1536 iterations=200000
score_ip_lut_no_qjl_4bit_block32 isa=neon total=1.493478125s candidates=6400000 ns_per_candidate=233.4
prepare_ip_query_lut_no_qjl_4bit iterations=50000 total=172.192625ms ns_per_iter=3443.9
```

Even/full-chunk fast path:

```text
task124_lut32_profile backend=neon dim=1536 iterations=200000
score_ip_lut_no_qjl_4bit_block32 isa=neon total=1.486012167s candidates=6400000 ns_per_candidate=232.2
prepare_ip_query_lut_no_qjl_4bit iterations=50000 total=174.533ms ns_per_iter=3490.7
```

Shift/OR byte-index rewrite:

```text
task124_lut32_profile backend=neon dim=1536 iterations=200000
score_ip_lut_no_qjl_4bit_block32 isa=neon total=2.57771075s candidates=6400000 ns_per_candidate=402.8
prepare_ip_query_lut_no_qjl_4bit iterations=50000 total=173.225667ms ns_per_iter=3464.5
```

Restored baseline / current code state:

```text
task124_lut32_profile backend=neon dim=1536 iterations=200000
score_ip_lut_no_qjl_4bit_block32 isa=neon total=2.059896708s candidates=6400000 ns_per_candidate=321.9
prepare_ip_query_lut_no_qjl_4bit iterations=50000 total=250.159458ms ns_per_iter=5003.2
```

## Interpretation

The profiler gives Task 124 a direct scorer-internal metric. The two attempted
kernel changes do not land:

- the even/full-chunk fast path's apparent `233.4 -> 232.2 ns/candidate` delta is
  too small relative to observed run variance;
- the shift/OR byte-index rewrite regressed the scorer to `402.8 ns/candidate`.

Further Task 124 work should use this profiler to continue scorer-kernel,
query-prep, and batch/flush-width optimization with direct TQ-internal deltas.
