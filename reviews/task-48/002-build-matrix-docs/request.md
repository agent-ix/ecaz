# Task 48/002: build matrix docs (closes §Exit Criteria #4)

## Scope

Closes Task 48 §Exit Criteria gate 4:

> 4. `docs/build-matrix.md` documents the supported matrix, the
>    cadence, and the policy for adding new targets.

Validation head: `ddb51741e`. Documentation-only slice (the
companion change in `docs/hardening.md` closes Task 46 §Exit #5
under `reviews/task-46/004-engine-matrix-docs/`).

## What changed

- `docs/build-matrix.md` — new file. Documents:
  - Supported target triples + PG versions + toolchains + cadences
    in a single policy table (5 entries: `aarch64-apple-darwin`,
    `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`,
    `x86_64-unknown-linux-musl`, `s390x-unknown-linux-gnu`
    via qemu-user).
  - PG version policy: pg18 primary on every per-PR lane; pg17
    optional unless touched; pg19 lands when upstream RC ships.
  - Rust toolchain policy: stable everywhere; nightly for miri,
    careful, sanitizers, cargo-fuzz.
  - Per-cadence Make drivers (per-PR / nightly / weekly /
    pre-release).
  - Soak harness with `make soak DURATION=24h` and the slope-fit
    RSS gate.
  - Resource-exhaustion scenario table (6 scenarios with stress
    target + expected disposition).
  - Qemu-user big-endian decode lane for Task 42 fixtures.
  - Add-a-target / downgrade-a-target policies.

No production code change. Documentation-only.

## Reviewer focus

- Matrix table is the policy reference; future build-matrix slices
  add their target as a row + corresponding workflow file under
  `.github/workflows/build-matrix-{triple}.yml`.
- macOS-specific `_BufferBlocks` dyld blocker is referenced in the
  `aarch64-apple-darwin` row note (memory:
  `feedback_dyld_buffer_blocks_known`) rather than re-derived.
- Resource-exhaustion scenarios match Task 48 §Scope bullet 4 line-
  for-line, including the disk-full scenario that pairs with Task
  38 fault injection.
- Cross-references at the bottom point to Task 42 (fixtures), Task
  38 (fault injection), Task 49 (CI governance) — these are the
  upstream tasks whose work this doc consumes.

## Out of scope (other Task 48 gates)

- §Exit #1 (CI matrix lanes running green): doc names the lanes
  and Make targets; the workflow files + green runs are follow-up
  slices.
- §Exit #2 (`make soak DURATION=24h` weekly): kickoff harness
  lands at packet 001; the Make wrapper + weekly cadence + RSS
  sampler + slope-fit assertion are follow-up slices.
- §Exit #3 (`make resource-exhaustion` nightly): doc names the
  six scenarios; the CLI subcommand + nightly cadence is a
  follow-up slice.

This packet closes §Exit #4 only. Task 48 now ~25% complete
(1 of 4 gates closed; one partial).
