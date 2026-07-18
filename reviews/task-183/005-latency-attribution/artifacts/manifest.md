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
- Installed release measurement head:
  `97cd5a76a5ea2d20ef94078566f66f85dacc97b2`

## Validation artifacts

- `pg18-feature-check.log`
  - command: `cargo check --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark'`
  - result: pass
  - SHA-256: `85f187f6d6b4f081714434d0a5a02d838d58f4333b8d6e75b42de99e82fa6c47`
- `production-feature-check.log`
  - command: `cargo check --no-default-features --features pg18`
  - result: pass; measurement-only code remains feature-gated
  - SHA-256: `c083ccdca73052781d13899fcb3ef201d671157a0ff731f4dfa516931e556504`
- `stage-counter-test.log`
  - command: `cargo test --lib --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark' am::ec_distann::stage_counters::tests::counters_accumulate_nested_samples_and_reset -- --exact`
  - result: 1 passed, 0 failed
  - SHA-256: `99a691b2825b8a8bbe3af656000bd890f5824cb25859d9e3a8e4efb3e84f4c70`
- `cli-counter-test.log`
  - command: `cargo test -p ecaz-cli distann_stage_counters_merge_and_report_per_scan_mean`
  - result: 1 passed, 0 failed
  - SHA-256: `b93ed9287a51a4686d065becbd7905ff7bd05b5e8ed8ea0cfa499c080a10d6d9`
- `suite-profile-test.log`
  - command: `cargo test -p ecaz-cli distann_local_multinode_expands_task183_stage_profile`
  - result: 1 passed, 0 failed
  - SHA-256: `97f9cb7182cad3d9f7b5b8361e9fc3bf55fe91e1d298099538e1a4bdf0a58496`
- `cli-debug-build.log`
  - command: `cargo build -p ecaz-cli`
  - result: pass; only the pre-existing unused `path` warning
  - SHA-256: `c0e629d1f18f330e8a70cee6ee2c287a7b04a3204093e8df8cbe41ceaa854a13`
- `latency-profile-100k-suite.json`
  - runner: `ecaz bench suite`
  - result: one fresh 100k trained-production physical fixture; 50 timed
    queries after 10 warmups; stage counters enabled
  - SHA-256: `d513813531933e9e276cdf1ad407f745921ad8b7987415b4bb839aba227f9e68`
- `audit.log`
  - command: `target/debug/ecaz bench suite audit --config reviews/task-183/005-latency-attribution/artifacts/latency-profile-100k-suite.json`
  - result: pass, 1 step
  - SHA-256: `8e593d39857a8c90b28ddeaa17b953ed650b8e6b5fc99982f5a489ce93af20f2`
- `dry-run.log` and `run/suite-manifest.json`
  - command: `target/debug/ecaz bench suite run --config reviews/task-183/005-latency-attribution/artifacts/latency-profile-100k-suite.json --dry-run`
  - result: expands the intended policy/work contract and
    `--distann-stage-counters`
  - log SHA-256: `c3ceb47d129f5f41a6fc4287dbe71e2d7bb49095d92113cb4e180a6d0712b801`
  - dry-run manifest SHA-256:
    `e31c2c1048abd66d69e8b267d87f0238991fff743b2e1384c2b203803fff0351`
- `implementation-install.log`
  - command: `cargo pgrx install --release --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config --no-default-features --features 'pg18 pg_test distann-head-attribution-benchmark'`
  - result: pass; 1,117 SQL entities discovered and installed
  - log SHA-256: `642796eededa62bc6345f48d4fabd19b26dfc3d393e7758141e2ec552982bb71`
  - installed `ecaz.so` SHA-256:
    `58a2af361807a98b8ec37dd9ad0f32b15bf4738539273915ea0513078550dfe2`
- `cli-release-build.log`
  - command: `cargo build --release -p ecaz-cli`
  - result: pass; only the pre-existing unused `path` warning
  - log SHA-256: `e1670cca971610d2da76adfffb1c040c2bfb2ed0558a8135266a9bbd5154d2e9`
  - release CLI SHA-256:
    `9b67de7770016bf0f04c81f3b9c1fc64f0fc814b5aa8264cfc5141f3549ba1ec`

No profile result or optimization decision is claimed yet.
