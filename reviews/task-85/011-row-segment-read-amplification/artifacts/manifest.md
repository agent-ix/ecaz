# Task 85 Packet 011 Artifact Manifest

- head SHA before checkpoint: `2308759c8eb38cab94641cb95a1d67f7487e7443`
- task bucket: `reviews/task-85/`
- packet path: `reviews/task-85/011-row-segment-read-amplification/`
- timestamp: `2026-06-07T16:03:39Z`
- lane: local harness instrumentation
- fixture: none
- storage format: SPIRE V2/V3/V4 leaf row segment diagnostics
- rerank mode: not applicable
- isolated/shared surface: not applicable

## Commands

```sh
script -q -e -c "cargo fmt --check" reviews/task-85/011-row-segment-read-amplification/artifacts/cargo-fmt-check.log
```

```sh
script -q -e -c "timeout 20 cargo test -p ecaz-cli spire_pipeline --locked --offline --no-run" reviews/task-85/011-row-segment-read-amplification/artifacts/cargo-test-ecaz-cli-spire-pipeline-no-run-timeout.log
```

```sh
script -q -e -c "ps -eo pid,ppid,stat,etime,comm,args | rg 'cargo test -p ecaz-cli spire_pipeline|cargo check -p ecaz|rustc' || true" reviews/task-85/011-row-segment-read-amplification/artifacts/cargo-process-status-after-timeout.log
```

## Artifacts

- `cargo-fmt-check.log`: formatting validation. Passed with existing stable
  rustfmt warnings about ignored nightly-only config keys.
- `cargo-test-ecaz-cli-spire-pipeline-no-run-timeout.log`: bounded compile
  validation attempt. Timed out before compiler output.
- `cargo-process-status-after-timeout.log`: process status after timeout; no
  lingering target validation cargo/rustc process beyond the status command
  itself.

## Validation Summary

Formatting passed. Focused compile/test validation is incomplete because Cargo
timed out before `rustc` in the current environment. Do not treat this packet
as AWS-ready until a later checkpoint reruns focused compile/tests.
