# Task 214: ec_distann Spec Remediation

Status: **ready** (2026-08-01). Priority: P1 (documentation debt; blocks no
runtime work but degrades every review that leans on the spec).

Entry gate: none. Runs independently of 211–213; their P0 spec additions
should land against the remediated structure where sequencing allows.

## Why

The ec_distann spec set has drifted below the standard the SPIRE docs hold,
and reviews increasingly route around it to the code. Known gaps, from
comparison against the SPIRE documentation set:

- **Missing internal table descriptions.** The extension owns a substantial
  catalog surface (`ec_distann_generation`, `_generation_head_state` — now
  carrying the membership blob — `_generation_head_sample`,
  `_head_shard_replica`, `_head_replica_state`, `_active_epoch`,
  `_build_candidate`, `_build_registration`, `_node_registry`, the traversal
  replica tables, …) with no per-table description of columns, invariants,
  lifecycle owners, or which endpoints read/write them. SPIRE documents its
  internal tables; ec_distann should match.
- **Missing sequence and process diagrams.** Epoch build (T-stages),
  publish/decide/retire, sharded head search (owner and replica serving),
  replica population/attestation, gateway-copy population/serving, and the
  scan open/epoch-pin lifecycle all exist only as prose and code. Each needs
  a sequence or process diagram at SPIRE-doc fidelity.
- **Over-verbosity.** Some requirement texts have accreted narrative that
  belongs in review packets; requirements should be normative and tight,
  with history linked, not inlined.

## Goal

The ec_distann spec set is structurally at parity with the SPIRE docs:
internal tables described, core flows diagrammed, requirement text normative
and concise, and the whole set passing `/spec-review` cleanly.

## Phases

- **P0 — inventory.** Enumerate the ec_distann spec/doc surface against the
  SPIRE set; produce the gap list (tables, diagrams, verbosity items) as the
  packet's working inventory. `/spec-review` (and, where useful,
  `/spec-integrity-analysis`, `/spec-object-review`) to seed findings rather
  than hand-auditing.
- **P1 — internal tables.** Author the table descriptions via `/specify`
  (catalog/object docs): schema, invariants, writers/readers, lifecycle,
  epoch-scoping, conformance class of each relation (the NFR-021 storage
  classes now in the emitter: `coordinator_resident_unsharded`, `bounded`,
  `control`).
- **P2 — diagrams.** Sequence/process diagrams for the flows above, stored
  with the spec set in the repo's diagram convention.
- **P3 — verbosity pass.** Tighten overly narrative requirement texts;
  history moves to links. Each touched artifact re-validated with
  `/spec-review`.

## Output

Spec commits per phase plus a review packet under `reviews/task-214/`
tracking the inventory and its burn-down. No benchmark gate — this task
changes no quantizer/index/scan/storage behavior; closeout is the clean
`/spec-review` pass over the touched set.
