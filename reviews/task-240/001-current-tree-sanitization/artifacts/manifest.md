# Task 240 packet 001 artifact manifest

- Task/packet: `task-240/001-current-tree-sanitization`
- Baseline: `23fb9b7ba1f0803be5dfc700d9865f80fbf60862`
- Implementation: `e393515c7c94e0f127bcbac633052ee457c0527e`
- Review-request checkpoint: this packet commit
- Scope: current tracked tree and future added diff lines
- Excluded: Git-history rewrite, hosted hidden refs/caches, and bulk
  normalization of unrelated legacy workstation paths

## Audit results

- Baseline files containing the identified internal scratch locator: 31.
- Implementation files containing that locator: 0.
- Baseline files containing the audited absolute workstation prefix: 6,396.
- Implementation files containing that exact prefix: 6,390.
- Added-lines guard across baseline to implementation: pass.
- Whitespace/diff validation: pass.

The sensitive locator and matching source text are intentionally absent from
this public packet. The audit reports counts and categories only.

## Functional validation

- `python3 scripts/tests/test_check_workstation_paths.py`: 3 tests pass.
- `python3 scripts/check_workstation_paths.py --diff-range
  23fb9b7ba1f0803be5dfc700d9865f80fbf60862...e393515c7c94e0f127bcbac633052ee457c0527e`:
  pass.
- `cargo fmt --all -- --check`: pass with existing stable-rustfmt warnings.
- Isolated `cargo build -p ecaz-cli`: pass with one existing dead-code
  warning for `LoadedDistributedPlacementConfig.path`.
- `cargo test -p ecaz-cli commands::dev::install`: compile pass; 546 tests
  filtered out and zero selected.
- Freshly built CLI with each repository variable unset:
  - `ecaz dev install pgvector --pg 18`: exit 2; `--repo` required.
  - `ecaz dev install vectorscale --pg 18`: exit 2; `--repo` required.

No installer runtime path was reached during the missing-argument checks.

## Known baseline test-runner failures

`scripts/tests/run.sh` reports:

- three failures in
  `scripts/tests/test_pg17_scratch_psql_socket_resolution.py`;
- four errors in
  `scripts/tests/test_resolve_scratch_socket_dir.py`.

Both groups arise because the unchanged baseline references already-deleted
scripts. They are outside Task 240 and are recorded without being reclassified
as a Task 240 regression.

## Generated artifact cleanup

The isolated Cargo target directory used for the behavioral CLI checks was
removed after validation. It contained generated build output only; no tracked
file or installed extension was removed.
