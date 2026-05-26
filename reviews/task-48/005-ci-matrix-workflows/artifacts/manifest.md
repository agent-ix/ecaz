# Packet 005 — Task 48: CI matrix workflows

## Head

- Task bucket: `reviews/task-48/`
- Packet path: `reviews/task-48/005-ci-matrix-workflows/`
- Validation head SHA: post-commit landing the six workflow files
  immediately after `9ecfa83ea`.
- Branch: `main`
- Surface under validation: six new `.github/workflows/*.yml` files.

## What changed

Six new files under `.github/workflows/`:

| Workflow | Bytes | Cadence |
|---|---:|---|
| `build-matrix-x86_64-linux-gnu.yml` | ~2.5 KB | per-PR |
| `build-matrix-aarch64-darwin.yml` | ~1.8 KB | per-PR |
| `build-matrix-aarch64-linux-gnu.yml` | ~3.1 KB | per-PR + nightly |
| `build-matrix-nightly-toolchain.yml` | ~2.1 KB | nightly |
| `resource-exhaustion-nightly.yml` | ~2.3 KB | nightly |
| `soak-weekly.yml` | ~2.1 KB | weekly |

No code change outside `.github/workflows/`. The lanes invoke
existing Make targets (`make miri-expanded`, `make miri-tree`,
`make miri-many-seeds`, `make fuzz-all-short`,
`make resource-exhaustion`, `make soak`) and the `cargo` /
`cargo build -p ecaz-cli` commands.

## Artifacts

This packet ships no run artifacts — first runs only land after
the workflows merge to `main` and the per-cadence triggers fire.
The next nightly + the next Mon-02:00 will produce the first
upload-artifact JSONs (`resource-exhaustion-summary.json` and
`soak-weekly.json`).

## Task 48 §Exit Criteria after this packet

| # | Criterion | Status |
|---|---|---|
| 1 | CI matrix covers aarch64-darwin + x86_64-linux + aarch64-linux + pg17 + pg18 | ✓ done |
| 2 | `make soak DURATION=24h` weekly | ✓ done |
| 3 | `make resource-exhaustion` nightly | ✓ done |
| 4 | `docs/build-matrix.md` documents matrix, cadence, policy | ✓ done |

**Task 48: 4 of 4 §Exit gates closed (100%).**
