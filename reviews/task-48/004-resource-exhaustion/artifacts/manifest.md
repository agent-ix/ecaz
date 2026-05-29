# Packet 004 — Task 48: resource-exhaustion sweep

## Head

- Task bucket: `reviews/task-48/`
- Packet path: `reviews/task-48/004-resource-exhaustion/`
- Validation head SHA: `9ecfa83ea`
- Branch: `main`
- Surface under validation: `ecaz dev resource-test` CLI
  subcommand + `make resource-exhaustion` Make recipe.

## What changed

| Path | Kind |
|---|---|
| `crates/ecaz-cli/src/commands/dev/resource_test.rs` | new code (~360 LOC) |
| `crates/ecaz-cli/src/commands/dev/mod.rs` | dispatch arm |
| `Makefile` | `make resource-exhaustion` recipe |

## Artifacts

This packet ships no PG-backed smoke artifact — restart-only GUCs
(`max_locks_per_transaction`, `max_connections`, `shared_buffers`)
require a pre-configured cluster that the local CLI host does not
provide. The nightly CI workflow
(`.github/workflows/resource-exhaustion-nightly.yml`, shipped in
Task 48/005) is where the full sweep first executes.

Compile evidence:
- `cargo check -p ecaz-cli` finishes in 8.38s. Zero new warnings.

## Outcome taxonomy

| Outcome | Hard failure? |
|---|---|
| `pass` | no |
| `prereq_unmet` | no (operator action item) |
| `workload_did_not_trigger` | no (tuning signal) |
| `broken_connection` | **yes** — exit non-zero |

## Task 48 progress after this slice

3 of 4 §Exit gates closed (#2 soak, #3 resource-exhaustion, #4
build-matrix docs). #1 (CI matrix) closes in the companion 005
packet.
