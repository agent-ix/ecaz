# Artifact Manifest

- Implementation HEAD: `e94e3dace`
- Task bucket: `reviews/task-38/`
- Packet: `reviews/task-38/001-distann-remote-fault-expansion/`
- Fixture shape: isolated one-table/one-index fixtures; DistANN uses one
  fixture per neighbor-code format
- Benchmark matrix: not applicable; this checkpoint changes only
  test/operator fault planning and fixture SQL

## Focused validation

### `fault-injection-tests.log`

- Command: `cargo test -p ecaz-fault-injection`
- Key result: `8 passed; 0 failed`, including five-AM coverage, all three
  DistANN codecs, and the supported 1536-D TurboQuant fixture.

### `ecaz-cli-check.log`

- Command: `cargo check -p ecaz-cli`
- Key result: CLI and extension compile successfully; the log retains the
  pre-existing unused `LoadedDistributedPlacementConfig.path` warning.

### `ecaz-cli-distann-parse-test.log`

- Command:
  `cargo test -p ecaz-cli cli_parses_distann_fault_smoke_dry_run_command`
- Key result: `1 passed; 0 failed; 459 filtered out`.

### `distann-cancel-plan.log`

- Command:
  `cargo run -p ecaz-cli -- dev fault plan --am distann --lane cancel`
- Key result: six cases printed, covering cancel and terminate for RaBitQ,
  TurboQuant, and grouped PQ.

### `distann-grouped-pq-timeout-dry-run.log`

- Command:
  `cargo run -p ecaz-cli -- dev fault smoke --lane timeout --am distann
  --distann-codec grouped-pq --dry-run`
- Key result: grouped-PQ statement-timeout and idle-transaction timeout cases
  printed without connecting to PostgreSQL.

## Host

Host OS, architecture, and systemd/cgroup capability evidence will be added
with the live/capability checkpoint. Provider match configuration and
`fault=1` proof are not applicable to this model-only checkpoint.
