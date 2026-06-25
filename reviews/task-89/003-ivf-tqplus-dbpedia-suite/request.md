# Task 89 Review Request: IVF TQ+ DBPedia Suite and 10k Lane

## Summary

This checkpoint makes the IVF TQ+ calibration reloption reachable from
`ecaz bench suite`, fixes the first scan-time TQ+ bug found by a real 10k run,
and records the first successful DBPedia 10k A/B lane.

Code changes under review:

- `crates/ecaz-cli/src/profiles.rs` now allows `turboquant_calibration` as an
  `ec_ivf` reloption, so `ecaz corpus load --profile ec_ivf --storage-format
  turboquant --reloption turboquant_calibration=tqplus_experimental` can be
  driven by suite configs.
- `src/am/ec_ivf/scan.rs` now resolves scan-side quantizer metadata through the
  same TQ+ detection used by query preparation. This fixes the real benchmark
  failure: `ec_ivf prepared query does not match quantizer profile`.

Benchmark scaffold under review:

- `suite.json` defines separate baseline TurboQuant and TQ+ prefixes at each
  DBPedia scale.
- Each scale has load, recall@10, latency, and storage steps.
- The suite uses only `ecaz bench suite`; there is no ad hoc sweeper.

## Validation

- `cargo test -p ecaz-cli profiles::tests::ec_ivf_profile_uses_nprobe_and_raw_real_scan_query`
  - Result: pass
  - Log: `artifacts/cargo-test-ecaz-cli-profile.log`
- `cargo check -p ecaz --lib --no-default-features --features pg18`
  - Result: pass
  - Log: `artifacts/cargo-check-pg18-after-scan-resolver.log`
- `cargo test -p ecaz --lib --no-default-features --features pg18 tqplus_`
  - Result: pass
  - Log: `artifacts/cargo-test-tqplus-after-scan-resolver.log`
- `cargo pgrx install --release --pg-config /opt/homebrew/opt/postgresql@18/bin/pg_config --no-default-features --features pg18`
  - Result: pass
  - Log: `artifacts/cargo-pgrx-install-pg18-release-after-scan-resolver.log`
- `./target/debug/ecaz bench suite audit --config reviews/task-89/003-ivf-tqplus-dbpedia-suite/suite.json`
  - Result: `audit passed: 24 steps`
  - Log: `artifacts/suite-audit.log`
- `./target/debug/ecaz bench suite run --config reviews/task-89/003-ivf-tqplus-dbpedia-suite/suite.json --dry-run --manifest-output reviews/task-89/003-ivf-tqplus-dbpedia-suite/artifacts/suite-manifest-dry-run.json`
  - Result: dry-run expanded all 24 steps
  - Log: `artifacts/suite-dry-run.log`

## DBPedia 10k Result

Command:

```text
./target/debug/ecaz bench suite run --config reviews/task-89/003-ivf-tqplus-dbpedia-suite/suite.json --host /Users/peter/.pgrx --port 28818 --only-tag real10k
```

Result:

- 8 selected real10k steps succeeded.
- `artifacts/suite/results.jsonl` contains normalized recall, latency, and storage rows.
- `artifacts/suite/suite-manifest.json` records the selected/succeeded steps.

At `nprobe=48`:

```text
baseline recall@10=0.9770 p50=4.34ms p95=4.60ms index_per_row=983.9B
TQ+      recall@10=0.9720 p50=8.24ms p95=8.63ms index_per_row=985.5B
```

This is early evidence only. On DBPedia 10k no-QJL, this TQ+ calibration shape is
slower and slightly lower recall than baseline TQ at the representative nprobe.

## Not Claimed

This is not Task 89 closeout evidence.

Open gates:

- Run DBPedia 50k/100k from the suite.
- Add and run a non-1536-dimensional QJL/gamma-aware fixture.
- Measure insert/update drift.
- Add at least one non-DBPedia corpus.
