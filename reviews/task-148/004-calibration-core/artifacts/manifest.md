# Task 148 Packet 004 Artifact Manifest

- head SHA: `cce65f951a41a0874ce566ff6dc2f7ef7466e22e`
- task bucket: `reviews/task-148/004-calibration-core`
- timestamp: 2026-07-05
- scope: Slice 3 calibration-core checkpoint only; no index AM wiring and no benchmark claim.

## Artifacts

### `cargo-test-calibration-core.log`

- sha256: `4e516969581bf3d5300071624e58501ca4c7c9753fc36610549df48f3fd98e90`
- command:

```sh
script -q reviews/task-148/004-calibration-core/artifacts/cargo-test-calibration-core.log cargo test --release --lib calibration_no_qjl_4bit_reduces_anisotropic_score_error
```

- key result lines:

```text
running 1 test
test quant::prod::tests::calibration_no_qjl_4bit_reduces_anisotropic_score_error ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2257 filtered out; finished in 0.01s
```

