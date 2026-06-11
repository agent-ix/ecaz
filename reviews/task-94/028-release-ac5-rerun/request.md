# Task 94 Review Request: Release-Backend AC5 Rerun

## Scope

This packet closes the reviewer follow-up from packet 027:

- add a SQL-visible backend build-profile marker,
- teach `ecaz bench suite run` to preflight latency/recall suites and reject debug backends unless `--allow-debug-backend` is passed,
- reinstall a release PG18 backend,
- rerun the Task 94/101 AC5 latency matrix with no pg_test invocation between install and bench.

Code checkpoint: `5fc436162` (`Guard suites against debug backend runs`).

## Implementation Summary

- Added `ecaz_build_profile()` in `src/lib.rs`, returning `release` or `debug` from `cfg!(debug_assertions)`.
- Added suite preflight metadata to `crates/ecaz-cli/src/commands/bench/suite.rs`.
- `bench suite run` now queries `ecaz_build_profile()` before latency/recall steps, records the loaded backend path/SHA/profile into `suite-manifest.json`, and refuses debug backends unless `--allow-debug-backend` is explicit.

## Evidence

See `artifacts/manifest.md` for command provenance.

Backend proof:

- `artifacts/install-ecaz-pg18-release.log`: installed backend SHA `dc9b8141751dd3db0d58a10e1bd4d9681e03cf58dabac439305387f1f1cb6646`.
- `artifacts/build-profile-probe.log`: `SELECT ecaz_build_profile()` returned `release`.
- `artifacts/suite-manifest.json`: preflight recorded `backend.build_profile = "release"`, the same backend SHA, and `/home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so`.

Validation:

- `artifacts/cargo-test-ecaz-cli-suite.log`: `cargo test -p ecaz-cli bench::suite --no-default-features` passed, `46 passed`.
- `artifacts/suite-audit.log`: suite audit passed.
- `artifacts/suite-status.log`: selected latency run completed `8`, failed `0`, skipped `6` recall steps intentionally excluded by `--only`.

## AC5 Latency Result

The release-backend rerun is back in packet-025 territory and the batch path wins all IVF cells:

| Fixture | Sweep | batch-off p50 | batch-on p50 | batch-on vs off |
| --- | --- | ---: | ---: | ---: |
| IVF 10k | nprobe=32 | 2.95 ms | 2.79 ms | -5.4% |
| IVF 10k | nprobe=64 | 4.81 ms | 4.44 ms | -7.7% |
| IVF 25k | nprobe=32 | 5.73 ms | 5.33 ms | -7.0% |
| IVF 25k | nprobe=64 | 10.00 ms | 9.46 ms | -5.4% |
| IVF 100k | nprobe=32 | 18.50 ms | 16.60 ms | -10.3% |
| IVF 100k | nprobe=64 | 34.30 ms | 31.00 ms | -9.6% |

Compared to packet 025's release-baseline batch-on p50s, the current batch-on run is `3.8%` to `10.4%` faster across all six IVF cells. DiskANN forced grouped-PQ p50s are also release-profile again: 50k list_size 64/128 = `23.90 ms` / `18.10 ms`; 100k list_size 64/128 = `40.80 ms` / `32.00 ms`.

The packet 027 latency issue was caused by a debug `ecaz.so` installed by pg_test runs into the shared pgrx tree. This packet records the release backend in the suite manifest before step 1, so the AC5 rerun is no longer vulnerable to that silent profile skew.
