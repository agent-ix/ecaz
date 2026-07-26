# Artifact Manifest

- Implementation HEAD: `4d9bbec47`
- Task bucket: `reviews/task-38/`
- Packet: `reviews/task-38/001-distann-remote-fault-expansion/`
- Current evidence capture: `2026-07-25 17:17 America/Los_Angeles`
- Fixture shape: isolated one-table/one-index fixtures; DistANN uses one
  fixture per neighbor-code format
- Benchmark matrix: not applicable; this checkpoint changes only
  test/operator fault planning and fixture SQL

## Focused validation

Unless a subsection says otherwise, the final-suffixed, live, matrix, cgroup,
formatting, install/restart, and final-status artifacts below were captured at
the current evidence timestamp against implementation HEAD `4d9bbec47`.
Earlier foundation logs remain provenance for their named checkpoint and are
superseded where a final-suffixed log exists.

### `fault-injection-tests-final.log`

- Command: `cargo test -p ecaz-fault-injection`
- Key result: `9 passed; 0 failed`, including five-AM coverage, all three
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

### Final focused CLI parser logs

- `ecaz-cli-distann-parsing-final.log`
  - Command: `cargo test -p ecaz-cli cli_parses_distann`
  - Result: `2 passed; 0 failed`; covers DistANN smoke and cgroup planning.
- `ecaz-cli-socket-parsing-final.log`
  - Command:
    `cargo test -p ecaz-cli cli_parses_fault_socket_provider_env_command`
  - Result: `1 passed; 0 failed`.
- `ecaz-cli-slow-disk-parsing-final.log`
  - Command:
    `cargo test -p ecaz-cli cli_parses_measured_slow_disk_smoke_command`
  - Result: `1 passed; 0 failed`.

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

### `full-fault-matrix-dry-run.log`

- Command: `target/debug/ecaz --log-file <artifact> dev fault plan`
- Shape: four original single-index fixtures plus three codec-specific
  DistANN single-index fixtures.
- Key result: all seven fixtures appear in every applicable lane; 42 DistANN
  cases are printed.

### `make-fault-full-dry-run.log`

- Command: `make fault-full`
- Mode: default `FAULT_SMOKE_FLAGS=--dry-run`; this is operator/Make coverage,
  not a live aggregate or CI/nightly claim.
- Key result: every local Make lane completes and includes all three DistANN
  codecs.

## Live PG18 DistANN

All live commands used database `ecaz_fault_task38`, Unix socket
`/Users/peter/.pgrx`, port 28818, and isolated one-index-per-table fixtures.
The extension was installed with the repository operator helper; see
`ecaz-pg18-install.log` and `pg18-preload-restart.log`.

### `distann-all-codecs-cancel-live.log`

- Command:
  `target/debug/ecaz ... dev fault smoke --lane cancel --am distann --rows 16`
- Phase/fault: repeated real codec KNN work; `pg_cancel_backend` and
  `pg_terminate_backend`.
- Key result: all six cases pass; fixture pins are zero; I/O and WAL counters
  are nondecreasing.

### `distann-all-codecs-timeout-live.log`

- Command:
  `target/debug/ecaz ... dev fault smoke --lane timeout --am distann --rows 16`
- Phase/fault: repeated real codec KNN work; statement timeout and
  idle-in-transaction timeout.
- Key result: all six cases pass with zero fixture pins and clean accounting.

### `distann-all-codecs-lock-timeout-live.log`

- Command:
  `target/debug/ecaz ... dev fault smoke --lane lock-timeout --am distann
  --rows 16`
- Phase/fault: concurrent reindex, create-index, and vacuum-full ownership
  conflicts.
- Key result: all codec fixtures pass and shared postconditions are clean.

### `distann-all-codecs-resource-live.log`

- Command:
  `target/debug/ecaz ... dev fault smoke --lane resource --am distann
  --rows 16`
- Phase/fault: 4096-row codec fixtures; target 1000 accumulator pressure,
  tiny work/maintenance memory, temp-file limit, insert/vacuum, and forced WAL
  rotation.
- Key result:
  - RaBitQ: `returned=1000 returned_fraction_ppm=1000000`
  - TurboQuant: `returned=999 returned_fraction_ppm=999000`
  - grouped PQ: `returned=1000 returned_fraction_ppm=1000000`
  - final fixture pins: zero; I/O/WAL totals nondecreasing.

### `final-pg18-status-cleanup.log`

- Command: installed `ecaz dev sql` with packet-local `--log-output`.
- Key result: PostgreSQL 18.3, `shared_preload_libraries=ecaz`,
  `pg_is_in_recovery=false`, zero fault sessions, zero fault locks, zero
  prepared transactions, and `ecaz.fault_palloc_nth=-1`.

## Formatting

### `cargo-fmt-all-check.log`

- Command: `cargo fmt --all -- --check`
- Result: fails on the refreshed upstream base, beginning with untouched
  `crates/ecaz-cli/src/commands/corpus/load.rs` and continuing across many
  production files. No unrelated mass formatting was applied.

### `modified-rustfmt-check.log`

- Command:
  `rustfmt --edition 2021 --check crates/ecaz-cli/src/cli.rs
  crates/ecaz-cli/src/commands/dev/fault.rs
  crates/ecaz-fault-injection/src/lib.rs`
- Result: pass.

## Host

`host-capability.log` records macOS 26.4.1 / Darwin 25.4.0 on arm64.
`systemd-run` and `/sys/fs/cgroup/cgroup.controllers` are unavailable.
Consequently:

- Linux LD_PRELOAD provider execution: unavailable on this host.
- Exact-peer socket reset/slow live run: unavailable on this host.
- cgroup v2 user-scope OOM run: unavailable on this host.

No provider `fault=1` proof is claimed yet; it requires a Linux live run.

`cgroup-plan-current-host.log` records
`availability=unavailable linux=false cgroup_v2=false systemd_run=false` and
prints the seven host-independent scope cases. `socket-reset-env-current-host.log`
records the exact TCP peer filter and `LD_PRELOAD=<linux-only provider not
built>`. SPIRE remote SQL and DistANN owner/payload transport exist, but their
live socket-provider cases are unavailable on this host. SPIRE object-store
reads are separately classified as nonexistent.

## Reviewer verification artifacts (2026-07-25)

Added by `feedback/2026-07-25-01-reviewer.md`. PR #50 head `22bbe6284`,
verified in a clean worktree on Apple M5 (macOS 26.4.1, Darwin 25.4.0, arm64) —
the same host class as the packet, with the same LD_PRELOAD limits.

### `2026-07-25-reviewer-test-rerun.log`

- Commands: `cargo test -p ecaz-fault-injection`, `cargo test -p ecaz-cli fault`
- Status: PASS, reproducing the PR body's counts.
- Key result: 9 passed / 0 failed for the provider crate (including
  `socket_provider_environment_pins_exact_peer` and
  `all_lanes_cover_every_distann_codec_with_distinct_fixture_ids`); 30 passed /
  0 failed for the CLI fault parsers.

### `2026-07-25-reviewer-c-syntax-ceiling.log`

- Command: `clang -fsyntax-only` on `ldpreload_provider.c` at `origin/main` and
  at PR head.
- Status: 4 errors on both sides, all pre-existing Linux-only types (`off64_t`,
  implicit int) that this SDK cannot resolve.
- Key result: the diff introduces no NEW syntax errors. This is the ceiling of
  what an Apple host can establish about the C provider —
  `crates/ecaz-fault-injection/build.rs` returns early unless
  `CARGO_CFG_TARGET_OS == "linux"`, so the provider is not compiled here, and
  repository CI is `workflow_dispatch`-only, so it has not been compiled there
  either. The new socket code is therefore uncompiled on every host, not merely
  unexecuted on this one.

## Reviewer-flag follow-up (2026-07-25)

### `2026-07-25-reviewer-flags-local-validation.log`

- Implementation base: PR #50 reviewer head `df063bf83`; follow-up changes
  are the local commits merged with that reviewer head.
- Commands: `cargo check -p ecaz-cli`,
  `cargo test -p ecaz-fault-injection`, `cargo test -p ecaz-cli fault`, and
  `git diff --check`.
- Key results: check PASS; fault crate `9 passed; 0 failed`; CLI fault filter
  `31 passed; 0 failed`; diff check PASS.
- Change coverage: stable named-Unix/TCP peer validation with unnamed and
  abstract Unix peers rejected; scalar, vectored, datagram, and message socket
  entry points; `errno` preservation; unrepresentable invalid DistANN fixture
  state; Linux provider compilation enables `-Wall -Wextra`.
- Linux compile evidence: manual workflow run `30183037880`, PG18 job
  `89742835246`, head `18bf5e248`. Its `Build operator CLI` step passed on
  Ubuntu 24.04 x86_64. Since `ecaz-cli` directly depends on
  `ecaz-fault-injection`, the Linux-only provider build script completed with
  `-Wall -Wextra`.
- Evidence ceiling: CI did not load or exercise the provider. A focused TCP
  reset against a real peer and network syscall trace still require a Linux
  runtime.

### `2026-07-25-linux-provider-compile-ci.log`

- Source: GitHub Actions run `30183037880`, job `89742835246`.
- Key result: `Build operator CLI` completed successfully at exact head
  `18bf5e248`; the full PG18 job also completed successfully.
- Scope: Linux x86_64 compilation only. No provider runtime claim.

### `2026-07-25-reviewer-reverify.log`

- Added by `feedback/2026-07-25-02-reviewer.md`. Head `adb808b93`, Apple M5
  macOS arm64.
- Commands: `cargo test -p ecaz-fault-injection`, `cargo test -p ecaz-cli fault`,
  `clang -fsyntax-only -Wall -Wextra` on the provider.
- Status: PASS.
- Key result: 9 passed / 0 failed provider; 31 passed / 0 failed CLI fault
  (up from 30 at seq 01); 0 non-deprecated warnings under `-Wall -Wextra`,
  which is the evidence that adding `-Werror` to `build.rs` is now low-risk.
- Independent check of the cited compile evidence: CI run `30183037880` reports
  `headSha=18bf5e248`, `event=workflow_dispatch`, job `pgrx pg18` success. The
  provenance chain holds because `crates/ecaz-cli/Cargo.toml:41` declares a path
  dependency on `ecaz-fault-injection`, so that job's `cargo build -p ecaz-cli`
  runs the fault crate's `build.rs`. HEAD `adb808b93` changes artifacts only, so
  the compile covers the exact current source.
