# Task 214: ec_distann Spec Remediation

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **complete** (2026-08-01). Priority: P1 (documentation debt; blocks no
runtime work but degrades every review that leans on the spec).

Closeout: all phases P0–P5 done same-day (P0 drift inventory
`reviews/task-214/001-drift-inventory/`, 78 findings; P1 elevation to
`spec/functional/distann/` + FR-085 domain model; P2 amendments incl.
FR-086/ADR-087; P3 FR-087 catalog; P4 six flow diagrams; P5 base
spec-review + failure-domain/integrity/scope-boundary analyses with all
high/medium findings resolved — `spec/reviews/*.md`). Packet request open
for reviewer feedback; candidate code bugs surfaced by the audit are
listed in the packet inventory §H, not fixed here.

Entry gate: none. Runs independently of 211–213; their P0 spec additions
should land against the remediated structure where sequencing allows.

## Why

The ec_distann spec set has drifted below the standard the SPIRE docs hold,
and reviews increasingly route around it to the code. Two structural
problems and a list of content gaps:

- **The spec is not verified against the implementation.** ec_distann has
  moved fast (Tasks 179–213: epochs, physical shards, sharded/membership-only
  head, replicas, gateway copies, the NFR-021 ratchet), and the FRs were
  amended reactively. Nothing has walked the code and confirmed every FR/NFR
  still describes what ships — Task 203 already found four axes of
  paper-vs-code drift, and spec-vs-code drift is the same failure one layer
  down.
- **The spec is buried.** SPIRE owns a top-level functional directory
  (`spec/functional/spire/` with its own `index.md`, domain model, and
  `local/`/`distributed/`/`storage/`/`operations/` structure) while
  ec_distann — the fifth access method and the current program focus — sits
  at `spec/functional/distann/` as a subdirectory of the generic index
  specs.

Known content gaps, from comparison against the SPIRE documentation set:

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

The ec_distann spec set is **verified current against the implementation**,
lives at a **top-level directory peer to SPIRE**
(`spec/functional/distann/`), and is structurally at parity with the SPIRE
docs: internal tables described, core flows diagrammed, requirement text
normative and concise, the whole set passing `/spec-review` cleanly.

## Phases

- **P0 — full spec-vs-code examination.** Walk the ec_distann implementation
  (`src/am/ec_distann/`, the SQL surface, the catalog DDL, the suite/fixture
  conformance machinery) against every distann FR/NFR/ADR and itemise drift:
  behavior shipped but unspecified, specified but changed, specified but
  removed. Recent known deltas to fold in rather than rediscover: sharded +
  membership-only head as the default with the state-row membership blob
  (Task 210 round 2), replica serving with the members-derived shard
  ordinal, attested replica population, gateway copies and their capacity
  semantics, the NFR-021 storage classes and the closed-allowlist ratchet,
  and the fused-hop/crown surface arriving from Tasks 212/213. Use
  `/implementation-gap-analysis` and `/spec-review` to seed the audit rather
  than hand-walking alone. Output: the packet's drift+gap inventory — the
  work list for every later phase.
- **P1 — elevation.** Move `spec/functional/distann/` to
  `spec/functional/distann/` as a peer of `spec/functional/spire/`, with its
  own `index.md`, a domain-model anchor (SPIRE's `FR-048` pattern), and
  SPIRE-style substructure where the content warrants it; fix every
  cross-reference (specs, ADRs, tasks, review packets, code comments that
  cite spec paths).
- **P2 — spec updates from the P0 inventory.** Amend or author the drifted
  FRs/NFRs via `/specify`, validated with `/spec-review`.
- **P3 — internal tables.** Author the table descriptions via `/specify`
  (catalog/object docs): schema, invariants, writers/readers, lifecycle,
  epoch-scoping, conformance class of each relation (the NFR-021 storage
  classes now in the emitter: `coordinator_resident_unsharded`, `bounded`,
  `control`).
- **P4 — diagrams.** Sequence/process diagrams for the flows above, stored
  with the spec set in the repo's diagram convention.
- **P5 — verbosity pass.** Tighten overly narrative requirement texts;
  history moves to links. Each touched artifact re-validated with
  `/spec-review`.

## Output

Spec commits per phase plus a review packet under `reviews/task-214/`
tracking the P0 drift+gap inventory and its burn-down. No benchmark gate —
this task changes no quantizer/index/scan/storage behavior; closeout is the
P0 inventory fully burned down, the set relocated, and a clean
`/spec-review` pass over the touched set.
