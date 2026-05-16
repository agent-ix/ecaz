# Review Request: Task 34 Comprehensive Hardening Surface

Head: `83b5669f2b1f08de78df4c435e6770ee20484b2d`

Scope:
- `Makefile`
- `scripts/hardening.sh`
- `scripts/check_unsafe_comments.sh`
- `scripts/unsafe_comment_baseline.txt`
- `docs/hardening.md`
- `fuzz/`
- `hardening/careful/`
- `hardening/kani/`
- `hardening/loom/`
- `hardening/shuttle/`
- `supply-chain/`
- Miri, sanitizer, and formatting support edits in touched Rust modules

What changed:
- Added a local-first hardening surface for supply-chain checks, unsafe/static
  hygiene, expanded Miri, cargo-careful, libFuzzer/AFL, Kani, Loom, Shuttle,
  sanitizers, SQLsmith, and aggregate `hardening-local` /
  `hardening-nightly-local` lanes.
- Routed toolchain-sensitive lanes through `scripts/hardening.sh` so repeated
  invocations use script flags or Make variables instead of command-line
  environment prefixes.
- Added `FUZZ_SECONDS` and `SQLSMITH_DSN` Make knobs for the lanes that need
  operator input.
- Added standalone pure-Rust hardening crates for cargo-careful and Kani so
  those lanes avoid PostgreSQL callback symbol loading.
- Made the fuzz crate standalone over pure modules and pointed all fuzz targets
  at the shared fuzz API.
- Initialized cargo-vet report-mode scaffolding and deny/audit policy for the
  current dependency graph.
- Made the unsafe-comment audit baseline-backed so the lane blocks new
  uncommented unsafe without forcing a full historical cleanup in this task.
- Documented manual/deferred MIRAI and Flux setup and left `cargo-geiger` as a
  standalone reporting lane because it can force a large clean rebuild.

Review focus:
- Whether every task 34 lane has a clear local command or an explicit
  documented manual deferral.
- Whether `hardening-local` and `hardening-nightly-local` are the right
  low-noise local aggregates for burn-in.
- Whether the standalone pure-Rust harnesses are a reasonable boundary for
  Miri, cargo-careful, Kani, and sanitizer smoke checks.
- Whether the unsafe baseline policy is acceptable for "no new unsafe without a
  nearby `SAFETY` comment" enforcement.

Validation:
- `git diff --check` passed.
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
- `make loom` passed.
- `make shuttle` passed.
- `make rudra` produced the expected missing-tool output captured at
  `artifacts/rudra-missing-tool.log`.

Known local limits:
- `make test` still aborts in this macOS local runner on the existing pgrx
  callback-loader issue with unresolved PostgreSQL symbol `_BufferBlocks`; the
  new aggregate uses `test-hardening-local` for non-live local coverage.
- PG18 live lanes, PG sanitizer lanes, and SQLsmith require a running PG18
  cluster with `ecaz` installed and were not run in this local closeout.
- Rudra, MIRAI, and Flux remain manual/deferred until their upstream tools are
  installed. Rudra has a runnable Make/script lane and this packet captures the
  current missing-tool result.
