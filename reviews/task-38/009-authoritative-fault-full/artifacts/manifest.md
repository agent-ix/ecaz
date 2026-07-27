# Task 38 Packet 009 Artifact Manifest

- Code checkpoint: `addeb885ae46e556b340dcdb68e02cdb57955d89`
- Task bucket: `reviews/task-38/`
- Packet: `009-authoritative-fault-full`
- Host: local Apple M5, macOS `26.4.1`, `arm64`
- PostgreSQL target: PG18
- Run timestamp: `2026-07-27T05:47:16Z`
- Remote/AWS/CI/nightly/Docker/Intel execution: none
- Fixture isolation: aggregate plan requires one index per fixture table;
  live fixture execution was not attempted on macOS

## Artifacts

### `fault-full-plan.log`

- Command:
  `target/debug/ecaz --log-file
  reviews/task-38/009-authoritative-fault-full/artifacts/fault-full-plan.log
  dev fault full --dry-run --artifact-dir
  reviews/task-38/009-authoritative-fault-full/artifacts/live-evidence
  --runtime-dir target/task38-packet009-runtime`
- Exit: `0`
- Lines: `118`
- SHA-256:
  `69d6fba47b2167a667585e040c7dfdb15d9e80d551aee44a48a057c823044ddd`
- Key result:
  `full_plan cases=116 fixtures=7 provider_cases=56
  remote_socket_cases=4 cgroup_cases=7 dry_run=true`
- Final marker:
  `full_plan_complete live_authority=false reason="dry-run planning only"`

### `plan-counts.log`

- Command: packet-local `awk` audit over every `full_case` line in
  `fault-full-plan.log`.
- Exit: `0`
- Lines: `8`
- SHA-256:
  `b1a9ebc9212b6456b63d7e1048179611432b62bff3e0c2ecd8c2564a761ec4aa`
- Key results:
  - `cases=116`
  - `unique=116`
  - `duplicates=0`
  - `local=35`
  - `mutation=14`
  - `provider=56`
  - `remote_socket=4`
  - `cgroup=7`

### `m5-live-preflight.log`

- Command:
  `target/debug/ecaz --log-file
  reviews/task-38/009-authoritative-fault-full/artifacts/m5-live-preflight.log
  dev fault full --artifact-dir
  reviews/task-38/009-authoritative-fault-full/artifacts/live-evidence
  --runtime-dir target/task38-packet009-live-runtime`
- Exit: `1` (expected host rejection)
- Lines: `125`
- SHA-256:
  `f10af636444e06adc9f1e1076ad6cbb8bc2ead877cc6762dfe5ec81b826c7d65`
- Key results:
  - The full 116-case matrix is printed with `dry_run=false`.
  - The command rejects the host with:
    `live fault-full requires Linux LD_PRELOAD, cgroup v2, and a user systemd
    manager; use --dry-run on macos`.
  - Both requested roots remained absent after exit:
    `artifacts/live-evidence` and `target/task38-packet009-live-runtime`.

### `m5-build.log`

- Command:
  `cargo build -p ecaz-cli --color never --message-format short`
- Exit: `0`
- Lines: `27`
- SHA-256:
  `2ef84478f60fc390accae323f3445aa68065c2b4a74b7e64f3354a773643eae3`
- Key result:
  `Finished dev profile [unoptimized + debuginfo] target(s) in 20m 01s`.
- Diagnostic boundary: one existing warning at
  `crates/ecaz-cli/src/commands/corpus/load.rs:190` for unused field `path`;
  no Task 38 build warning.

### `fault-model-tests.log`

- Command:
  `cargo test -p ecaz-fault-injection --color never --message-format short`
- Exit: `0`
- Lines: `21`
- SHA-256:
  `3b5edcb6e1c45f9833e86b04ade3ad30e51f3fb25a30afffeba911e8fe9f77aa`
- Key result:
  `9 passed; 0 failed`.
- Architecture boundary: the new slow-disk exact-path LD_PRELOAD regression
  test and the pre-existing provider syscall tests are `target_os = "linux"`
  gated. They compile for the Linux target only and were not executed on M5.

## Additional Local Validation

- Nightly `rustfmt` was applied to both changed Rust files.
- `git diff --check` passed.
- `cargo check -p ecaz-cli --tests --message-format short` passed in `8m16s`.
- `cargo clippy -p ecaz-fault-injection --all-targets -- -D warnings` passed.
- `cargo clippy -p ecaz-cli --tests` exited `0`. Its full-repository warning
  set contained pre-existing MSRV, complexity, and unrelated-module warnings;
  the two new aggregate warnings were corrected before the final build.

## Evidence Boundary

This packet proves source compilation, host-independent matrix construction,
case uniqueness/counts, M5-safe preflight behavior, and M5-applicable unit
tests. It does not prove Linux LD_PRELOAD syscalls, DistANN/SPIRE socket
fault behavior, cgroup-v2 OOM behavior, Intel behavior, or live aggregate
completion. Those exact gates remain open and must be evidenced by running
`make fault-full` on the designated Intel/Linux host.
