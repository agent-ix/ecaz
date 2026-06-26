---
head_sha: 1d9b3d20a31c9378d8deffe3ab482848b059519e
task_bucket: reviews/task-89
packet: reviews/task-89/003-ivf-tqplus-dbpedia-suite
timestamp_utc: 2026-06-25T22:12:51Z
---

# Artifact Manifest

## Scope

Task 89 IVF TQ+ DBPedia suite scaffold plus successful real-corpus 10k and 50k
A/B lanes. This packet does not close Task 89: DBPedia 100k, QJL-active fixture
coverage, insert drift, and non-DBPedia evidence remain open.

## Code Checkpoint

- Head SHA: `1d9b3d20a31c9378d8deffe3ab482848b059519e`
- Code commits covered:
  - `048afad36` allows `turboquant_calibration` through the `ecaz-cli` IVF reloption registry.
  - `1d9b3d20a` resolves TQ+ metadata to `IvfQuantizerProfile::TurboQuantTqPlus` during IVF scan scoring, matching query preparation and fixing the observed prepared-query/profile mismatch.

## Suite Shape

### `suite.json`

- Lane: local PG18 `ec_ivf`
- Fixture: staged DBPedia `data/staged-current/ec_real_{10k,50k,100k}_*.tsv`
- Storage format: `storage_format=turboquant`
- Variants:
  - baseline TurboQuant
  - TQ+ via `turboquant_calibration=tqplus_experimental`
- Query/rerank mode: no rerank reloption, forced index recall/latency scans
- Isolation: one prefix/table/index per scale and variant
- Matrix: load, recall@10, latency, storage for 10k/50k/100k

### `suite-audit.log`

Command:

```text
./target/debug/ecaz bench suite audit --config reviews/task-89/003-ivf-tqplus-dbpedia-suite/suite.json
```

Key result:

```text
[suite:task89-ivf-tqplus-dbpedia] audit passed: 24 steps
```

### `suite-dry-run.log`

Command:

```text
./target/debug/ecaz bench suite run --config reviews/task-89/003-ivf-tqplus-dbpedia-suite/suite.json --dry-run --manifest-output reviews/task-89/003-ivf-tqplus-dbpedia-suite/artifacts/suite-manifest-dry-run.json
```

Key result:

```text
[suite:task89-ivf-tqplus-dbpedia] wrote reviews/task-89/003-ivf-tqplus-dbpedia-suite/artifacts/suite-manifest-dry-run.json
```

## Validation Artifacts

### `cargo-test-ecaz-cli-profile.log`

Command:

```text
cargo test -p ecaz-cli profiles::tests::ec_ivf_profile_uses_nprobe_and_raw_real_scan_query
```

Key result:

```text
test profiles::tests::ec_ivf_profile_uses_nprobe_and_raw_real_scan_query ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 408 filtered out
```

### `cargo-check-pg18-after-scan-resolver.log`

Command:

```text
cargo check -p ecaz --lib --no-default-features --features pg18
```

Key result:

```text
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### `cargo-test-tqplus-after-scan-resolver.log`

Command:

```text
cargo test -p ecaz --lib --no-default-features --features pg18 tqplus_
```

Key result:

```text
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 2221 filtered out
```

### `cargo-pgrx-install-pg18-release-after-scan-resolver.log`

Command:

```text
cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config --no-default-features --features pg18
```

Key result:

```text
Finished `release` profile [optimized] target(s)
Finished installing ecaz
```

## DBPedia 10k Evidence

### `suite-run-real10k.log`

Command:

```text
./target/debug/ecaz bench suite run --config reviews/task-89/003-ivf-tqplus-dbpedia-suite/suite.json --host /Users/peter/.pgrx --port 28818 --only-tag real10k
```

- Database: `tqvector_bench`
- Socket: `/Users/peter/.pgrx`
- Port: `28818`
- Selected steps: 8 succeeded, remaining 16 skipped by `--only-tag real10k`
- Normalized rows: `artifacts/suite/results.jsonl`
- Manifest: `artifacts/suite/suite-manifest.json`

Key nprobe=48 rows:

```text
baseline recall@10=0.9770 mean_q_time=4.65ms latency_p50=4.34ms latency_p95=4.60ms index_per_row=983.9B
TQ+      recall@10=0.9720 mean_q_time=8.16ms latency_p50=8.24ms latency_p95=8.63ms index_per_row=985.5B
```

Load timing:

```text
baseline load total=9.77s
TQ+      load total=9.79s
```

Storage:

```text
baseline total=168.4MiB index=9.4MiB per_row_total=17661.1B
TQ+      total=168.4MiB index=9.4MiB per_row_total=17662.8B
```

## DBPedia 50k Evidence

### `suite-run-real50k.log`

Command:

```text
./target/debug/ecaz bench suite run --config reviews/task-89/003-ivf-tqplus-dbpedia-suite/suite.json --artifact-dir reviews/task-89/003-ivf-tqplus-dbpedia-suite/artifacts/suite-real50k --host /Users/peter/.pgrx --port 28818 --only-tag real50k
```

- Database: `tqvector_bench`
- Socket: `/Users/peter/.pgrx`
- Port: `28818`
- Selected steps: 8 succeeded, remaining 16 skipped by `--only-tag real50k`
- Normalized rows: `artifacts/suite-real50k/results.jsonl`
- Manifest: `artifacts/suite-real50k/suite-manifest.json`

Key nprobe=64 rows:

```text
baseline recall@10=0.9430 mean_q_time=8.48ms latency_p50=8.50ms latency_p95=9.12ms index_per_row=941.6B
TQ+      recall@10=0.9460 mean_q_time=24.86ms latency_p50=24.6ms latency_p95=26.8ms index_per_row=941.9B
```

Load timing:

```text
baseline load total=89.93s
TQ+      load total=84.37s
```

Storage:

```text
baseline total=839.8MiB index=44.9MiB per_row_total=17611.8B
TQ+      total=839.8MiB index=44.9MiB per_row_total=17612.1B
```

## Known Gaps

- DBPedia 100k has suite steps but has not been run in this packet.
- The local staged DBPedia fixtures are all 1536-dimensional, which uses the
  no-QJL TurboQuant tile path. QJL/gamma-aware TQ+ still needs a separate
  non-tile-dimensional fixture before Task 89 can claim broader format coverage.
- Insert/update drift and non-DBPedia corpus evidence remain open.
