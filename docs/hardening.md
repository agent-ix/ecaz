# Hardening Lanes

Task 34 keeps new hardening local-first until each lane is reproducible and
low-noise. The Makefile is the entrypoint; optional tools are checked by
`scripts/hardening.sh` so missing tools fail with setup text instead of cargo
subcommand errors.

Reusable optional-tool setup lives outside the repository:

```sh
bash scripts/install_hardening_tools.sh --all
bash scripts/install_hardening_tools.sh --check
```

The installer keeps upstream source checkouts under `~/.ecaz/hardening-tools`
and puts reusable binaries or shims on the normal user tool path. Use
`--log-file reviews/task-{id}/001-topic/artifacts/name.log` when an install/update log needs to
be attached to a review packet.

## Aggregates

- `make hardening-local` runs stable local checks that do not need a live
  cluster: format check, PG18 Clippy with the current repository lint baseline,
  CLI unit tests, a standalone pure Rust extension harness, property tests,
  SIMD/scalar differential tests, layout assertions, unsafe comment audit, full `cargo-deny`, and
  `cargo-audit`.
- `make hardening-nightly-local` adds slower and toolchain-sensitive local
  lanes: expanded Miri, `cargo-careful`, fuzz smoke, PG fault-injection dry-run
  coverage, Kani, and pure Rust
  ASan/LSan. `cargo-geiger` remains a standalone reporting lane because it can
  force a large clean rebuild.
- `make hardening-validate` checks that local hardening crates import real
  ECAZ `src/` code and that retired synthetic-only lanes have not reappeared.
- `make hardening-tiers-report` prints the current lane tier inventory. See
  `docs/hardening-governance.md` for promotion and demotion rules.

## Baseline

- `make fmt-check`
- `make lint`
- `make lint-hardening`
- `make test`
- `make test-local`
- `make test-hardening-local`
- `make pg-test`
- `make proptest`
- `make simd-diff`
- `make layout-check`
- `make audit-unsafe`
- `make miri`
- `make fuzz-all-short`
- `make fault-full`
- `make deny-full`
- `make bench`
- `make bench-iai`

PG18 remains the primary validation target. PG17 compatibility checks stay
manual unless a change is PG17-facing.

## Supply Chain

- `make cargo-audit`: install with `cargo install cargo-audit`.
- `make deny-full`: install with `cargo install cargo-deny`.
- `make cargo-vet`: install with `cargo install cargo-vet`, then initialize
  `supply-chain/config.toml` with `cargo vet init`.

Promotion plan: `cargo-audit` and full `cargo-deny check` can become PR gates
after local burn-in. `cargo-vet` remains report/manual until third-party audit
imports and criteria are reviewed.

## Unsafe And Static Hygiene

- `make cargo-geiger`: install with `cargo install cargo-geiger`.
- `make mirai`: runs the archived MIRAI analyzer against
  `hardening/careful/` with its pinned nightly toolchain.

Policy: new unsafe blocks need a nearby `SAFETY` comment and, when the unsafe
surface is non-trivial, the review packet should call out why the boundary is
valid. MIRAI stays standalone/manual while its false-positive profile is
unknown for pgrx-heavy code. Rudra and Flux return only when they exercise real
ECAZ code instead of synthetic harness crates.

### Unsafe Quality Burndown

Task 34 introduced `scripts/unsafe_comment_baseline.txt` only as a temporary
grandfathering mechanism so new checks could land without hiding new debt. Task
35 owns burning that baseline down to zero.

Use `make unsafe-baseline-report` before and after each unsafe-burndown packet.
Review packets should cite the before/after counts, include the raw report logs
under packet-local artifacts when making count claims, and explain whether each
covered unsafe was removed, wrapped behind a safer boundary, or documented with
a specific invariant. Baseline growth is a blocker unless the packet calls out
the temporary exception and a reviewer accepts it.

## Miri And Cargo-Careful

- `make miri-expanded`: runs the expanded `miri_` pure-Rust test set through
  the repo hardening script with Miri's default aliasing model.
- `make miri-tree`: reruns the same `miri_` prefix with
  `-Zmiri-tree-borrows` so Tree Borrows and the default model can be compared
  in packet-local evidence.
- `make miri-many-seeds`: reruns the `miri_` prefix with
  `-Zmiri-many-seeds=128` by default. Override the count with
  `MIRI_MANY_SEEDS=N` when a packet needs a smaller triage run or a deeper
  campaign.
- `make miri-full`: runs the default, Tree Borrows, and many-seeds Miri lanes.
  `hardening-nightly-local` uses this aggregate.
- `make careful`: runs a standalone pure-Rust harness under
  `hardening/careful/` so PostgreSQL callback symbols are kept out of the
  `cargo-careful` test binary.

Miri and Kani cover only pure Rust paths. pgrx, SPI, libpq, PostgreSQL memory
contexts, and C callback entrypoints are outside their model and must stay in
PG18 pgrx or live-cluster lanes.

Review packets for Miri depth work should keep the default, Tree Borrows, and
many-seeds logs separate. If Tree Borrows and the default model disagree, the
packet should identify the exact test, preserve both logs, and classify the
difference as likely UB, model-specific false positive, or a test harness
problem before promoting a new default.

Seeded Miri coverage now includes:

- storage `ItemPointer` and data-page chain behavior,
- DiskANN metadata encode/decode,
- SPIRE leaf V2 object metadata and segment invariants through existing
  in-module tests with `miri_` prefixes.

## SIMD/Scalar Differential Validation

- `make simd-diff`: runs `tests/simd_diff.rs` with the `bench` feature. The
  harness compares production-dispatched scoring/FWHT entry points against
  scalar-reference entry points in the same process, and also calls test-only
  AVX2/FMA or NEON entry points directly when the host supports them. This
  keeps backend pinning independent of `ECAZ_SIMD` process-global dispatch.
- GitHub Actions runs the same lane in a focused `simd-diff` job on
  `ubuntu-24.04` x64 and `ubuntu-24.04-arm` arm64 hosted runners so both AVX2
  and NEON coverage are PR-visible.
- Tolerances:
  - FWHT lanes: absolute/relative `1e-5`.
  - `score_ip_from_parts`: absolute/relative `1e-5`.
  - `score_ip_codes_lite`: absolute/relative `1e-5`.
  - AM source inner product (HNSW/DiskANN): absolute/relative `1e-4`
    because production SIMD may use fused multiply-add while the scalar
    reference accumulates with separate operations.

Tolerance changes require a review packet that explains the numeric reason.
The Miri scalar fallback remains useful for reference-path UB checks, but SIMD
correctness is owned by this differential lane.

## Fuzzing

- `make fuzz-all-short`: runs each libFuzzer target for `FUZZ_SECONDS`, default
  30 seconds. Override without environment prefixes:
  `make fuzz-all-short FUZZ_SECONDS=5`.
- Individual targets: `make fuzz-parse-text`, `make fuzz-unpack`,
  `make fuzz-element-decode`, `make fuzz-neighbor-decode`,
  `make fuzz-diskann-metadata`, `make fuzz-item-pointer`, and
  `make fuzz-vector-normalize`.
- `make afl-decoders`: builds the DiskANN metadata and `ItemPointer` decoder
  targets with AFL.rs for longer manual campaigns.

SQLsmith is live-cluster only:

```sh
make sqlsmith-pg18 SQLSMITH_DSN='postgresql://localhost/postgres'
```

Use a PG18 cluster with `ecaz` installed. Capture crashes and raw SQLsmith logs
under the relevant review packet before citing findings.

## Test Quality

Task 39 adds measurement lanes for the quality of existing tests:

- `make coverage`: checks for `cargo-llvm-cov`, runs the local pure-Rust
  hardening subset currently safe outside a live PostgreSQL backend
  (`ecaz-cli` and `hardening/careful`), and writes `summary.txt` plus
  `coverage.json` under `target/quality/coverage`.
- `make coverage-report`: same lane with an HTML report under
  `target/quality/coverage/html`.
- `make mutants MUTANTS_MODULE=src/quant/prod.rs`: checks for
  `cargo-mutants` and runs a bounded mutation campaign for one critical module.
- `make mutants-full`: runs the initial critical-module target list from Task
  39. This is weekly/manual until survivor volume is triaged.
- `make flake-hunt`: re-runs proptest and short fuzz targets across multiple
  seeds. Override with `FLAKE_HUNT_SEEDS=N` and
  `FLAKE_HUNT_FUZZ_SECONDS=N`.

Current interpretation:

| Lane | Gate Level | Artifact | Rule |
| --- | --- | --- | --- |
| `make coverage` | Report-first / PR candidate after burn-in | summary, JSON, optional HTML | No repository-wide threshold yet; touched production files should not drop by more than 2 percentage points once a baseline packet exists. |
| `make mutants` | Weekly/manual | cargo-mutants output directory | Each survivor is triaged as equivalent, test gap, or follow-up bug. |
| `make flake-hunt` | Nightly candidate | seed sweep log | Any nondeterministic failure or new fuzz crash becomes a follow-up with the seed and minimized input attached. |

The first coverage scope intentionally avoids claiming live pgrx callback
coverage. PG18 integration coverage is still a gap until instrumentation is
stable for `cargo pgrx test pg18` and the resulting logs are packeted.

## PG Fault Injection

- `ecaz dev fault plan`: prints the required Task 38 fault matrix for every
  ECAZ AM (`ec_hnsw`, `ec_ivf`, `ec_diskann`, `ec_spire`) and every lane.
- `make fault-io-smoke`, `make fault-mem-smoke`, `make fault-cancel-smoke`,
  `make fault-timeout-smoke`, `make fault-lock-smoke`,
  `make fault-resource-smoke`, and `make fault-slow-disk-smoke`: run the
  operator smoke entry points. They default to `FAULT_SMOKE_FLAGS=--dry-run` so
  local and nightly hardening can verify matrix coverage without a live
  injection provider.
- To run a live probe, clear the dry-run flag, for example:
  `make fault-timeout-smoke FAULT_SMOKE_FLAGS=`.
- `ecaz dev fault provider-env` prints the LD_PRELOAD environment for the
  built-in Linux provider. That provider can inject matched-path `EIO` reads,
  matched-path `ENOSPC` writes/creates/fsyncs, and slow-disk latency once the
  PG postmaster is started with the printed environment. Example:
  `make fault-provider-env FAULT_PROVIDER_MODE=slow-disk`.
- `ecaz dev fault provider-restart` and `ecaz dev fault provider-restore`
  wrap the local pgrx `pg_ctl restart` step so provider-backed lanes do not
  require hand-assembled `LD_PRELOAD` commands. Marker paths passed to
  `provider-restart` are made absolute before they are exported to the
  postmaster, so backend workers append to the same marker even after
  PostgreSQL changes its working directory.
- `ecaz dev fault prepare --rows N` creates the AM-specific fixtures before
  destructive provider modes are enabled. Live I/O smoke then runs with
  `--assume-prepared --provider-marker <marker>` against an `eio-read` or
  `enospc-write` provider-backed postmaster.
  Provider-backed smoke lanes require the same marker path via
  `--provider-marker` so they cannot pass against a normal postmaster.
- Live memory smoke uses the extension GUC `ecaz.fault_palloc_nth` and
  `ecaz_fault_reset_palloc_counter()` to raise a clean ERROR at instrumented
  AM memory-fault boundaries. The current smoke covers each AM's build,
  insert, and vacuum callback boundary, and sweeps the first few Nth allocation
  points for each AM scan workload.

The current live CLI smoke creates AM-specific fixtures for `ec_hnsw`, `ec_ivf`,
`ec_diskann`, and `ec_spire`, then directly exercises cancellation and
backend termination with repeated AM KNN scans, statement timeout with repeated
AM KNN scans, `idle_in_transaction_session_timeout` after each AM fixture is
touched inside an open transaction, lock timeout with
`REINDEX INDEX CONCURRENTLY`, `CREATE INDEX`, and `VACUUM (FULL)`, and
scan/insert/vacuum/resource settings on those fixtures.
Slow-disk runs the same AM-specific scan/insert/vacuum smoke against a
provider-backed postmaster and requires a non-empty provider marker. I/O smoke
uses prebuilt fixtures and checks one provider mode at a time: `eio-read`
expects clean ERROR from AM scan reads, while `enospc-write` expects clean
ERROR from AM writes. When the provider marker records `match=pg_wal`, the I/O
lane treats WAL-path ENOSPC as a crash-recovery surface: it records the backend
disconnect, prints `wal_enospc_provider_restore_required=true`, and expects the
operator to run `ecaz dev fault provider-restore`, whose fallback path performs
an immediate stop/start if fast restart cannot shut down the faulting
postmaster. Resource smoke prepares pressure-sized AM fixtures, runs high-limit
KNN scans under `work_mem = '64kB'` and `effective_cache_size = '1MB'`, emits
`resource_accumulator_pressure` markers with the prepared row count, requested
limit, and returned row count, then runs AM scan/insert/vacuum under tiny
`work_mem`/`maintenance_work_mem` settings and forces a temp-spill failure with
`temp_file_limit = '64kB'`, verifying the backend remains usable. When the
postmaster is restarted with an `enospc-write` provider whose marker records
`match=pgsql_tmp`, the resource lane instead disables `temp_file_limit` and
expects the temp-spill failure to come from provider-backed ENOSPC. The
provider appends `fault=1` marker lines with mode, operation, errno, count, and
target path when it actually injects EIO/ENOSPC, and provider-backed smoke
asserts `provider_fault_events ... count>0` for the configured match. The same
resource lane now performs AM-backed writes, forces `pg_switch_wal()`, and
emits `wal_rotation_accounting` markers proving WAL LSN advancement plus
non-decreasing `pg_stat_wal` record/byte counters after stats flush. Memory smoke
injects palloc failures at the
instrumented AM build/scan/insert/vacuum boundaries. Build, scan, insert, and
vacuum probes sweep `ecaz.fault_palloc_nth` from 1 through the smoke cap and
stop at the first successful Nth value, emitting `memory_palloc_sweep_fault`
and `memory_palloc_sweep_completed` markers so the log shows how many currently
instrumented palloc boundaries were covered. The lane verifies the backend
remains usable after each ERROR. Every lane
uses the shared post-condition probe inventory from `ecaz-fault-injection`:
leftover fault sessions, surviving locks, prepared transactions, optional
`pg_buffercache` fixture pin counts, optional `pg_stat_io` non-decreasing
operation counters, and optional `pg_stat_wal` non-decreasing WAL record/byte
counters. Resource temp-spill probes also print
`resource_temp_spill_accounting` markers from `pg_stat_database.temp_bytes` for
readable before/after accounting; temp-file-limit failures may abort before the
database temp-byte total advances, so the smoke asserts readability and
non-decreasing totals rather than byte-perfect attribution. Memory smoke also
caps a warmed backend's `RLIMIT_AS` during AM build work, expecting an
OOM-class ERROR or backend disconnect followed by a usable postmaster, then
SIGKILLs worker backends during AM build/scan/insert as an OOM-kill proxy and
waits for postmaster recovery. Those subcases are crash-recovery checks; lower
post-run `pg_stat_io` or `pg_stat_wal` totals are recorded as stats resets
after recovery rather than treated as monotonicity failures.

SPIRE remote transport faults reuse `ecaz dev spire-multicluster fault-pg18`.
The Stage E fixture scripts keep their PostgreSQL Unix sockets under a short
target-local socket directory derived from the run directory so descriptive run
ids do not exceed PostgreSQL's Unix socket path limit.

Current interrupt inventory:

- DiskANN build/scan paths call `maybe_check_for_interrupts()` from
  `src/am/ec_diskann/mod.rs`, including the scan loop and build/import loops in
  `src/am/ec_diskann/scan.rs` and `src/am/ec_diskann/routine.rs`.
- SPIRE remote candidate dispatch polls PostgreSQL interrupt and statement
  timeout flags in `src/am/ec_spire/coordinator/remote_candidates/dispatch.rs`.
- HNSW parallel build calls `pg_sys::ProcessInterrupts()` in
  `src/am/ec_hnsw/build_parallel.rs`.

Missing or newly discovered long-running loops should be added to this list
with either an interrupt check or a follow-up task.

## Concurrency And Formal Pilots

- `make kani`: bounded proof for `ItemPointer` decode length behavior.

Kani is intentionally separate from normal `cargo test` so the repo does not
acquire heavyweight model-checking dependencies on the default path. Flux,
Loom, and Shuttle lanes are deferred until Tasks 44 and 40 can point them at
real ECAZ invariants.

## Sanitizers

Pure Rust:

- `make sanitizer-asan`
- `make sanitizer-lsan`
- `make sanitizer-tsan`
- `make sanitizer-msan`

PG18/pgrx:

- `make sanitizer-pg18-asan`
- `make sanitizer-pg18-tsan`

Sanitizer runs require nightly Rust and platform support. PG18 sanitizer lanes
also require a pgrx-ready cluster; keep them nightly/manual until the cluster
setup is stable.
