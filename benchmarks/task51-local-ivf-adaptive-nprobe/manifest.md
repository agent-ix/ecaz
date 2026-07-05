# Task 51 Local IVF Adaptive Nprobe Smoke

- head SHA: `5cf94f0c8dad41b366a5ecc4f6a26c44df38a801`
- timestamp: `2026-05-23T14:29:48Z`
- benchmark packet: `benchmarks/task51-local-ivf-adaptive-nprobe/`
- runner: `ecaz bench suite`
- SuiteConfig: `benchmarks/task51-local-ivf-adaptive-nprobe/suite.json`
- suite manifest: `benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/suite-manifest.json`
- results: `benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/results.jsonl`
- lane: local PG18 / WSL2 only
- AWS: not used
- competitors: none; this packet is IVF/RaBitQ only
- table surface: reused preserved isolated prefix `task51_local_990k_ivf_rabitq1_n1024_w50`
- corpus: staged anchor corpus, 990000 rows, 10000 query rows, dim 1536
- profile: `ec_ivf`
- storage format: `rabitq`
- reloptions: `nlists=1024`, `nprobe=256`, `training_sample_rows=10000`, `quant_bits=1`, `rerank=heap_f32`, `rerank_width=50`, `storage_format=rabitq`
- adaptive policy: opt-in `ec_ivf.adaptive_nprobe`, thresholds `1000`, `10000`, `100000` score-gap micros
- rerank mode: heap f32 rerank width 50
- recall query limit: 100 local smoke waiver
- latency iterations: 100, concurrency 1
- isolated one-index-per-table surface: yes, inherited from packet `benchmarks/task51-local-ivf-rabitq-990k/`

## Commands

Install local extension for GUC verification:

```text
cargo pgrx install --test --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config
target/debug/ecaz --host /home/peter/.pgrx --port 28818 dev sql --pg 18 --db tqvector_bench --socket-dir /home/peter/.pgrx --raw --sql "LOAD 'ecaz'; SHOW ec_ivf.adaptive_nprobe; SHOW ec_ivf.adaptive_nprobe_score_gap_micros;"
```

Release local extension install before performance run:

```text
cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config
```

CLI build and suite execution:

```text
cargo build -p ecaz-cli
target/debug/ecaz --log-file benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/suite-audit.log bench suite audit --config benchmarks/task51-local-ivf-adaptive-nprobe/suite.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/suite-dry-run-after-cli-build.log bench suite run --config benchmarks/task51-local-ivf-adaptive-nprobe/suite.json --dry-run --manifest-output benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/suite-manifest.json
target/debug/ecaz --host /home/peter/.pgrx --port 28818 --log-file benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/suite-run-release.log bench suite run --config benchmarks/task51-local-ivf-adaptive-nprobe/suite.json --manifest-output benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/suite-status.log bench suite status --manifest benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/suite-manifest.json
target/debug/ecaz --log-file benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/suite-report.log bench suite report --manifest benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/suite-manifest.json --results-output benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/results-report.jsonl
```

## Status

`suite-status.log`:

```text
[suite:task51-local-ivf-adaptive-nprobe] completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

## Key Results

Recall, q=100, k=10:

| mode | nprobe | recall@10 | recall_p10 | NDCG@10 | mean q-time |
| --- | ---: | ---: | ---: | ---: | ---: |
| static | 64 | 0.9570 | 0.8900 | 0.9971 | 299.34 ms |
| adaptive gap=1000 | 64 | 0.9490 | 0.8000 | 0.9960 | 226.71 ms |
| adaptive gap=10000 | 64 | 0.9570 | 0.8900 | 0.9971 | 293.10 ms |
| adaptive gap=100000 | 64 | 0.9570 | 0.8900 | 0.9971 | 297.17 ms |
| static | 128 | 0.9750 | 0.9000 | 0.9986 | 569.17 ms |
| adaptive gap=1000 | 128 | 0.9730 | 0.9000 | 0.9986 | 514.80 ms |
| adaptive gap=10000 | 128 | 0.9750 | 0.9000 | 0.9986 | 562.39 ms |
| adaptive gap=100000 | 128 | 0.9750 | 0.9000 | 0.9986 | 563.94 ms |
| static | 256 | 0.9850 | 0.9900 | 0.9995 | 1096.99 ms |
| adaptive gap=1000 | 256 | 0.9820 | 0.9000 | 0.9992 | 1032.89 ms |
| adaptive gap=10000 | 256 | 0.9850 | 0.9900 | 0.9995 | 1096.94 ms |
| adaptive gap=100000 | 256 | 0.9850 | 0.9900 | 0.9995 | 1089.26 ms |

Latency, q=100, concurrency 1:

| mode | nprobe | p50 | p95 | p99 | mean |
| --- | ---: | ---: | ---: | ---: | ---: |
| static | 64 | 290.1 ms | 361.7 ms | 382.2 ms | 290.7 ms |
| adaptive gap=1000 | 64 | 225.8 ms | 336.4 ms | 357.2 ms | 223.5 ms |
| adaptive gap=10000 | 64 | 304.8 ms | 360.4 ms | 371.0 ms | 301.6 ms |
| adaptive gap=100000 | 64 | 289.1 ms | 348.9 ms | 389.4 ms | 290.7 ms |
| static | 128 | 566.8 ms | 660.8 ms | 676.2 ms | 575.2 ms |
| adaptive gap=1000 | 128 | 540.4 ms | 617.3 ms | 688.6 ms | 504.3 ms |
| adaptive gap=10000 | 128 | 558.6 ms | 636.8 ms | 665.8 ms | 557.6 ms |
| adaptive gap=100000 | 128 | 569.3 ms | 656.7 ms | 661.1 ms | 567.4 ms |
| static | 256 | 1089.8 ms | 1184.3 ms | 1203.5 ms | 1085.5 ms |
| adaptive gap=1000 | 256 | 1071.5 ms | 1210.4 ms | 1236.3 ms | 1026.0 ms |
| adaptive gap=10000 | 256 | 1068.8 ms | 1182.0 ms | 1215.8 ms | 1076.7 ms |
| adaptive gap=100000 | 256 | 1113.2 ms | 1261.6 ms | 1320.4 ms | 1117.4 ms |

## Interpretation

- The current halving policy is wired and measurable locally.
- `gap=1000` is too aggressive: it improves local p50 at low nprobe but loses
  average recall and recall tail.
- `gap=10000` and `gap=100000` preserve recall on this q=100 smoke, but they
  mostly behave like static probing and do not produce a useful latency win.
- This is not enough to promote adaptive nprobe to AWS or production. The next
  adaptive slice should either expose actual selected-nprobe counters per query
  or use a more conservative policy that can preserve worst-query recall while
  still reducing work.

## Caveats

- This is local PG18/WSL2 smoke evidence only, not Graviton evidence.
- The recall query count is q=100 as a local smoke waiver. It is below the Task
  51 final evidence bar and does not satisfy the adaptive-policy acceptance
  requirement by itself.
- The suite intentionally did not run vchord or pgvectorscale.
- The first attempted suite run used a debug local extension install and was
  canceled with `pg_cancel_backend`; the successful benchmark evidence is from
  the later release extension install and `suite-run-release.log`.
- The suite reused the preserved 990k IVF/RaBitQ table/index from
  `benchmarks/task51-local-ivf-rabitq-990k/`; no corpus rebuild ran.

## Artifacts

- `suite.json`: checked-in suite config.
- `artifacts/local-pgrx-install.log`: debug local install used only to verify new GUC registration.
- `artifacts/local-pgrx-install-release.log`: release local install before performance run.
- `artifacts/cargo-build-ecaz-cli.log`: CLI build so suite expands adaptive fields.
- `artifacts/suite-audit.log`: audit output.
- `artifacts/suite-dry-run.log`: stale CLI dry-run that exposed missing adaptive expansion.
- `artifacts/suite-dry-run-after-cli-build.log`: authoritative dry-run with adaptive flags.
- `artifacts/suite-run.log`: canceled debug-mode partial run.
- `artifacts/suite-run-release.log`: authoritative suite run.
- `artifacts/suite-status.log`: final suite status.
- `artifacts/suite-report.log`: parsed report.
- `artifacts/suite-manifest.json`: structured suite manifest.
- `artifacts/results.jsonl`: parsed structured results from the successful suite.
- `artifacts/results-report.jsonl`: parsed structured results from the final report command.
- `artifacts/recall-*.log`: recall tables.
- `artifacts/latency-*.log`: latency tables.
- `artifacts/truth-ec-real-990k-q100-k10.json`: copied q=100/k=10 truth cache from the prior 990k packet.
