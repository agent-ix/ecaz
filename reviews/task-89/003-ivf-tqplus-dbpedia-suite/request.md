# Task 89 Review Request: IVF TQ+ DBPedia Suite Scaffold

## Summary

This checkpoint makes the IVF TQ+ calibration reloption reachable from the
standard `ecaz bench suite` load path and adds a packet-local DBPedia A/B suite
for the required 10k/50k/100k measurement matrix.

Code change under review:

- `crates/ecaz-cli/src/profiles.rs` now allows `turboquant_calibration` as an
  `ec_ivf` reloption, so `ecaz corpus load --profile ec_ivf --storage-format
  turboquant --reloption turboquant_calibration=tqplus_experimental` can be
  driven by suite configs.

Benchmark scaffold under review:

- `suite.json` defines separate baseline TurboQuant and TQ+ prefixes at each
  DBPedia scale.
- Each scale has load, recall@10, latency, and storage steps.
- The suite uses only `ecaz bench suite`; there is no ad hoc sweeper.

## Validation

- `cargo test -p ecaz-cli profiles::tests::ec_ivf_profile_uses_nprobe_and_raw_real_scan_query`
  - Result: pass
  - Log: `artifacts/cargo-test-ecaz-cli-profile.log`
- `./target/debug/ecaz bench suite audit --config reviews/task-89/003-ivf-tqplus-dbpedia-suite/suite.json`
  - Result: `audit passed: 24 steps`
  - Log: `artifacts/suite-audit.log`
- `./target/debug/ecaz bench suite run --config reviews/task-89/003-ivf-tqplus-dbpedia-suite/suite.json --dry-run --manifest-output reviews/task-89/003-ivf-tqplus-dbpedia-suite/artifacts/suite-manifest-dry-run.json`
  - Result: dry-run expanded all 24 steps
  - Log: `artifacts/suite-dry-run.log`

## Not Claimed

This is not Task 89 closeout evidence. The full benchmark run has not been
executed in this packet.

The available local staged corpora are DBPedia 1536-dimensional fixtures, so
this suite covers the real no-QJL TurboQuant lane only. The QJL/gamma-aware TQ+
benchmark lane still needs a separate non-tile-dimensional fixture before the
task can satisfy the requested broader format evaluation.
