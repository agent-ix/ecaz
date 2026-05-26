# Build Matrix, Cross-Platform, Soak, And Resource Exhaustion

Task 48 specifies a build matrix wider than the per-PR fast lane, cross-arch
decode coverage, long-running soak workloads, and resource-exhaustion sweeps.
This document is the policy reference: which targets are supported, on what
cadence, and how to add a new one.

## Supported targets

| Target triple | PG versions | Toolchains | Cadence | Notes |
|---|---|---|---|---|
| `aarch64-apple-darwin` | pg18 | stable, nightly | per-PR | developer default; macOS `_BufferBlocks` dyld blocker applies to pgrx-backed runtime tests — validate via compile + Miri instead, see `feedback_dyld_buffer_blocks_known`. |
| `x86_64-unknown-linux-gnu` | pg17, pg18 | stable, nightly | per-PR | production primary; runs the full `hardening-local` + `hardening-nightly-local` lanes. |
| `aarch64-unknown-linux-gnu` | pg18 | stable | per-PR (compile) + nightly (full) | Graviton production target; full lane uses GitHub Actions arm64 runners. |
| `x86_64-unknown-linux-musl` | pg18 | stable | nightly | static binaries; ecaz-cli only — pgrx extension does not link statically. |
| `s390x-unknown-linux-gnu` (qemu-user) | n/a | stable | nightly | big-endian fixture decode lane only — see [On-Disk Format Cross-Arch](#on-disk-format-cross-arch). |

PG version policy:
- pg18 is the validation primary. Every per-PR lane must run pg18.
- pg17 is compatibility coverage. Per-PR lanes may skip pg17 unless the
  change touches PG17-facing code; nightly lanes run pg17 on linux-gnu.
- pg19 lands in this table when the upstream RC is published.

Rust toolchain policy:
- `stable` runs every lane.
- `nightly` runs `miri`, `cargo-careful`, sanitizers, and `cargo-fuzz`.
  Pinned to the toolchain recorded under `~/.rustup/toolchains/`.

## Cadences

| Cadence | Driver | Make/script | Failure policy |
|---|---|---|---|
| per-PR | GitHub Actions | `make ci-quick`, `make hardening-local` | blocking |
| nightly | GitHub Actions `schedule` | `make ci-matrix-local`, `make hardening-nightly-local`, `make endian-qemu`, `make resource-exhaustion` | blocks merge to main; auto-issue on first failure |
| weekly | GitHub Actions `schedule` | `make soak DURATION=24h`, `make fuzz-cross-pollinate` | informational; deviation requires a follow-up packet |
| pre-release | manual | `make ci-matrix-local`, plus the weekly soak artifact from the most recent run | blocking for the release |

## Soak

`make soak DURATION=24h` runs `ecaz stress soak-quant-cache` and, when later
slices land them, the PG-backed mixed-workload harness, against a local PG18
cluster (or pure-Rust where the harness is PG-free). Artifacts land in a
weekly packet under `reviews/task-48/{NNN}-soak-{date}/artifacts/`. The
slope-fit RSS check is the leak gate; deviation beyond `--slope-tolerance`
(default 1 KiB/iter) blocks the next release.

`make soak DURATION=…` accepts any duration that the underlying Rust
`humantime` parser handles (`1h`, `24h`, `1d`, etc.). The default short
soak in `hardening-nightly-local` uses `DURATION=300s`.

## Resource exhaustion

`make resource-exhaustion` runs `ecaz dev resource-test` over every
configured scenario:

| Scenario | What it stresses | Expected disposition |
|---|---|---|
| `max-locks` | `max_locks_per_transaction` under heavy parallel DDL | clean `ERROR`, no PANIC, cluster healthy |
| `max-connections` | `max_connections` under burst | clean `ERROR`, no PANIC |
| `work-mem-min` | `work_mem` / `maintenance_work_mem` at minimum | builds succeed or fail with clean ERROR |
| `temp-file-limit` | `temp_file_limit` reached during spill | clean ERROR; spill file cleaned up |
| `shared-buffers-thrash` | cold cache + random scan | no segfault; post-test buffer-cache health passes |
| `disk-full` | injected ENOSPC via Task 38 fault-injection | clean ERROR, no torn pages |

Each scenario asserts both the negative result (ERROR class, no PANIC, no
broken connection state) and the post-condition (cluster health, page
consistency, no leaked temp files).

## On-Disk Format Cross-Arch

Cross-arch decode coverage uses `qemu-user` to exercise the Task 42
on-disk-format fixtures on a big-endian target without requiring physical
hardware:

```sh
make endian-qemu QEMU_TARGET=s390x-unknown-linux-gnu
```

The lane is compile-and-run; the BE binary loads the LE-canonical fixtures
under `fixtures/m5_*` and asserts roundtrip equality of every typed page.
PPC64 is a future addition; s390x is sufficient for the BE branch coverage.

## Adding a new target

A new entry in the table at the top requires:

1. A review packet under `reviews/task-48/{NNN}-add-target-{triple}/` with
   manifest, command transcripts, and a green run on the chosen lane
   subset.
2. A GitHub Actions workflow file under `.github/workflows/` named
   `build-matrix-{triple}.yml` invoking the chosen lane. Per-PR additions
   block on the workflow being passing on `main` before the table row is
   marked `per-PR`.
3. An entry in this document recording the lane the new target runs and
   the cadence. Add the row to the policy table, do not bury it in prose.
4. If the new target requires a non-default toolchain, install steps go
   into `scripts/install_hardening_tools.sh` and a brief note here, not
   into the workflow file itself.

## Removing or downgrading a target

Removing or downgrading a target cadence (e.g. moving a per-PR target to
nightly only) requires the same review packet structure plus an explicit
note in the table cell on the next-up release notes. Do not silently drop a
lane.

## Cross-references

- Task 48 spec: `plan/tasks/48-build-matrix-and-soak.md`
- Task 42 (on-disk format fixtures): see fixtures under `fixtures/m5_*`
- Task 38 (fault injection): `make fault-full`
- Task 49 (CI governance): which lanes block PRs vs nightly is decided
  there, applied here.
- Macos dyld blocker policy: memory `feedback_dyld_buffer_blocks_known`
  and `feedback_macos_cli_pg_static_stubs`.
