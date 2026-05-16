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
  layout assertions, unsafe comment audit, full `cargo-deny`, and
  `cargo-audit`.
- `make hardening-nightly-local` adds slower and toolchain-sensitive local
  lanes: expanded Miri, `cargo-careful`, fuzz smoke, Kani, Loom, Shuttle, and
  pure Rust ASan/LSan. `cargo-geiger` remains a standalone reporting lane
  because it can force a large clean rebuild.

## Baseline

- `make fmt-check`
- `make lint`
- `make lint-hardening`
- `make test`
- `make test-hardening-local`
- `make pg-test`
- `make proptest`
- `make layout-check`
- `make audit-unsafe`
- `make miri`
- `make fuzz-all-short`
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
