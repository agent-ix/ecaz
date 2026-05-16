# Task 34: Comprehensive Hardening

Status: **implemented** — local-first hardening surface, docs, and seed
harnesses landed; optional/manual lanes remain local-reporting until burn-in.

## Scope

Expand the existing safety stack into a broad, tool-backed hardening program.
The first slice should add durable targets, docs, and repeatable local lanes;
CI promotion comes only after each lane is low-noise and reproducible.

This is a union of the current repo scaffolding and the proposed hardening
toolset. Do not reduce scope by choosing a smaller overlapping subset.

## Tooling Lanes

- **Existing baseline.** Keep `fmt-check`, Clippy, unit tests, PG18 pgrx tests,
  property tests, layout assertions, unsafe-comment audit, Miri, cargo-fuzz,
  cargo-deny, and benchmark gates.
- **Supply chain.** Add `cargo-audit`, full `cargo-deny check`, and `cargo-vet`
  setup. Start with local/reporting mode; promote cargo-audit and cargo-deny to
  PR gates after the initial burn-in.
- **Unsafe/static hygiene.** Add `cargo-geiger` unsafe surface reporting, Rudra
  one-shot audits for this crate and unsafe-heavy dependencies, and exploratory
  MIRAI runs over pure Rust modules.
- **Runtime UB checks.** Add `cargo-careful` for pure Rust test paths. Add
  sanitizer lanes for pgrx and pure Rust where viable: ASan/LSan, TSan, and
  MSan.
- **Memory safety under interpretation.** Expand Miri beyond the current small
  quant/page tests into pure Rust vector ops, graph traversal helpers,
  serialization/layout code, DiskANN metadata, and SPIRE storage/merge/routing
  state that does not cross pgrx/SPI/libpq boundaries.
- **Fuzzing.** Keep libFuzzer/cargo-fuzz and add AFL.rs companion targets for
  the highest-value decoders. Add SQLsmith against a live PG18 cluster with
  ECAZ loaded for planner/CustomScan crash discovery.
- **Concurrency model checking.** Add Loom for small atomic/lock-free state and
  Shuttle for larger SPIRE coordinator and distributed merge state machines.
- **Formal verification.** Add Kani proofs for bounded wire/layout/merge/routing
  invariants. Pilot Flux on dimension/index invariants where it can be adopted
  incrementally without reshaping the codebase.

## Initial Implementation Slices

1. **Task surface and docs.**
   - Add Makefile targets for all lanes, with missing-tool checks that print
     install instructions instead of failing obscurely.
   - Add docs describing prerequisites, local-first cadence, CI promotion tiers,
     and why pgrx/SPI/live Postgres paths remain outside Miri/Kani.
   - Add aggregate targets:
     - `hardening-local`: stable local checks that do not need a live cluster.
     - `hardening-nightly-local`: slower sanitizer, fuzz, formal, and SQLsmith
       lanes.
2. **Miri/cargo-careful expansion.**
   - Add `miri_` tests for storage page chains, DiskANN metadata, SPIRE leaf V2
     object metadata, SPIRE placement/epoch metadata, top-k merge logic, and
     remote payload limit handling.
   - Add `careful` targets for pure Rust tests and selected CLI tests.
3. **Fuzz expansion.**
   - Add fuzz targets for SPIRE storage object/header/leaf V2 decoders, DiskANN
     metadata decode, CLI TSV parsing, manifest parsing, remote tuple payloads,
     row locators, top-k merge inputs, and vector normalization.
   - Add `fuzz-all-short` for smoke duration and longer manual target knobs.
4. **Formal/concurrency pilots.**
   - Add Kani harnesses for `ItemPointer`, tuple alignment arithmetic, quant
     payload lengths, SPIRE leaf V2 segment metadata, top-k merge ordering,
     partition routing, and remote parser rejection behavior.
   - Add Shuttle tests for SPIRE coordinator candidate merge/state transitions.
   - Add Loom tests for isolated atomic worker-slot claim/release and any small
     cache/free-list state that can be abstracted without pgrx pointers.
5. **Sanitizers and live-cluster hardening.**
   - Add ASan/LSan PG18 pgrx target for Rust/C boundary and memory-context
     issues.
   - Add TSan PG18 stress target for parallel build/scan, CustomScan
     concurrency, and SPIRE coordinator paths.
   - Add MSan where dependency/toolchain support makes it practical.
   - Add SQLsmith PG18 smoke target with ECAZ loaded.
6. **Unsafe and dependency audits.**
   - Add cargo-geiger reporting and a "no new unsafe without review-packet note"
     policy.
   - Run Rudra as a one-shot artifact-producing audit; triage findings into
     follow-up tasks instead of making Rudra a recurring PR gate immediately.
   - Initialize cargo-vet in report mode and import third-party audits where
     practical.

## CI Promotion Plan

- **Local first:** all new lanes start as Makefile/script targets and
  documented operator commands.
- **PR after burn-in:** promote cargo-audit, full cargo-deny, unsafe audit, and
  cargo-geiger delta reporting once local output is stable.
- **Nightly:** promote Miri expansion, cargo-careful, sanitizer pgrx runs,
  fuzz smoke, SQLsmith smoke, Kani, Shuttle, and Loom.
- **Weekly/manual:** keep Rudra, MIRAI, Flux, AFL.rs long campaigns, cargo-vet
  audit refresh, and long sanitizer/fuzz campaigns outside PR gating.

## Primary Targets

- Pure Rust:
  - `src/quant/*`
  - `src/storage/page.rs`
  - `src/am/ec_diskann/page.rs`
  - SPIRE storage, metadata, routing, remote payload, and merge helpers that do
    not require pgrx/SPI/libpq.
- Live PG18:
  - pgrx integration tests.
  - SPIRE CustomScan, DML, remote coordinator, parallel build/scan, and
    multicluster/fault scripts through existing operator surfaces.

## Acceptance Criteria

- Every listed tool has a documented local command or explicit documented
  deferral with rationale.
- Missing optional tools fail with actionable install/setup text.
- `hardening-local` runs the stable local subset without requiring a live
  cluster.
- Miri/cargo-careful/fuzz/Kani/Shuttle/Loom initial targets land with at least
  one meaningful ECAZ or SPIRE path each.
- Sanitizer and SQLsmith targets are documented and runnable in a PG18-capable
  environment, even if they remain nightly/manual.
- Rudra output is captured in a review packet artifact and follow-up findings
  are filed or explicitly closed.

## Assumptions

- Do not replace PG18 pgrx tests with Miri/Kani; these tools cover different
  failure classes.
- Prefer the broadest useful toolset, accepting overlap between tools.
- Keep first landing local-first to avoid making noisy or toolchain-sensitive
  lanes block normal development before burn-in.
