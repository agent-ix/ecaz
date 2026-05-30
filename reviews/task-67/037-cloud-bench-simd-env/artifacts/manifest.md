# Task 67 Packet 037 Artifact Manifest

- head SHA: `bcd8e29c6073a3baff161cfd03e53dd238d44d04`
- task bucket: `reviews/task-67/037-cloud-bench-simd-env/`
- timestamp: `2026-05-30T16:32:46Z`
- lane: Task 67 cloud benchmark runner fix
- fixture / storage format / rerank mode: not a benchmark run; cloud SSM script generation fix for Task 67 SIMD benchmark evidence
- isolated one-index-per-table or shared-table surfaces: not applicable

## Code Change

- `crates/ecaz-cloud/src/commands/bench.rs`
  - `--simd-mode` already set `ECAZ_SIMD` in PostgreSQL's systemd environment.
  - This change also exports `ECAZ_SIMD` into the remote `ecaz bench suite` process.
  - This matters for Task 67 sidecar-rerank evidence because `bench sidecar-rerank` scores RaBitQ sidecar payloads in the CLI process, not only inside PostgreSQL.

## Validation

- `artifacts/cargo-fmt-check.log`
  - command: `cargo fmt --check`
  - result: exit 0; rustfmt emitted existing stable-toolchain warnings about unstable import grouping options.
- `artifacts/ecaz-cloud-simd-env-test.log`
  - command: `cargo test -p ecaz-cloud remote_suite_script_exports_simd_for_cli_and_postgres`
  - result: 1 passed, 0 failed.

## Re-run

After this commit, rerun Task 67 AWS sidecar suites with:

```bash
target/debug/ecaz cloud bench \
  --profile 10k-intel \
  --simd-mode scalar \
  --config reviews/task-67/<packet>/artifacts/<scalar-suite>.json \
  --suite <scalar-suite-name> \
  --database postgres \
  --ecaz-bin /usr/local/bin/ecaz

target/debug/ecaz cloud bench \
  --profile 10k-intel \
  --simd-mode auto \
  --config reviews/task-67/<packet>/artifacts/<auto-suite>.json \
  --suite <auto-suite-name> \
  --database postgres \
  --ecaz-bin /usr/local/bin/ecaz
```

The generated SSM script now includes both:

- `sudo systemctl set-environment ECAZ_SIMD='<mode>'`
- `export ECAZ_SIMD='<mode>'` before invoking `/usr/local/bin/ecaz bench suite run`
