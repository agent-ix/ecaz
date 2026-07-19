# Task 184 packet 003 artifact manifest

- Task bucket / packet: `reviews/task-184/003-isolated-candidate/`
- Runner checkpoint: `eacd0026428a02cf306cc3be411a2007d9265b42`
- Candidate checkpoint: `245c2054f`
- Status: implementation checkpoint; live correctness and A/B pending

| Artifact | Command / meaning | Result |
| --- | --- | --- |
| `cargo-check-runner-extension.log` | `cargo check -p ecaz-cli` | pass |
| `cargo-test-runner-parser.log` | focused materialization-arm parser test | pass |
| `cargo-test-runner-suite.log` | focused same-generation expansion test | pass |
| `cargo-check-candidate.log` | feature PG18 check | pass |
| `cargo-check-candidate-production.log` | normal PG18 check | pass; benchmark GUC/path absent |
| `cargo-test-candidate-window.log` | focused deterministic ranked-window test | 1 passed |

The runner accepts an optional sixth physical seed-variant field for the
materialization batch size. Five-field configs remain compatible and default
to eager materialization (`0`). Positive variants set the benchmark-feature-
only GUC in recall and latency child sessions while retaining the same
immutable generation and seed digest.

The candidate keeps pending remote `vec_id` identities in ranked output. On
executor demand it fetches all still-pending remote identities from the current
global 10-slot window, capped by the proven prefix, through the existing
schema/epoch-fenced endpoint. Search deepening starts a rebuilt window at the
already-consumed cursor, preventing duplicate fetches of the stable prefix.

Live correctness, same-generation isolated A/B, installed release provenance,
and compact suite outputs will be appended before packet completion.
