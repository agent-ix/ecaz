# Task 51 Local IVF Adaptive Nprobe Ratio Follow-Up

- head SHA: `7e215f5edf9bc4e8dd906bc2d36f861ae9f00b61`
- code commit: `7e215f5edf9bc4e8dd906bc2d36f861ae9f00b61` - adds IVF-only adaptive nprobe score margin-ratio signal and recall `recall_worst`
- timestamp: `2026-05-23T16:22:17Z`
- benchmark packet: `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/`
- runner: `ecaz bench suite`
- SuiteConfig: `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/suite.json`
- suite manifest: `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-manifest.json`
- results: `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/results.jsonl`
- lane: local PG18 / WSL2 only
- AWS: not used
- competitors: none; IVF/RaBitQ only
- table surface: reused preserved isolated prefix `task51_local_990k_ivf_rabitq1_n1024_w50`
- corpus: staged anchor corpus, 990000 rows, 10000 query rows, dim 1536
- profile: `ec_ivf`
- storage format: `rabitq`
- reloptions: `nlists=1024`, `nprobe=256`, `training_sample_rows=10000`, `quant_bits=1`, `rerank=heap_f32`, `rerank_width=50`, `storage_format=rabitq`
- adaptive policy: opt-in `ec_ivf.adaptive_nprobe`; score margin-ratio thresholds `2500`, `10000`, `50000` basis points
- rerank mode: heap f32 rerank width 50
- recall query limit: 100 local smoke waiver
- latency iterations: 100, concurrency 1
- isolated one-index-per-table surface: yes, inherited from packet `benchmarks/task51-local-ivf-rabitq-990k/`
- vchord / pgvectorscale: not run

## Commands

Release local extension install before performance run:

```text
script -q -e -c "cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config" benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/local-pgrx-install-release.log
```

GUC verification:

```text
target/debug/ecaz --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db tqvector_bench --socket-dir /home/peter/.pgrx --raw --sql "LOAD 'ecaz'; SHOW ec_ivf.adaptive_nprobe_score_margin_ratio_bps;" --log-output benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/guc-ratio-show.log
```

CLI build and suite execution:

```text
script -q -e -c "cargo build -p ecaz-cli" benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/cargo-build-ecaz-cli.log
target/debug/ecaz --log-file benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-ivf-adaptive-nprobe-ratio/suite.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-dry-run.log bench suite run --config benchmarks/task51-local-ivf-adaptive-nprobe-ratio/suite.json --dry-run --manifest-output benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-manifest.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-run.log bench suite run --config benchmarks/task51-local-ivf-adaptive-nprobe-ratio/suite.json --manifest-output benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-adaptive-nprobe-ratio/artifacts/results-report.jsonl
```

Focused tests before suite:

```text
cargo test -p ecaz-cli --no-default-features adaptive_nprobe
cargo test -p ecaz-cli --no-default-features recall_summary
cargo test -p ecaz-cli --no-default-features expands_recall
rustfmt --edition 2021 --check crates/ecaz-cli/src/commands/bench/mod.rs crates/ecaz-cli/src/commands/bench/recall.rs crates/ecaz-cli/src/commands/bench/latency.rs crates/ecaz-cli/src/commands/bench/suite.rs crates/ecaz-cli/src/commands/bench/spire_pipeline.rs src/am/ec_ivf/mod.rs src/am/ec_ivf/options.rs src/am/ec_ivf/scan.rs
```

`cargo test -p ecaz --lib adaptive_nprobe` built but could not execute in this local shell because the pgrx test binary failed dynamic lookup with `undefined symbol: LockBuffer`; this is the existing local standalone-pgrx-test limitation, not a failure in the changed logic. The IVF selector unit coverage was still added in code and will run under the proper pgrx/CI environment.

## Status

`suite-status.log`:

```text
[suite:task51-local-ivf-adaptive-nprobe-ratio] completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Key Results

Recall, q=100, k=10:

| mode | nprobe | recall@10 | recall_p10 | recall_worst | NDCG@10 | mean q-time |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| static | 64 | 0.9570 | 0.8900 | 0.5000 | 0.9971 | 310.56 ms |
| ratio=2500 | 64 | 0.9570 | 0.8900 | 0.5000 | 0.9971 | 286.35 ms |
| ratio=10000 | 64 | 0.9570 | 0.8900 | 0.5000 | 0.9971 | 297.77 ms |
| ratio=50000 | 64 | 0.9570 | 0.8900 | 0.5000 | 0.9971 | 293.22 ms |
| static | 128 | 0.9750 | 0.9000 | 0.5000 | 0.9986 | 570.63 ms |
| ratio=2500 | 128 | 0.9750 | 0.9000 | 0.5000 | 0.9986 | 561.76 ms |
| ratio=10000 | 128 | 0.9750 | 0.9000 | 0.5000 | 0.9986 | 578.00 ms |
| ratio=50000 | 128 | 0.9750 | 0.9000 | 0.5000 | 0.9986 | 564.13 ms |
| static | 256 | 0.9850 | 0.9900 | 0.5000 | 0.9995 | 1082.73 ms |
| ratio=2500 | 256 | 0.9850 | 0.9900 | 0.5000 | 0.9995 | 1192.23 ms |
| ratio=10000 | 256 | 0.9850 | 0.9900 | 0.5000 | 0.9995 | 1119.64 ms |
| ratio=50000 | 256 | 0.9850 | 0.9900 | 0.5000 | 0.9995 | 1117.21 ms |

Latency, q=100, concurrency 1:

| mode | nprobe | p50 | p95 | p99 | mean |
| --- | ---: | ---: | ---: | ---: | ---: |
| static | 64 | 282.5 ms | 336.4 ms | 356.9 ms | 285.1 ms |
| ratio=2500 | 64 | 301.3 ms | 362.1 ms | 389.2 ms | 301.4 ms |
| ratio=10000 | 64 | 290.7 ms | 355.9 ms | 362.3 ms | 293.3 ms |
| ratio=50000 | 64 | 286.5 ms | 343.0 ms | 362.6 ms | 285.2 ms |
| static | 128 | 557.2 ms | 646.9 ms | 659.6 ms | 558.2 ms |
| ratio=2500 | 128 | 592.0 ms | 681.8 ms | 715.5 ms | 588.9 ms |
| ratio=10000 | 128 | 557.8 ms | 637.7 ms | 709.4 ms | 563.9 ms |
| ratio=50000 | 128 | 551.6 ms | 628.0 ms | 646.9 ms | 554.2 ms |
| static | 256 | 1100.8 ms | 1178.4 ms | 1219.3 ms | 1088.4 ms |
| ratio=2500 | 256 | 1129.7 ms | 1272.5 ms | 1385.1 ms | 1127.2 ms |
| ratio=10000 | 256 | 1068.8 ms | 1173.1 ms | 1185.6 ms | 1068.2 ms |
| ratio=50000 | 256 | 1071.0 ms | 1165.0 ms | 1203.4 ms | 1071.8 ms |

## Interpretation

- The recall runner now reports worst-query recall; this closes the metric gap from reviewer feedback on packet 014.
- The non-time-based margin-ratio signal preserved recall and worst-query recall on this q=100 local smoke.
- The ratio signal did not produce a broad, useful latency win:
  - `ratio=2500` is slower than static across all latency p50/p95/p99 cells.
  - `ratio=10000` and `ratio=50000` are roughly static, with small mixed deltas that do not justify promotion.
- This closes Exp 5 locally as not productive for the current IVF/RaBitQ corpus and access path. It should not be promoted to AWS.
- Conservative deployment posture remains default-off. There is no recall-preserving operating point with a meaningful latency win to recommend enabling.

## Caveats

- This is local PG18/WSL2 smoke evidence only.
- q=100 is intentionally below the Task 51 final AWS evidence bar; this packet is an adaptive-policy screen and negative closeout, not an AWS promotion packet.
- The preserved 990k table/index was reused; no corpus rebuild ran.
- The suite intentionally did not run vchord or pgvectorscale.

## Artifacts

- `suite.json`: checked-in suite config.
- `artifacts/local-pgrx-install-release.log`: release local install before performance run.
- `artifacts/guc-ratio-show.log`: PG18 visibility check for `ec_ivf.adaptive_nprobe_score_margin_ratio_bps`.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build with ratio suite expansion and recall worst metric.
- `artifacts/suite-audit.log`: suite audit output.
- `artifacts/suite-dry-run.log`: suite dry-run with expanded commands.
- `artifacts/suite-run.log`: authoritative suite run.
- `artifacts/suite-status.log`: final suite status.
- `artifacts/suite-report.log`: parsed report.
- `artifacts/suite-manifest.json`: structured suite manifest.
- `artifacts/results.jsonl`: parsed structured results from the successful suite.
- `artifacts/results-report.jsonl`: parsed structured results from the final report command.
- `artifacts/recall-*.log`: recall tables.
- `artifacts/latency-*.log`: latency tables.
- `artifacts/truth-ec-real-990k-q100-k10.json`: packet-local q=100/k=10 truth cache copied from packet 014.
