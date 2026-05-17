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
`--log-file review/.../artifacts/name.log` when an install/update log needs to
be attached to a review packet.

## Aggregates

- `make hardening-local` runs stable local checks that do not need a live
  cluster: format check, PG18 Clippy with the current repository lint baseline,
  CLI unit tests, a standalone pure Rust extension harness, property tests,
  SIMD/scalar differential tests, layout assertions, unsafe comment audit, full `cargo-deny`, and
  `cargo-audit`.
- `make hardening-nightly-local` adds slower and toolchain-sensitive local
  lanes: expanded Miri, `cargo-careful`, fuzz smoke, PG fault-injection dry-run
  coverage, Kani, Loom, Shuttle, and pure Rust ASan/LSan. `cargo-geiger` remains a standalone reporting lane
  because it can force a large clean rebuild.

## Baseline

- `make fmt-check`
- `make lint`
- `make lint-hardening`
- `make test`
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
- `make rudra`: runs the reusable Rudra Docker helper against
  `hardening/rudra/`. Override with `RUDRA_MANIFEST=...` for manual audits.
- `make mirai`: runs the archived MIRAI analyzer against
  `hardening/careful/` with its pinned nightly toolchain.
- `make flux`: runs Flux against `hardening/flux/`, a small
  dimension/index-invariant pilot harness.

Policy: new unsafe blocks need a nearby `SAFETY` comment and, when the unsafe
surface is non-trivial, the review packet should call out why the boundary is
valid. Rudra, MIRAI, and Flux stay standalone/manual rather than aggregate
targets while their false-positive profile is unknown for pgrx-heavy code.

## Miri And Cargo-Careful

- `make miri-expanded`: runs the expanded `miri_` pure-Rust test set through
  the repo hardening script.
- `make careful`: runs a standalone pure-Rust harness under
  `hardening/careful/` so PostgreSQL callback symbols are kept out of the
  `cargo-careful` test binary.

Miri and Kani cover only pure Rust paths. pgrx, SPI, libpq, PostgreSQL memory
contexts, and C callback entrypoints are outside their model and must stay in
PG18 pgrx or live-cluster lanes.

Seeded Miri coverage now includes:

- storage `ItemPointer` and data-page chain behavior,
- DiskANN metadata encode/decode,
- SPIRE leaf V2 object metadata and segment invariants through existing
  in-module tests with `miri_` prefixes.

## SIMD/Scalar Differential Validation

- `make simd-diff`: runs `tests/simd_diff.rs` with the `bench` feature. The
  harness compares production-dispatched scoring/FWHT entry points against
  scalar-reference entry points in the same process, covering the host-reachable
  SIMD backend without relying on `ECAZ_SIMD` process-global dispatch.
- Tolerances:
  - FWHT lanes: absolute/relative `1e-5`.
  - `score_ip_from_parts`: absolute/relative `1e-5`.
  - `score_ip_codes_lite`: absolute/relative `1e-5`.

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

The current live CLI smoke creates AM-specific fixtures for `ec_hnsw`, `ec_ivf`,
`ec_diskann`, and `ec_spire`, then directly exercises cancellation, statement
timeout, lock timeout, scan, insert, vacuum, and resource settings on those
fixtures. I/O, memory-allocation failure, and slow-disk lanes
require a provider such as LD_PRELOAD/libfiu, a FUSE filter, or a PG test hook;
until that provider is installed, the CLI refuses non-dry-run execution for
those lanes rather than reporting a false pass. Every lane uses the shared
post-condition probe inventory from `ecaz-fault-injection`.

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
- `make flux`: Flux proof for bounded payload dimension/index arithmetic.
- `make loom`: small atomic worker-slot claim/release model.
- `make shuttle`: small deterministic coordinator merge-order model.

These are pilot lanes. They are intentionally separate from normal `cargo test`
so the repo does not acquire heavyweight model-checking dependencies on the
default path.

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
