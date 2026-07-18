# Task 183 latency-attribution manifest

- Pre-registration head: `8c47c940830eab0c63f0e08cf886e80207dd2afb`
- Task bucket / packet: `reviews/task-183/005-latency-attribution/`
- Implementation head: `03921f632`
- Lane: benchmark-only stage-counter implementation and frozen 100k profile;
  measurement pending
- Retained policy: Task 182 training landmarks, cap 4,096, exact head scoring,
  32 seeds, BW4/H100, RaBitQ traversal, exact final ranking
- Evaluation input: held-out rows 1--200
- Initial profile: 100k, 50 timed latency queries after 10 warmups,
  concurrency 1, warm cache
- Isolation: fresh one-index-per-table physical generation through the
  checked-in `ecaz bench suite` config
- Timestamp: 2026-07-17 America/Los_Angeles

## Validation artifacts

- `pg18-feature-check.log`
  - command: `cargo check --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark'`
  - result: pass
  - SHA-256: `ad5b90339baa80fb3125390a71f3aa7863c2b8b58b1b80e117f5978deb7d4286`
- `production-feature-check.log`
  - command: `cargo check --no-default-features --features pg18`
  - result: pass; measurement-only code remains feature-gated
  - SHA-256: `6717d387aae9ef7b109ce063b3bc817a744057ee37434befd4485056daba7b82`
- `stage-counter-test.log`
  - command: `cargo test --lib --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark' am::ec_distann::stage_counters::tests::counters_accumulate_nested_samples_and_reset -- --exact`
  - result: 1 passed, 0 failed
  - SHA-256: `a82b36e4f4d7546903c6ee39425e1873705a0428eb56ce40b2d2d0dcf23d0fd1`
- `cli-counter-test.log`
  - command: `cargo test -p ecaz-cli distann_stage_counters_merge_and_report_per_scan_mean`
  - result: 1 passed, 0 failed
  - SHA-256: `797adca67d2e0ebc4d1550db9c5cc3ff21fc4f86f496c7fb77f7d8bc5da9a0bc`
- `suite-profile-test.log`
  - command: `cargo test -p ecaz-cli distann_local_multinode_expands_task183_stage_profile`
  - result: 1 passed, 0 failed
  - SHA-256: `55130c7fe8792746c5ff7f064896bcd3eb35d1d0e6d7cdd6f95119f5e0b68645`
- `cli-debug-build.log`
  - command: `cargo build -p ecaz-cli`
  - result: pass; only the pre-existing unused `path` warning
  - SHA-256: `9bb28d79b399ea2e1d8a86d71e1db52f7200809401d9acba48b5097384a627f8`
- `latency-profile-100k-suite.json`
  - runner: `ecaz bench suite`
  - result: one fresh 100k trained-production physical fixture; 50 timed
    queries after 10 warmups; stage counters enabled
  - SHA-256: `d513813531933e9e276cdf1ad407f745921ad8b7987415b4bb839aba227f9e68`
- `audit.log`
  - command: `target/debug/ecaz bench suite audit --config reviews/task-183/005-latency-attribution/artifacts/latency-profile-100k-suite.json`
  - result: pass, 1 step
  - SHA-256: `3025dd0afdbd598d618064aa92416056dc22a3dea8d0101b4939bf929e3f2ac7`
- `dry-run.log` and `run/suite-manifest.json`
  - command: `target/debug/ecaz bench suite run --config reviews/task-183/005-latency-attribution/artifacts/latency-profile-100k-suite.json --dry-run`
  - result: expands the intended policy/work contract and
    `--distann-stage-counters`
  - log SHA-256: `bdd8c4c2aef5aa1a44b17e7116719865048c833ed01a875d9506f4a66d3bdbef`
  - dry-run manifest SHA-256:
    `e31c2c1048abd66d69e8b267d87f0238991fff743b2e1384c2b203803fff0351`

No profile result or optimization decision is claimed yet.
