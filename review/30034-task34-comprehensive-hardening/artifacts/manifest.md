# Manifest: Task 34 Comprehensive Hardening Surface

Head SHA: `bb2d8a0b5a2b9e71baac4d0ed8010c0da13534fb`
Packet: `30034-task34-comprehensive-hardening`
Timestamp: `2026-05-16T20:14:01Z`

This packet does not cite performance or recall measurements. The validation
claims in `request.md` are command pass/fail results from local hardening lanes.

## Artifacts

### `install-hardening-tools.log`

- Head SHA: `bb2d8a0b5a2b9e71baac4d0ed8010c0da13534fb`
- Packet/topic: `30034-task34-comprehensive-hardening`
- Lane: reusable optional tool installer
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `bash scripts/install_hardening_tools.sh --all --log-file review/30034-task34-comprehensive-hardening/artifacts/install-hardening-tools.log`
- Timestamp: `2026-05-16T19:08:00Z`
- Surface: local, no table/index
- Key result lines:
  - Rudra Docker helper installed under `~/.ecaz/hardening-tools`
  - Initial MIRAI build stopped on missing `cmake`

### `install-hardening-tools-retry.log`

- Head SHA: `bb2d8a0b5a2b9e71baac4d0ed8010c0da13534fb`
- Packet/topic: `30034-task34-comprehensive-hardening`
- Lane: MIRAI/Flux installer retry
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `bash scripts/install_hardening_tools.sh --mirai --flux --log-file review/30034-task34-comprehensive-hardening/artifacts/install-hardening-tools-retry.log`
- Timestamp: `2026-05-16T19:09:00Z`
- Surface: local, no table/index
- Key result lines:
  - MIRAI build reached Rust compilation
  - Stable Homebrew cargo could not build MIRAI's rustc-private crate

### `install-hardening-tools-rustup-retry.log`

- Head SHA: `bb2d8a0b5a2b9e71baac4d0ed8010c0da13534fb`
- Packet/topic: `30034-task34-comprehensive-hardening`
- Lane: MIRAI/Flux installer retry with rustup cargo
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `bash scripts/install_hardening_tools.sh --mirai --flux --log-file review/30034-task34-comprehensive-hardening/artifacts/install-hardening-tools-rustup-retry.log`
- Timestamp: `2026-05-16T19:13:00Z`
- Surface: local, no table/index
- Key result lines:
  - `Installed package ... executables cargo-mirai, mirai`
  - Flux installed but first runtime check exposed missing external `fixpoint`

### `install-hardening-tools-flux-fixpoint.log`

- Head SHA: `bb2d8a0b5a2b9e71baac4d0ed8010c0da13534fb`
- Packet/topic: `30034-task34-comprehensive-hardening`
- Lane: Flux installer with liquid-fixpoint
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `bash scripts/install_hardening_tools.sh --flux --log-file review/30034-task34-comprehensive-hardening/artifacts/install-hardening-tools-flux-fixpoint.log`
- Timestamp: `2026-05-16T19:15:00Z`
- Surface: local, no table/index
- Key result lines:
  - `cargo-flux 0cc1c5a (2026-05-15)`
  - liquid-fixpoint `fixpoint` installed for the host

### `mirai.log`

- Head SHA: `bb2d8a0b5a2b9e71baac4d0ed8010c0da13534fb`
- Packet/topic: `30034-task34-comprehensive-hardening`
- Lane: `mirai`
- Fixture: `hardening/careful/`
- Storage format: pure Rust harness
- Rerank mode: not applicable
- Command used: `bash scripts/hardening.sh mirai --log-file review/30034-task34-comprehensive-hardening/artifacts/mirai.log`
- Timestamp: `2026-05-16T19:14:00Z`
- Surface: local, no table/index
- Key result lines:
  - `Checking ecaz-careful-hardening`
  - `Finished dev profile`

### `flux.log`

- Head SHA: `bb2d8a0b5a2b9e71baac4d0ed8010c0da13534fb`
- Packet/topic: `30034-task34-comprehensive-hardening`
- Lane: `flux`
- Fixture: `hardening/flux/`
- Storage format: pure Rust dimension/index invariant harness
- Rerank mode: not applicable
- Command used: `bash scripts/hardening.sh flux --log-file review/30034-task34-comprehensive-hardening/artifacts/flux.log`
- Timestamp: `2026-05-16T19:17:00Z`
- Surface: local, no table/index
- Key result lines:
  - `3 functions processed: 3 checked; 0 trusted; 0 ignored`
  - `3 constraints solved`

### `rudra.log`

- Head SHA: `bb2d8a0b5a2b9e71baac4d0ed8010c0da13534fb`
- Packet/topic: `30034-task34-comprehensive-hardening`
- Lane: `rudra`
- Fixture: `hardening/rudra/`
- Storage format: pure Rust Rudra harness
- Rerank mode: not applicable
- Command used: `make rudra`
- Timestamp: `2026-05-16T19:49:00Z`
- Surface: local, no table/index
- Key result lines:
  - `Running rudra for target lib:ecaz-rudra-hardening`
  - `SendSyncVariance analysis finished`
  - `UnsafeDataflow analysis finished`
  - `cargo rudra finished`

### `rudra-missing-tool.log`

- Head SHA: `83b5669f2b1f08de78df4c435e6770ee20484b2d`
- Packet/topic: `30034-task34-comprehensive-hardening`
- Lane: `rudra`
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `make rudra`
- Timestamp: `2026-05-16T18:53:10Z`
- Surface: local, no table/index
- Key result lines:
  - `missing optional hardening tool: cargo-rudra`
  - `Install Rudra from https://github.com/sslab-gatech/Rudra and ensure cargo-rudra is on PATH`

### `rudra-careful.log`

- Head SHA: `bb2d8a0b5a2b9e71baac4d0ed8010c0da13534fb`
- Packet/topic: `30034-task34-comprehensive-hardening`
- Lane: `rudra`
- Fixture: `hardening/careful/`
- Storage format: pure Rust path-including harness
- Rerank mode: not applicable
- Command used: `bash scripts/hardening.sh rudra --manifest-path hardening/careful/Cargo.toml --log-file review/30034-task34-comprehensive-hardening/artifacts/rudra-careful.log`
- Timestamp: `2026-05-16T19:37:00Z`
- Surface: local, no table/index
- Key result lines:
  - `Running rudra for target lib:ecaz-careful-hardening`
  - `couldn't read src/../../../src/storage/page.rs`

### `rudra-root-metadata-failure.log`

- Head SHA: `bb2d8a0b5a2b9e71baac4d0ed8010c0da13534fb`
- Packet/topic: `30034-task34-comprehensive-hardening`
- Lane: `rudra`
- Fixture: repository root
- Storage format: not applicable
- Rerank mode: not applicable
- Command used: `bash scripts/hardening.sh rudra --manifest-path Cargo.toml --log-file review/30034-task34-comprehensive-hardening/artifacts/rudra-root-metadata-failure.log`
- Timestamp: `2026-05-16T20:15:00Z`
- Surface: local, no table/index
- Key result lines:
  - `Could not obtain Cargo metadata`
  - `failed to select a version for the requirement hashbrown = "^0.15"`

### `rudra-smoke.log`

- Head SHA: `bb2d8a0b5a2b9e71baac4d0ed8010c0da13534fb`
- Packet/topic: `30034-task34-comprehensive-hardening`
- Lane: `rudra`
- Fixture: `hardening/rudra/`
- Storage format: pure Rust Rudra harness
- Rerank mode: not applicable
- Command used: `bash scripts/hardening.sh rudra --manifest-path hardening/rudra/Cargo.toml --log-file review/30034-task34-comprehensive-hardening/artifacts/rudra-smoke.log`
- Timestamp: `2026-05-16T19:38:00Z`
- Surface: local, no table/index
- Key result lines:
  - `Running rudra for target lib:ecaz-rudra-hardening`
  - `cargo rudra finished`
