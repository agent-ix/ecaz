# Task 68 Packet 001 Artifact Manifest

- head SHA: `318641d6fa091291fb07f52bfdb30958b8facad8`
- task bucket: `reviews/task-68/001-spire-build-timing-notices`
- timestamp: `2026-05-30T03:44:29Z`
- lane: Task 68 SPIRE build characterization prerequisite
- fixture/storage/rerank: compile and static audit only; no fixture storage format or rerank mode
- isolated one-index-per-table or shared-table surface: not applicable

## Artifacts

### `cargo-check-ecaz-lib-pg18.log`

- command: `cargo check -p ecaz --lib --no-default-features --features pg18`
- result: passed
- key line:
  - `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 0.08s`

### `common-training-call-audit.txt`

- command: `rg -n "common_training::" src/am/ec_spire/build src/am/ec_spire/update/materialization.rs src/am/ec_spire/update/routing.rs`
- result: static call audit captured
- key build-path calls:
  - `src/am/ec_spire/build/training.rs:12` single-level `train_spherical_kmeans`
  - `src/am/ec_spire/build/training.rs:20` single-level `assign_vectors_to_centroids`
  - `src/am/ec_spire/build/training.rs:156` relation build `train_spherical_kmeans`
  - `src/am/ec_spire/build/training.rs:172` relation build `assign_vectors_to_centroids`
  - `src/am/ec_spire/build/recursive.rs:40` recursive routing `train_spherical_kmeans`
  - `src/am/ec_spire/build/recursive.rs:54` recursive routing `assign_vectors_to_centroids`

### `install-current-extension.log`

- command: `/Users/peter/.cargo/bin/ecaz dev install ecaz-pg-test --pg 18 --log-file reviews/task-68/001-spire-build-timing-notices/artifacts/install-current-extension.log`
- result: passed
- key lines:
  - `[install] backend artifact assertion passed`
  - `[install] installed_backend=/opt/homebrew/lib/postgresql@18/ecaz.dylib`
  - `[install] sha256=aa2a9243c73f054295c4cbbac714036319108bc2da88c1b7d1c7e4bdeb3a4e47`
