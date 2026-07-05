# Task 101 Review Request: AC5 Release-Backend Latency Closeout

## Scope

Task 101 packet 003 was approved except for AC5, pending a rerun on a verified release backend after the Task 94 packet 027 root-cause finding. This packet carries the shared release-backend latency rerun evidence and the suite-runner guard that prevents silently measuring a debug `ecaz.so`.

Code checkpoint: `5fc436162` (`Guard suites against debug backend runs`).

## Implementation Summary

- Added SQL-visible `ecaz_build_profile()` returning `release` or `debug`.
- Added `ecaz bench suite run` preflight for latency/recall suites.
- The preflight records backend profile, path, and SHA into `suite-manifest.json`, and refuses debug backends unless `--allow-debug-backend` is passed.

## Evidence

Packet-local artifact copies are under `artifacts/`; original owning packet is `reviews/task-94/028-release-ac5-rerun/`.

- `artifacts/build-profile-probe.log`: `SELECT ecaz_build_profile()` returned `release`.
- `artifacts/suite-manifest.json`: recorded `backend.build_profile = "release"` and backend SHA `dc9b8141751dd3db0d58a10e1bd4d9681e03cf58dabac439305387f1f1cb6646`.
- `artifacts/cargo-test-ecaz-cli-suite.log`: `cargo test -p ecaz-cli bench::suite --no-default-features` passed, `46 passed`.
- `artifacts/suite-status.log`: selected latency run completed `8`, failed `0`, skipped `6` recall steps intentionally excluded by `--only`.

## AC5 Result

The verified-release rerun shows no end-to-end regression. Batch-on wins every IVF cell:

| Fixture | Sweep | batch-off p50 | batch-on p50 | batch-on vs off |
| --- | --- | ---: | ---: | ---: |
| IVF 10k | nprobe=32 | 2.95 ms | 2.79 ms | -5.4% |
| IVF 10k | nprobe=64 | 4.81 ms | 4.44 ms | -7.7% |
| IVF 25k | nprobe=32 | 5.73 ms | 5.33 ms | -7.0% |
| IVF 25k | nprobe=64 | 10.00 ms | 9.46 ms | -5.4% |
| IVF 100k | nprobe=32 | 18.50 ms | 16.60 ms | -10.3% |
| IVF 100k | nprobe=64 | 34.30 ms | 31.00 ms | -9.6% |

Compared to packet 025's release-baseline batch-on p50s, the current Task 101 batch-on run is `3.8%` to `10.4%` faster across the six IVF cells. Width-bucket counter rows are present in `results.jsonl`, and scalar attribution remains zero for the verified AVX2 batch-on rows.

This closes the Task 101 AC5 gap called out in packet 003 feedback.
