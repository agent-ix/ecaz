# Task 167 packet 032 artifacts

- Task bucket: `reviews/task-167/`.
- Packet: `032-recovery-runtime`.
- Status: first synthetic diagnostic failed before its concurrency wave; no
  closeout result claimed.
- Suite config: `task167-recovery-suite.json`.
- Lane: local Intel PG18, three physical owners, one index per owner table and
  a single-index control.
- Storage format: RaBitQ neighbor codes with exact owner-routed payload rerank.
- Real-corpus reloptions: shipped defaults `graph_degree=32` and
  `head_index_cap=4096`; BW4/H100 retained production traversal posture.
- Scales: staged `ec_real_10k`, `ec_real_50k`, and `ec_real_100k` under
  `data/staged-current`; corpus/query TSVs are not packet artifacts.
- Stress fixture: 2k rows, dimension 4, degree 8, three owners.
- External run directories: dated Task 167 directories under
  `/home/peter/.ecaz/clusters/`, never under the repository or Cargo target.
- Execution policy: production extension built with
  `--release --no-default-features --features pg18`; no `pg_test` or debug
  override; preliminary results are diagnostic and the final matrix must use
  one exact SHA/config.

## Build and audit artifacts

- Exact head: `cdecb75e4bf02172010b65184095c682a1020704`.
- Release CLI SHA256:
  `7287f55c3aa5fe5d287dbacfbee54a941746c0e4dac139c75d6a37986d9a9c1d`.
- Installed PG18 `.so` SHA256:
  `a857283ce901f6b2e229a87904635a244f7808449d8910f0b2dde0c088f6a226`.
- CLI build: `build-cli.log` — passed; release profile.
- Extension install: `install-extension.log` — passed; release `.so`, features
  `pg18`, no default features.
- Suite audit: `suite-audit.log` — passed, four steps.

## Synthetic diagnostic 1

- Command:
  `/home/peter/.cargo-target/release/ecaz bench suite run --config reviews/task-167/032-recovery-runtime/artifacts/task167-recovery-suite.json --only concurrency-synthetic --artifact-dir reviews/task-167/032-recovery-runtime/artifacts/smoke-synthetic --manifest-output reviews/task-167/032-recovery-runtime/artifacts/smoke-synthetic/suite-manifest.json --results-output reviews/task-167/032-recovery-runtime/artifacts/smoke-synthetic/results.jsonl --log-file reviews/task-167/032-recovery-runtime/artifacts/smoke-synthetic/suite-run.log`.
- Preflight: `extension_git_sha=cdecb75e4bf02172010b65184095c682a1020704`,
  `extension_build_profile=release`, `extension_features=pg18`,
  `debug_override=false`, three unanimous nodes.
- Topology: three owners reached Ready and Published with exact row/record
  counts and zero non-owner/orphan rows.
- Failure: the first serving query raised
  `ERROR: relation "ec_distann_retry_attribution" does not exist`; the suite
  marked `concurrency-synthetic` failed and left all real-corpus steps skipped.
- Interpretation: deterministic retry-attribution setup-order defect. The run
  did not reach the natural retry, liveness, backlink, or saturation checks.
