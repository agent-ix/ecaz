# Task 204: ec_distann Storage-Step Arm Fidelity

> **MULTI-NODE MEASUREMENT RULE (NON-NEGOTIABLE).** Any decision about
> distributed behavior — latency, recall, storage, or overhead — MUST be measured
> on a multi-node configuration. A single-node / single-instance arm is NEVER
> acceptable as the basis for a decision about a distributed algorithm; its only
> permitted use is a clearly labeled baseline that quantifies distribution
> overhead. Label every reported number with its arm's node count. See
> AGENTS.md → "Distributed Measurement: Multi-Node Arms Only".

Status: **ready** (2026-07-29). Priority: P0 measurement integrity.

Entry gate: none. This is the first slice; everything downstream reports storage.

## Why

Task 203 found the multinode suite's storage step cannot express a difference
between arms. `crates/ecaz-cli/src/commands/dev/distann_multicluster.rs:5153-5160`
computes `physical_generation_bytes`, `control_index_bytes`, `single_index_bytes`,
and `single_source_bytes` **once, before** the variant loop, and `:5209-5212`
reprints them unchanged **inside** it. The owner and replica rows at 100k are
therefore byte-identical by construction, both carrying
`physical_generation_bytes=2496659456`.

Two consequences:

1. Task 199 promoted on a claim of "unchanged storage between the owner and
   replica arms" that is a statement about what the step does not measure. The
   replica's 1,659,518,976 bytes reach only a log-only metric
   (`physical_benchmark_traversal_replica.relation_bytes`); `grep -c relation_bytes`
   returns 0 for every `results.jsonl` in the 198/199 packets.
2. The topology rows feeding the numerator are captured at `phase=ready`/
   `phase=published`, **before** the replica is built, so a coordinator-side
   relation could not enter the ratio even in principle.

`cluster_index_space_amplification` — the real NFR-018 ratio emitter at
`distann_multicluster.rs:7419-7482` — exists and ran for Tasks 172 and 197. It
was not run for 198/199, and `NFR-018` requires the ratio row in the packet
manifest per scale.

NFR-018 and NFR-021 (landed 2026-07-29) now require per-arm measurement, a
per-node maximum, and emission into `results.jsonl` rather than a log sidecar.
This task makes the runner able to satisfy them.

## Goal

Make ec_distann storage measurable **per arm** and **per node**, in
`results.jsonl`. No storage behavior changes; only measurement.

## Phases

1. **Move measurement inside the arm loop.** Compute the storage scalars per
   variant so the emitted row reflects the arm actually under test. Any value
   that is genuinely arm-invariant stays, but must be labeled as such rather than
   implying it was measured per arm.
2. **Emit coordinator-side and derived relations.** Parse
   `physical_benchmark_traversal_replica` (`relation_bytes`, `wal_bytes`,
   `copied_bytes`, `build_ms`) and
   `physical_benchmark_traversal_replica_cache` into `results.jsonl`. Every
   index-derived relation is attributed to the node holding it, including
   optional and disabled-by-default relations.
3. **Per-node rows and the NFR-021 growth ratio.** Emit max-single-node resident
   index bytes per scale, and the cross-scale growth ratio the invariant needs.
4. **Make the ratio row mandatory.** `cluster_index_space_amplification` runs on
   every distann gate run; its absence fails the run rather than being silently
   omitted.

## Validation

This task changes measurement, not index behavior, so the 10k/50k/100k
recall/latency closeout rule does not apply. The required proof is instead:

- a 100k two-arm distann run whose storage rows **differ between arms** where the
  arms genuinely differ, with the replica relation visible in `results.jsonl`;
- the NFR-018 summed ratio row and the per-node maximum row present per scale in
  the packet manifest;
- a re-read of the Task 198/199 committed artifacts under the new emitter,
  reporting what the corrected accounting shows.

Focused PG18 validation only where a callback or SQL path is touched.

## Required review packets

1. `reviews/task-204/001-arm-fidelity/` — code checkpoint plus the two-arm
   demonstration and the corrected re-read of the 198/199 numbers.

## Non-goals

- Changing storage layout, the replica, or any index behavior.
- Deciding FR-084's disposition.
- Re-running the 198/199 benchmark matrices. A re-read of committed artifacts is
  sufficient here; re-measurement belongs to whichever task reopens them.

## References

- `reviews/task-203/001-decision-reaudit/` Defect 4b.
- `NFR-018` (per-node term, per-arm requirement), `NFR-021`, `NFR-022`, `NFR-007`.
