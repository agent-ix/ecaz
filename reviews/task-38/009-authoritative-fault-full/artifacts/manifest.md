# Task 38 Packet 009 Artifact Manifest

- Initial code checkpoint: `addeb885ae46e556b340dcdb68e02cdb57955d89`
- Review-response checkpoint:
  `c29c6dca5a05db33477bd87e390729b6f8c44642`
- M5 findings checkpoint:
  `147d44d05`
- DiskANN physical-page checkpoint:
  `a35d1cd71`
- Task bucket: `reviews/task-38/`
- Packet: `009-authoritative-fault-full`
- Host: local Apple M5, macOS `26.4.1`, `arm64`
- PostgreSQL target: PG18
- Initial run timestamp: `2026-07-27T05:47:16Z`
- M5 response run timestamp: `2026-07-28T07:36:54Z`
- Remote/AWS/CI/nightly/Docker/Intel execution: none
- Fixture isolation: one index per fixture table; the M5 partial aggregate
  executed the local and mutation phases against those isolated surfaces

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

### `slow-disk-threshold-test.log`

- Response timestamp: `2026-07-27T07:15:52Z`
- Command:
  `cargo test -p ecaz-cli
  slow_disk_threshold_accepts_boundary_and_rejects_underflow_or_overflow
  --color never --message-format short`
- Code checkpoint:
  `c29c6dca5a05db33477bd87e390729b6f8c44642`
- Exit: `0`
- Lines: `31`
- SHA-256:
  `26420f0afaf053745dd17c0da2f03b26bec1691b53193b81a94cef2ab0fb0c66`
- Key result:
  `1 passed; 0 failed; 471 filtered out`.
- Boundary cases executed:
  - provider time exactly equal to `baseline + configured latency`: pass;
  - provider time one millisecond below the threshold: rejected; and
  - `u128` checked-add overflow while forming the threshold: rejected.

### `m5-mutation-control-postfindings.log`

- Command: `make fault-mutation-control` against local PG18 at
  `/Users/peter/.pgrx`, port `28818`, database `ecaz_fault_task38`.
- Exit: `0`
- Lines: `58`
- SHA-256:
  `a6296963a6f6611e16fd9c6c40966a1d68727f187efdec057818991ec92a9fd8`
- Key result:
  `mutation_control_complete kind=All fixtures=7 clean_postconditions=true`.
- Pgstat result: every mutation gate reports numeric
  `pg_stat_io_ops_before/after` and `pg_stat_wal_records_before/after`; there
  are no false `unavailable` or `baseline_absent` markers.

### `m5-install-postfindings.log`

- Command:
  `/Users/peter/dev/tqvector/target/debug/ecaz --log-file
  reviews/task-38/009-authoritative-fault-full/artifacts/m5-install-postfindings.log
  dev install ecaz-pg-test --pg 18 --pgrx-home /Users/peter/.pgrx`
- Exit: `0`
- Lines: `6`
- SHA-256:
  `334f7bd8fe3e3dc937416327a49dd6e28f90e8a0c388b24d2c6781f4cce1f575`
- Installed backend:
  `/opt/homebrew/lib/postgresql@18/ecaz.dylib`
- Installed SHA-256:
  `75f1462b19a54a38ba10a18ea2df1045c2b10fcff5fc735dc2c9c38e086721f2`

### `m5-partial-live.log`

- Command:
  `make fault-full FAULT_DATABASE=ecaz_fault_task38
  FAULT_HOST=/Users/peter/.pgrx FAULT_PORT=28818 FAULT_ROWS=16
  FAULT_FULL_LOG_FILE=reviews/task-38/009-authoritative-fault-full/artifacts/m5-partial-live.log
  FAULT_FULL_ARTIFACT_DIR=reviews/task-38/009-authoritative-fault-full/artifacts/m5-partial-live
  FAULT_FULL_RUNTIME_DIR=target/task38-p009-m5-partial-runtime`
- Exit: `0`
- Lines: `542`
- SHA-256:
  `5ab004905f2987e7d6e5d55d0741389ae8798cd67d21dc88d4590e07aae29e74`
- Isolation: one index per fixture table; runtime state under the named
  `target/` root; durable logs under this packet.
- Key results:
  - `full_phase_skipped phase=provider reason=linux-only cases=56`
  - `full_phase_skipped phase=remote-socket reason=linux-only cases=4`
  - `full_phase_skipped phase=cgroup reason=linux-only cases=7`
  - `mutation_control_complete kind=All fixtures=7
    clean_postconditions=true`
  - `full_no_panic logs_scanned=1 result=pass failures=0`
  - `full_complete live_authority=partial executed=49 skipped=67 cases=116
    fixtures=7 no_panic=true shared_postconditions=true`
- Pgstat result: numeric baselines and final values are present throughout;
  crash-recovery resets are explicitly recognized for counters that reset
  after SIGKILL recovery.

### `m5-partial-live/main-postmaster.log`

- Source: main PG18 postmaster log delta captured unconditionally by
  `fault-full`.
- Lines: `1546`
- SHA-256:
  `117e8c69243bdd5245ecbf0a9822ddb97638f9d5d7b51ea159d9866fa33b83f9`
- Key result: all AM build/scan/insert SIGKILL probes recover the postmaster;
  the final no-panic audit found no `PANIC:` marker.

### `m5-partial-live/no-panic-audit.log`

- Lines: `3`
- SHA-256:
  `ef0a0e93c48953f603ce7e30247bc77d6afd3ad778bfb8522728243dfdde8a50`
- Key result: `postmaster_logs_scanned=1`, `result=PASS`,
  `pattern=PANIC:`.

### `diskann-physical-page-materialization-test.log`

- Command:
  `cargo test -p ecaz data_page_materialization_ --lib --color never`
- Exit: `0`
- Lines: `31`
- SHA-256:
  `28365106284aa93c2fe2fed43bbc3b3fc57b56af67da910b15c019659c63cf31`
- Key result: `2 passed; 0 failed; 2522 filtered out`.
- Coverage: physical unused-line-pointer offsets remain stable, and
  materialization uses PostgreSQL-equivalent `PageAddItem` accounting.

## Additional Local Validation

- Nightly `rustfmt` was applied to both changed Rust files.
- `git diff --check` passed.
- `cargo check -p ecaz-cli --tests --message-format short` passed in `8m16s`.
- `cargo clippy -p ecaz-fault-injection --all-targets -- -D warnings` passed.
- `cargo clippy -p ecaz-cli --tests` exited `0`. Its full-repository warning
  set contained pre-existing MSRV, complexity, and unrelated-module warnings;
  the two new aggregate warnings were corrected before the final build.
- Stable `rustfmt --check` passed for all Rust files changed by the M5
  response (with only the repository's nightly-option warnings).
- `cargo check -p ecaz-cli -p ecaz-fault-injection` passed before the live
  aggregate; the only warning was the existing unused `corpus/load.rs:path`.
- The authoritative `make fault-full` invocation rebuilt both `ecaz` and
  `ecaz-cli` successfully before executing the live phases.

## Evidence Boundary

This packet proves source compilation, host-independent matrix construction,
case uniqueness/counts, both mutation controls across seven fixtures, and live
completion of all 49 Apple-M5-compatible aggregate cases with shared
postconditions and no-panic audit. It does not prove Linux LD_PRELOAD syscalls,
DistANN/SPIRE socket fault behavior, cgroup-v2 OOM behavior, or Intel behavior.
Those 67 exact gates remain open and must be evidenced by running
`make fault-full` on the designated Intel/Linux host.
