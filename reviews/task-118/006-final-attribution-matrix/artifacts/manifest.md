# Task 118 Final Attribution Matrix Artifacts

## Packet

- Task bucket: `reviews/task-118/006-final-attribution-matrix`
- Branch: `task-118-hnsw-quantized-recall-attribution`
- Current checkpoint SHA: `e94ba88d199a972b3152bf028ba44a02033b2f17`
- Timestamp: `2026-06-24T10:35:10-0700`

## M5 release closeout evidence

This update supersedes the earlier AMD-local partial evidence for Task 118
closeout. The benchmark host was the M5 laptop, using local PG18 on
`/Users/peter/.pgrx:28818` and database `tqvector_task118_m5_release`.
`ecaz_build_profile()` returned `release` after installing the release PG
extension. The suite runner was the branch-local `./target/debug/ecaz`; measured
query/build/storage work ran inside the release PostgreSQL extension backend.

Suite config:

- `crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json`

Corpus staging:

The staged corpus/query TSV files are intentionally not committed.

| Profile | Rows | Queries | Corpus SHA256 | Queries SHA256 |
| --- | ---: | ---: | --- | --- |
| `ec_real_10k` | 10000 | 200 | `c67c5810b66d982d705974e48d4775479adfbd92a988f694091266e049a35e75` | `a2c191bb742017d849e73f6e6866e8e0f0bac1579ba212f7fc76b8eb09904ae8` |
| `ec_real_50k` | 50000 | 1000 | `56023baaa7bc42f758272e8617603d538808e6290a8a70a3a84e057571240133` | `95ac7992578aa80bb193657f10fbcbf1ea3867e559739244bf5a467f7a5a9fa3` |
| `ec_real_100k` | 100000 | 1000 | `07275cfd5a7a4b415ddf5eacc086de98294ac978532df46ffae30f9202323a95` | `a7cbec6fc44f6c148234538f61339d00d2f10646febc8f667dcbe75d9cf41782` |

Release suite commands:

- 10k:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --database tqvector_task118_m5_release --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-10k-m5-release.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --manifest-output reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-10k-m5-release.json --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-m5-release.jsonl --only-tag ec_real_10k --continue-on-error`
- 50k:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --database tqvector_task118_m5_release --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-50k-m5-release.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --manifest-output reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-50k-m5-release.json --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-m5-release.jsonl --only-tag ec_real_50k --continue-on-error`
- 100k:
  `./target/debug/ecaz --host /Users/peter/.pgrx --port 28818 --database tqvector_task118_m5_release --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-100k-m5-release.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --manifest-output reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-100k-m5-release.json --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-m5-release.jsonl --only-tag ec_real_100k --continue-on-error`

Suite status:

| Scale | Manifest | Results | Status |
| --- | --- | --- | --- |
| 10k | `suite-manifest-10k-m5-release.json` | `results-10k-m5-release.jsonl` | `completed=36 failed=0 skipped=72 dry_run=0 missing_artifacts=0 stale=0`; `180` result rows |
| 50k | `suite-manifest-50k-m5-release.json` | `results-50k-m5-release.jsonl` | `completed=36 failed=0 skipped=72 dry_run=0 missing_artifacts=0 stale=0`; `180` result rows |
| 100k | `suite-manifest-100k-m5-release.json` | `results-100k-m5-release.jsonl` | `completed=36 failed=0 skipped=72 dry_run=0 missing_artifacts=0 stale=0`; `180` result rows |

Derived summary:

- `task118-m5-release-summary.jq`
  - Command:
    `jq -sr -f reviews/task-118/006-final-attribution-matrix/artifacts/task118-m5-release-summary.jq reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-m5-release.jsonl reviews/task-118/006-final-attribution-matrix/artifacts/results-50k-m5-release.jsonl reviews/task-118/006-final-attribution-matrix/artifacts/results-100k-m5-release.jsonl > reviews/task-118/006-final-attribution-matrix/artifacts/final-decision-table-m5-release.txt`
- `final-decision-table-m5-release.txt`
  - Source of truth for the compact 10k/50k/100k M5 release decision table.

Score-correlation runtime sanity:

- `cargo-pgrx-test-pg18-score-sanity-m5.log`
  - Head SHA: `e94ba88d199a972b3152bf028ba44a02033b2f17`
  - Command:
    `script -q reviews/task-118/006-final-attribution-matrix/artifacts/cargo-pgrx-test-pg18-score-sanity-m5.log cargo pgrx test pg18 test_ech_score_correlation_synthetic_known_ordering`
  - Result: `1 passed; 0 failed`

Key `ef_search=200` M5 release results:

| Scale | Format | Source recall@10 | Compressed recall@10 | Source truth@10 in emitted pool | Mean Spearman | Source total storage |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| 10k | TurboQuant | 0.9705 | 0.9705 | 0.9965 | 0.8404 | 172.1 MiB |
| 10k | PqFastScan | 0.9945 | 0.9945 | 0.9960 | 0.8404 | 172.2 MiB |
| 10k | RaBitQ | 0.9480 | 0.9480 | 0.9705 | 0.9086 | 172.1 MiB |
| 50k | TurboQuant | 0.9315 | 0.9315 | 0.9815 | 0.6818 | 860.0 MiB |
| 50k | PqFastScan | 0.9735 | 0.9735 | 0.9815 | 0.6819 | 860.1 MiB |
| 50k | RaBitQ | 0.9130 | 0.9130 | 0.9530 | 0.8092 | 860.0 MiB |
| 100k | TurboQuant | 0.9245 | 0.9245 | 0.9675 | 0.6747 | 1.7 GiB |
| 100k | PqFastScan | 0.9630 | 0.9630 | 0.9675 | 0.6747 | 1.7 GiB |
| 100k | RaBitQ | 0.8990 | 0.8990 | 0.9325 | 0.8111 | 1.7 GiB |

Notes:

- Source-build and compressed-build recall match at every scale for every
  format, so the build-source-column A/B is neutral in this M5 release matrix.
- PqFastScan has the strongest 50k/100k recall. It has the same emitted-pool
  containment shape as TurboQuant, but uses exact rerank for the emitted pool.
- TurboQuant and RaBitQ use quantized rerank in these rows. RaBitQ has stronger
  score correlation than TurboQuant/PqFastScan but lower candidate containment
  and lower final recall.
- The raw truth cache files were generated to support recall runs and are not
  intended as cited decision artifacts.

## Checkpoint: compressed build prefix fix

The initial 10k suite pass exposed an identifier-length failure for the
TurboQuant and PqFastScan compressed-build loads. This checkpoint shortens the
compressed-build `prefix` values in
`crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json` while
leaving step names and artifact names unchanged.

Artifacts:

- `cargo-test-ecaz-cli-hnsw-prefix-fix.log`
  - Head SHA: `1a6e75720b5f10b8999a1d94958e99be39df2eff`
  - Command: `cargo test -p ecaz-cli hnsw -- --nocapture`
  - Result: `21 passed; 0 failed; 394 filtered out`
- `suite-dry-run-10k-compressed-prefix-fix.log`
  - Head SHA: `1a6e75720b5f10b8999a1d94958e99be39df2eff`
  - Command: `cargo run -p ecaz-cli -- --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-dry-run-10k-compressed-prefix-fix.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --only load-10k-hnsw-turboquant-compressed-build --only load-10k-hnsw-pq-fastscan-compressed-build --only load-10k-hnsw-rabitq-compressed-build --dry-run --allow-debug-backend`
  - Result: dry-run expands compressed-build load prefixes to `task118_r10k_tq_cb`, `task118_r10k_pq_cb`, and `task118_r10k_rq_cb`.

## Superseded in-progress matrix evidence

The older sections below predate the M5 release closeout update above. They
remain here as packet history, but the closeout evidence is now the M5 release
matrix, manifests, results JSONL, and final decision table listed above.

## 10k attribution checkpoint

Head SHA: `71b622d8a` for the packet update; code/config under measurement was
`1a6e75720b5f10b8999a1d94958e99be39df2eff`.

Commands:

- Source-build 10k pass:
  `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database tqvector_bench --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-10k.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --manifest-output reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-10k.json --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-10k.jsonl --only-tag ec_real_10k --continue-on-error --allow-debug-backend`
- Corrected compressed-build 10k rerun:
  `cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database tqvector_bench --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-10k-compressed-rerun.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --manifest-output reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-10k-compressed-rerun.json --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-10k-compressed-rerun.jsonl --only <18 explicit 10k compressed-build steps> --continue-on-error --allow-debug-backend`

Artifacts:

- `suite-manifest-10k.json`, `results-10k.jsonl`, `suite-run-10k.log`
  - Source-build 10k pass.
  - Manifest status: `26` succeeded, `10` failed, `72` skipped.
  - The failures were only the pre-fix TurboQuant/PqFastScan compressed-build
    steps; the source-build 10k rows succeeded.
- `suite-manifest-10k-compressed-rerun.json`,
  `results-10k-compressed-rerun.jsonl`,
  `suite-run-10k-compressed-rerun.log`
  - Corrected compressed-build 10k pass.
  - Manifest status: `18` succeeded, `90` skipped, no failed selected steps.
- Per-step `load-*`, `recall-*`, `frontier-*`, `score-correlation-*`,
  `latency-*`, and `storage-*` logs for 10k source-build and compressed-build
  runs.
  - Raw per-query frontier/score JSONL files are intentionally not cited as the
    checkpoint source of truth; the summarized per-step logs and suite results
    contain the decision-grade rows.

Key 10k `ef_search=200` results:

| Format | Build path | Recall@10 | Truth@10 in frontier | Exact rerank | Dropped before exact | Mean Spearman | Latency mean / p95 / p99 | Total / index storage |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- | --- |
| TurboQuant | source | 0.9950 | 0.9965 | 200 | 0 | 0.8404 | 42.4 ms / 49.6 ms / 60.0 ms | 172.1 MiB / 13.0 MiB |
| TurboQuant | compressed | 0.9950 | 0.9965 | 200 | 0 | 0.8404 | 42.5 ms / 49.5 ms / 55.7 ms | 172.1 MiB / 13.0 MiB |
| PqFastScan | source | 0.9945 | 0.9960 | 200 | 0 | 0.8404 | 45.7 ms / 54.5 ms / 60.2 ms | 172.2 MiB / 13.1 MiB |
| PqFastScan | compressed | 0.9945 | 0.9960 | 200 | 0 | 0.8404 | 45.0 ms / 51.0 ms / 60.0 ms | 172.2 MiB / 13.1 MiB |
| RaBitQ | source | 0.9705 | 0.9705 | 200 | 0 | 0.9086 | 80.9 ms / 92.7 ms / 109.4 ms | 172.1 MiB / 13.0 MiB |
| RaBitQ | compressed | 0.9705 | 0.9705 | 200 | 0 | 0.9086 | 81.4 ms / 93.1 ms / 115.3 ms | 172.1 MiB / 13.0 MiB |

10k interpretation:

- Source-build and compressed-build results match for all three formats at
  10k; no recall loss is attributable to the HNSW build source column at this
  scale.
- TurboQuant/PqFastScan candidate containment slightly exceeds final recall,
  while final rerank counters show no truncation before exact rerank.
- RaBitQ recall equals `truth@10 in frontier`, despite stronger score
  correlation than TurboQuant/PqFastScan. At 10k, the dominant RaBitQ loss is
  candidate containment/traversal, not final exact rerank or scorer ordering.

## 50k AMD-local partial checkpoint

This host is the slower AMD machine. Treat the 50k rows below as AMD-local
relative evidence only; final closeout measurement should be produced on the
Intel benchmark desktop when it is available.

Command:

`cargo run -p ecaz-cli -- --host /home/peter/.pgrx --port 28818 --database tqvector_bench --log-file reviews/task-118/006-final-attribution-matrix/artifacts/suite-run-50k.log bench suite run --config crates/ecaz-cli/suites/task118-hnsw-quantized-recall-attribution.json --artifact-dir reviews/task-118/006-final-attribution-matrix/artifacts --manifest-output reviews/task-118/006-final-attribution-matrix/artifacts/suite-manifest-50k.json --results-output reviews/task-118/006-final-attribution-matrix/artifacts/results-50k.jsonl --only-tag ec_real_50k --continue-on-error --allow-debug-backend`

Artifacts:

- `suite-manifest-50k.json`
  - Status at interrupt: `6` succeeded, `30` pending, `72` skipped.
  - Completed steps: source-build load and recall for TurboQuant, PqFastScan,
    and RaBitQ.
- `suite-run-50k.log`
  - Runner log for the AMD-local partial pass.
- `load-50k-hnsw-{turboquant,pq-fastscan,rabitq}.log`
  - Source-build load logs.
- `recall-50k-hnsw-{turboquant,pq-fastscan,rabitq}.log`
  - Source-build recall logs.
- `scratch-restart-after-amd-frontier-cancel.log`
  - After the suite was interrupted, the active frontier helper backend did not
    respond to PostgreSQL cancel/terminate. The PG18 scratch cluster was
    restarted to stop the AMD-local benchmark backend.

No `results-50k.jsonl` is present because the suite was intentionally
interrupted before normal result emission. The raw `truth-50k-k10.json` cache is
not committed.

50k AMD-local source-build recall at `ef_search=200`:

| Format | Recall@10 | Mean q-time |
| --- | ---: | ---: |
| TurboQuant | 0.9735 | 49.77 ms |
| PqFastScan | 0.9735 | 52.75 ms |
| RaBitQ | 0.9520 | 85.82 ms |

The AMD-local 50k recall shape matches the 10k direction: RaBitQ remains lower
than TurboQuant/PqFastScan and slower. Frontier, score-correlation, latency,
storage, compressed-build A/B, and 100k evidence remain for the Intel host.
