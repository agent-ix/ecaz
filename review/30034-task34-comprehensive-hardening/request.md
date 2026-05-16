# Review Request: Task 34 Comprehensive Hardening Surface

Head: `773c75b487277a15f8ca8c82cb313858be7abff9`

Scope:
- `Makefile`
- `scripts/hardening.sh`
- `scripts/install_hardening_tools.sh`
- `scripts/check_unsafe_comments.sh`
- `scripts/unsafe_comment_baseline.txt`
- `docs/hardening.md`
- `fuzz/`
- `hardening/careful/`
- `hardening/flux/`
- `hardening/kani/`
- `hardening/loom/`
- `hardening/rudra/`
- `hardening/shuttle/`
- `supply-chain/`
- Miri, sanitizer, and formatting support edits in touched Rust modules

What changed:
- Added a local-first hardening surface for supply-chain checks, unsafe/static
  hygiene, expanded Miri, cargo-careful, libFuzzer/AFL, Kani, Loom, Shuttle,
  sanitizers, SQLsmith, MIRAI, Flux, Rudra, and aggregate `hardening-local` /
  `hardening-nightly-local` lanes.
- Added reusable optional-tool setup in `scripts/install_hardening_tools.sh`;
  upstream Rudra/MIRAI/Flux checkouts live under `~/.ecaz/hardening-tools` so
  future tasks can reuse them.
- Routed toolchain-sensitive lanes through scripts and Make variables instead
  of command-line environment prefixes.
- Added `FUZZ_SECONDS`, `SQLSMITH_DSN`, and `RUDRA_MANIFEST` Make knobs for the
  lanes that need operator input.
- Added standalone pure-Rust hardening crates for cargo-careful, Flux, Kani,
  and Rudra so those lanes avoid PostgreSQL callback symbol loading and old
  toolchain limitations.
- Made the fuzz crate standalone over pure modules and pointed all fuzz targets
  at the shared fuzz API.
- Initialized cargo-vet report-mode scaffolding and deny/audit policy for the
  current dependency graph.
- Made the unsafe-comment audit baseline-backed so the lane blocks new
  uncommented unsafe without forcing a full historical cleanup in this task.

Review focus:
- Whether every task 34 lane has a clear local command or documented live/PG18
  prerequisite.
- Whether `hardening-local` and `hardening-nightly-local` are the right
  low-noise local aggregates for burn-in.
- Whether the standalone pure-Rust harnesses are a reasonable boundary for
  Miri, cargo-careful, Flux, Kani, Rudra, and sanitizer smoke checks.
- Whether the unsafe baseline policy is acceptable for "no new unsafe without a
  nearby `SAFETY` comment" enforcement.

Validation:
- `git diff --check` passed.
- `bash scripts/install_hardening_tools.sh --check` found all new optional
  tools: cargo-audit, cargo-deny, cargo-vet, cargo-geiger, cargo-careful,
  cargo-fuzz, cargo-afl, cargo-kani, sqlsmith, cargo-mirai, cargo-flux, and the
  Rudra Docker helper.
- `make test` passed: 331 CLI tests and 8 standalone pure-Rust harness tests.
- `make hardening-local` passed.
- `make hardening-nightly-local FUZZ_SECONDS=1` passed.
- `make cargo-vet` passed.
- `make cargo-geiger` completed and reported the current unsafe surface.
- `make afl-decoders` built the AFL decoder targets.
- `make sanitizer-asan` passed.
- `make sanitizer-lsan` skipped cleanly on `aarch64-apple-darwin`.
- `make sanitizer-msan` skipped cleanly on `aarch64-apple-darwin`.
- `make sanitizer-tsan` passed.
- `make miri-expanded` passed 19 `miri_` tests.
- `make careful` passed the standalone pure-Rust cargo-careful harness.
- `make kani` verified `kani_item_pointer_decode_contract`.
- `make flux` checked 3 Flux functions and solved 3 constraints.
- `make rudra` ran Rudra's SendSyncVariance and UnsafeDataflow analyses against
  the standalone Rudra harness.
- `make mirai` passed against the standalone pure-Rust cargo-careful harness.
- `make loom` passed.
- `make shuttle` passed.

Known local limits:
- PG18 live lanes, PG sanitizer lanes, and SQLsmith require a running PG18
  cluster with `ecaz` installed and were not run in this local closeout.
- Rudra's pinned 2021 Cargo cannot resolve the root workspace's current
  dependency graph and cannot compile the path-including careful harness when
  mounted as an isolated Docker workdir. Both failure modes are captured as
  packet artifacts; the default `make rudra` target now uses a stable
  no-dependency Rudra harness.
