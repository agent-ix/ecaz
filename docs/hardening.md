# Hardening Lanes

Task 34 keeps new hardening local-first until each lane is reproducible and
low-noise. The Makefile is the entrypoint; optional tools are checked by
`scripts/hardening.sh` so missing tools fail with setup text instead of cargo
subcommand errors.

## Aggregates

- `make hardening-local` runs stable local checks that do not need a live
  cluster: format check, PG18 Clippy, unit tests, property tests, layout
  assertions, unsafe comment audit, full `cargo-deny`, and `cargo-audit`.
- `make hardening-nightly-local` adds slower and toolchain-sensitive local
  lanes: expanded Miri, `cargo-careful`, fuzz smoke, Kani, Loom, Shuttle,
  `cargo-geiger`, and pure Rust ASan/LSan.

## Baseline

- `make fmt-check`
- `make lint`
- `make test`
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
- `make rudra`: install Rudra and keep the one-shot output under
  `review/30034-task34-comprehensive-hardening/artifacts/rudra.log`.
- `make mirai`: install with `cargo install --locked mirai`.
- `make flux`: install Flux from the upstream project.

Policy: new unsafe blocks need a nearby `SAFETY` comment and, when the unsafe
surface is non-trivial, the review packet should call out why the boundary is
valid. Rudra, MIRAI, and Flux stay manual while their false-positive profile is
unknown for pgrx-heavy code.

## Miri And Cargo-Careful

- `make miri-expanded`: runs `cargo +nightly miri test --lib -- miri_`.
- `make careful`: runs `cargo careful test --lib --tests`.

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
  30 seconds.
- Individual targets: `make fuzz-parse-text`, `make fuzz-unpack`,
  `make fuzz-element-decode`, `make fuzz-neighbor-decode`,
  `make fuzz-diskann-metadata`, `make fuzz-item-pointer`, and
  `make fuzz-vector-normalize`.
- `make afl-decoders`: builds the DiskANN metadata and `ItemPointer` decoder
  targets with AFL.rs for longer manual campaigns.

SQLsmith is live-cluster only:

```sh
export ECAZ_HARDENING_SQLSMITH_DSN='postgresql://localhost/postgres'
make sqlsmith-pg18
```

Use a PG18 cluster with `ecaz` installed. Capture crashes and raw SQLsmith logs
under the relevant review packet before citing findings.

## Concurrency And Formal Pilots

- `make kani`: bounded proof for `ItemPointer` decode length behavior.
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
