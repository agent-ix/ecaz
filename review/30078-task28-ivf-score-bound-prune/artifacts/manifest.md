# Artifacts Manifest

Packet: `30078-task28-ivf-score-bound-prune`

Baseline head SHA after backing out trial code: `591c10adf5eb2b859d5da77274d9bc3a9ea074bd`

Timestamp: `2026-04-27T19:20:48-07:00`

Lane: Task 28 A7 negative score-bound prune trial.

Fixture: local PG18, existing 10k DBPedia-derived `task28_ivf_postopt10k_n64w25` surface.

Storage format: IVF TurboQuant no-QJL 4-bit LUT lane.

Rerank mode: heap-f32, width 25.

Isolation: existing isolated one-index-per-table IVF surface.

Important: the latency artifacts were collected from uncommitted trial builds that were backed out. This packet records negative evidence only; it is not evidence for landed code.

## `latency_10k_n64w25_nprobe32_48.log`

- Trial: per-dimension suffix bound check.
- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --log-output review/30078-task28-ivf-score-bound-prune/artifacts/latency_10k_n64w25_nprobe32_48.log`
- Key lines:
  - `32 ... p50 71.6 ms ... p95 82.5 ms`
  - `48 ... p50 95.0 ms ... p95 102.7 ms`

## `latency_10k_n64w25_nprobe32_48_coarse.log`

- Trial: coarse suffix bound check.
- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --log-output review/30078-task28-ivf-score-bound-prune/artifacts/latency_10k_n64w25_nprobe32_48_coarse.log`
- Key lines:
  - `32 ... p50 110.1 ms ... p95 118.3 ms`
  - `48 ... p50 159.2 ms ... p95 169.5 ms`

## `latency_10k_n64w25_nprobe32_48_byte_lut.log`

- Trial: byte-LUT scorer plus byte-level suffix bound.
- Command: `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database postgres bench latency --prefix task28_ivf_postopt10k_n64w25 --profile ec_ivf --k 10 --iterations 100 --sweep 32,48 --log-output review/30078-task28-ivf-score-bound-prune/artifacts/latency_10k_n64w25_nprobe32_48_byte_lut.log`
- Key lines:
  - `32 ... p50 101.1 ms ... p95 107.7 ms`
  - `48 ... p50 142.0 ms ... p95 150.3 ms`

## `simd_bench_byte_lut.log`

- Trial: byte-LUT scorer isolated release kernel.
- Command: `cargo run --release --bin simd_bench -- --iterations 20000 --log-output review/30078-task28-ivf-score-bound-prune/artifacts/simd_bench_byte_lut.log`
- Key lines:
  - `backend=avx2+fma`
  - `score_ip_lut_no_qjl_4bit/d1536: total=21.662986ms ns_per_iter=1083.1`
