# Artifact Manifest

- Implementation HEAD: `6ca901124`
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

### `ecaz-cli-socket-provider-parse-test.log`

- Command:
  `cargo test -p ecaz-cli cli_parses_fault_socket_provider_env_command`
- Key result: `1 passed; 0 failed; 460 filtered out`.

### `socket-provider-env-dry-run.log`

- Command:
  `cargo run -p ecaz-cli -- dev fault provider-env --mode socket-reset
  --peer-match tcp:127.0.0.1:39711 --after 3`
- Provider filter: exact peer `tcp:127.0.0.1:39711`; inject beginning on the
  third matched socket operation.
- Key result: environment includes `ECAZ_FAULT_PROVIDER_MODE=socket-reset`,
  `ECAZ_FAULT_PROVIDER_AFTER=3`, and
  `ECAZ_FAULT_PROVIDER_PEER=tcp:127.0.0.1:39711`.

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

`host-capability.log` records macOS 26.4.1 / Darwin 25.4.0 on arm64.
`systemd-run` and `/sys/fs/cgroup/cgroup.controllers` are unavailable.
Consequently:

- Linux LD_PRELOAD provider compilation/execution: unavailable on this host.
- Exact-peer socket reset/slow live run: unavailable on this host.
- cgroup v2 user-scope OOM run: unavailable on this host.

No provider `fault=1` proof is claimed yet; it requires a Linux live run.
