# Task 48: Build Matrix, Cross-Platform, Soak, and Resource Exhaustion

Status: **proposed** — combines the CI-matrix and long-running stability
work that Task 34 does not cover. ECAZ ships to multiple architectures
and PG versions; long-lived operator workloads behave differently from
short test runs.

## Scope

Four areas:

1. **Build matrix.** CI lanes for the full target / toolchain matrix:
   - `aarch64-apple-darwin` (developer default).
   - `x86_64-unknown-linux-gnu` (production primary).
   - `aarch64-unknown-linux-gnu` (Graviton production target).
   - `x86_64-unknown-linux-musl` (static binaries).
   - PG versions: pg17, pg18, and future pg19 when supported.
   - Rust: stable, nightly (for Miri / sanitizers / careful).
2. **Cross-arch decode lane.** Run on-disk format fixtures (Task 42)
   under `qemu-user` for an opposite-endian target (s390x or ppc64).
3. **Soak / longevity.** Long-running workloads that surface
   memory-leak, drift, and cumulative-error bugs short tests miss:
   - 24-hour mixed read/write workload.
   - Continuous index build → vacuum → reindex cycle.
   - Memory-context lifetime walk: assert no monotonic palloc growth
     across N iterations.
4. **Resource exhaustion.** Workloads designed to hit configured
   resource limits cleanly:
   - `max_locks_per_transaction` exhaustion under heavy parallel DDL.
   - `max_connections` exhaustion under burst.
   - `work_mem` / `maintenance_work_mem` minimum-config runs.
   - `temp_file_limit` reached during spill.
   - `shared_buffers` thrash under cache-cold workloads.
   - Disk full during build (pairs with Task 38 ENOSPC).

## Why

ECAZ is built on a developer laptop (aarch64-darwin) and deployed to
linux production. The current CI runs only one or two combinations;
silent failures on a non-tested combination ship.

- Toolchain matrix: pgrx + nightly + sanitizer combinations are fragile;
  a stable-only repo will catch the breakage but only at the moment a
  release is built. Matrix CI catches it on the introducing PR.
- Cross-arch: every test today runs on little-endian hardware. The qemu
  lane is the only realistic way to exercise BE decode paths.
- Soak: leaks under `palloc` are easy to miss in a 30-second pgrx
  test. The same workload across 24 hours shows monotonic growth
  immediately.
- Resource exhaustion: production clusters hit `max_locks_per_transaction`
  during big REINDEX windows. Catching the ERROR path cleanly is a
  customer-visible contract.

## Approach

1. **GitHub Actions / equivalent CI matrix.** A separate workflow per
   target that runs `ci-quick` and `hardening-local` (or the subset
   relevant to the target). Failures are blocking for the matching
   target.
2. **Qemu lane.** Use `cross` or a hand-rolled `qemu-aarch64` /
   `qemu-s390x` step in CI to compile-and-run the decode-only test
   subset. Document in `docs/cross-arch.md`.
3. **Soak harness.** Add `crates/ecaz-soak/` (or a CLI subcommand)
   that:
   - boots a local PG18 cluster with monitoring,
   - runs a configurable mixed workload for `--duration`,
   - tracks `pg_stat_io`, `pg_stat_database`, `pg_stat_activity`,
     `pg_buffercache_summary`, `RSS`, allocator metrics,
   - asserts no monotonic-growth signatures (slope > threshold on a
     linear fit over the second half of the run),
   - dumps a JSON summary to packet artifacts.
4. **Resource exhaustion harness.** A CLI subcommand
   `ecaz dev resource-test` that:
   - configures PG to a low limit per scenario,
   - runs a workload designed to hit it,
   - asserts a clean ERROR (no PANIC, no broken connection state),
   - verifies post-test cluster health.
5. **Cadence.**
   - Build matrix: per-PR for stable lanes; nightly for nightly /
     sanitizer lanes.
   - Qemu cross-arch: nightly.
   - Soak: weekly (or before release).
   - Resource exhaustion: nightly.
6. **Make lanes:**
   - `make ci-matrix-local` — runs the matrix on a developer machine
     via Docker / `cross`.
   - `make soak DURATION=24h` — runs the soak harness locally.
   - `make resource-exhaustion` — runs the exhaustion sweep.
   - `make endian-qemu` (shared with Task 42).

## Validation

- CI matrix shows green for every documented target on a no-op PR.
- Qemu lane decodes Task 42 fixtures correctly on the BE target.
- Soak harness produces a flat memory-use line over 24 hours; a
  deliberately leaked allocation (test fixture) is caught by the
  slope check.
- Resource exhaustion sweeps each return clean ERROR (no PANIC, no
  segfault) and pass post-condition health checks.

## Exit Criteria

- CI matrix covers at least: aarch64-darwin, x86_64-linux-gnu,
  aarch64-linux-gnu, pg17, pg18.
- `make soak DURATION=24h` runs weekly and the artifact lands in a
  packet.
- `make resource-exhaustion` runs nightly.
- `docs/build-matrix.md` documents the supported matrix, the cadence,
  and the policy for adding new targets.

## Dependencies

- Task 42 (on-disk format) provides the fixtures the qemu lane runs.
- Task 38 (fault injection) provides the ENOSPC / OOM injection used
  by resource-exhaustion.
- Task 49 (CI governance) decides which lanes block PRs vs. nightly.
- Independent of other proposed tasks otherwise.
