# Manifest: Post-Main RaBitQ / IVF / SPIRE Sweep

- Task bucket: `reviews/task-50`
- Packet: `reviews/task-50/391-post-main-rabitq-ivf-spire-sweep`
- Head SHA during final manifest write:
  `14d8009373ceae63f3f6111d731911621cf92374`
- Merge SHA under validation:
  `69269be5aa961e8d4d6a4d335d5904c67a710710`
- Upstream main SHA merged:
  `24e7ea814`
- Branch: `task-50-unsafe-closeout`
- Timestamp: `2026-05-21T20:57:27-07:00`
- Primary target: PG18
- Local scratch connection: `localhost:28818`, database `tqvector_bench`
- Fixture scope: existing local prepared corpus surfaces
  `ec_real_10k_ivfrabitq` and `ec_real_10k_spirerabitq`
- Storage / rerank mode: RaBitQ, 4-bit suite default
- Query scope: `k=10`, `queries_limit=50`, `iterations=50`,
  `nprobe in {8,16}`, `force_index=true`
- Surface isolation note: separate corpus prefixes and indexes for IVF/RaBitQ
  and SPIRE/RaBitQ; not a shared-table cross-AM sweep.

## Code / Review Context

- `69269be5a` merged upstream Task 39 / Task 47 work from `origin/main`.
- `465d83def` consolidated the SPIRE DML predicate boundary.
- `3a1cbe69c` removed stale SPIRE DML test unsafe wrappers.
- `14d800937` committed packets 382-391, including the reviewer approval in
  `feedback/2026-05-21-01-reviewer.md`.

## Commands And Evidence

### Rust / pgrx test probes

- Command:
  `cargo test --no-default-features --features pg18,bench rabitq -- --nocapture`
- Artifact:
  `cargo-test-rabitq-pg18-bench.log`
- Result:
  failed before assertions because the test binary reported
  `undefined symbol: pg_re_throw`.

- Command:
  `cargo pgrx test --no-default-features -F 'pg18 bench' pg18 rabitq`
- Artifact:
  `cargo-pgrx-test-pg18-rabitq.log`
- Result:
  failed before assertions because the test binary reported
  `undefined symbol: LockBuffer`.

- Command:
  `cargo test --no-run --all-targets --no-default-features --features pg18,bench`
- Artifact:
  `cargo-test-no-run-pg18-bench.log`
- Result:
  completed successfully. Existing warning observed for unused SPIRE DML
  re-exports in `src/am/mod.rs`.

### Scratch cluster / corpus inventory

- Command:
  `target/debug/ecaz corpus list`
- Artifact:
  `ecaz-corpus-list.log`
- Result:
  failed because no host or hostaddr was configured.

- Command:
  `target/debug/ecaz dev scratch restart --pg 18 --database tqvector_bench --log-file reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/ecaz-dev-scratch-restart-pg18.log`
- Artifact:
  `ecaz-dev-scratch-restart-pg18.log`
- Result:
  restarted local PG18 scratch on port `28818`.

- Command:
  `target/debug/ecaz corpus list --host localhost --port 28818 --database tqvector_bench --log-file reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/ecaz-corpus-list-after-scratch.log`
- Artifact:
  `ecaz-corpus-list-after-scratch.log`
- Result:
  found prepared 10k IVF/RaBitQ and SPIRE/RaBitQ corpora with queries and
  indexes.

### SPIRE smoke probes

- Command:
  `scripts/run_spire_multicluster_pg18_smoke.sh`
- Artifact:
  `run-spire-multicluster-pg18-smoke.log`
- Result:
  failed because SPIRE remote search executor `endpoint_status`
  `requires_rabitq_storage_format` is not ready.

- Command:
  `scripts/run_spire_multicluster_customscan_read_pg18.sh`
- Artifact:
  `run-spire-multicluster-customscan-read-pg18.log`
- Result:
  passed. Key evidence includes `tuple_transport_status: ready`,
  `read_row=10|remote alpha|{red,blue}|domain alpha|(7,left)`,
  `typed_payload_probe=ready,pg_binary_attr_v1,t,t`, and
  `SPIRE multicluster CustomScan read passed`.

- Command:
  `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh`
- Artifact:
  `run-spire-multicluster-insert-read-after-customscan-pg18.log`
- Result:
  failed because local heap tuple delivery requires
  `custom_scan_tuple_delivery` before consuming remote placements.

- Command:
  `scripts/run_spire_multicluster_insert_read_after_customscan_pg18.sh --insert-mode trigger`
- Artifact:
  `run-spire-multicluster-insert-read-after-customscan-trigger-pg18.log`
- Result:
  failed with the same `custom_scan_tuple_delivery` requirement.

### Bench suite

- Suite config:
  `rabitq-ivf-spire-local-suite.json`
- Command:
  `target/debug/ecaz bench suite audit --config reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/rabitq-ivf-spire-local-suite.json --host localhost --port 28818 --database tqvector_bench --log-file reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/ecaz-bench-suite-audit.log`
- Artifact:
  `ecaz-bench-suite-audit.log`
- Result:
  audit passed: 4 steps.

- Command:
  `target/debug/ecaz bench suite run --config reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/rabitq-ivf-spire-local-suite.json --dry-run --host localhost --port 28818 --database tqvector_bench --manifest-output reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/suite-dry-run-manifest.json --log-file reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/ecaz-bench-suite-dry-run.log`
- Artifacts:
  `ecaz-bench-suite-dry-run.log`, `suite-dry-run-manifest.json`
- Result:
  dry-run expanded all four intended commands.

- Command:
  `target/debug/ecaz bench suite run --config reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/rabitq-ivf-spire-local-suite.json --host localhost --port 28818 --database tqvector_bench --manifest-output reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/suite-manifest.json --results-output reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/results.jsonl --log-file reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/ecaz-bench-suite-run.log`
- Artifacts:
  `ecaz-bench-suite-run.log`, `suite-manifest.json`, `results.jsonl`,
  `ivf-rabitq-10k-recall-k10.log`, `ivf-rabitq-10k-latency-k10-c1.log`,
  `spire-rabitq-10k-recall-k10.log`,
  `spire-rabitq-10k-latency-k10-c1.log`
- Result:
  completed 4 steps with 0 failures.

- Command:
  `target/debug/ecaz bench suite status --manifest reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/suite-manifest.json`
- Artifact:
  `ecaz-bench-suite-status.log`
- Result:
  `completed=4 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0`.

- Command:
  `target/debug/ecaz bench suite report --manifest reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/suite-manifest.json --results-output reviews/task-50/391-post-main-rabitq-ivf-spire-sweep/artifacts/results-report.jsonl`
- Artifacts:
  `ecaz-bench-suite-report.md`, `results-report.jsonl`
- Result:
  produced parsed report for all four successful suite steps.

## Key Result Lines

| Step | nprobe | Result |
| --- | ---: | --- |
| IVF RaBitQ recall | 8 | `recall@k=0.9720`, `ndcg@k=0.9995`, `mean q-time=59.36 ms` |
| IVF RaBitQ recall | 16 | `recall@k=0.9780`, `ndcg@k=0.9998`, `mean q-time=97.70 ms` |
| IVF RaBitQ latency | 8 | `mean=62.1 ms`, `p50=63.3 ms`, `p95=82.0 ms`, `p99=86.5 ms` |
| IVF RaBitQ latency | 16 | `mean=92.2 ms`, `p50=90.1 ms`, `p95=119.6 ms`, `p99=124.9 ms` |
| SPIRE RaBitQ recall | 8 | `recall@k=0.9880`, `ndcg@k=0.9996`, `mean q-time=330.34 ms` |
| SPIRE RaBitQ recall | 16 | `recall@k=0.9960`, `ndcg@k=0.9999`, `mean q-time=416.22 ms` |
| SPIRE RaBitQ latency | 8 | `mean=229.7 ms`, `p50=229.1 ms`, `p95=286.8 ms`, `p99=302.5 ms` |
| SPIRE RaBitQ latency | 16 | `mean=411.1 ms`, `p50=427.5 ms`, `p95=509.0 ms`, `p99=528.7 ms` |
