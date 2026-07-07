# Task 151: SPIRE RaBitQ slab-copy elimination (score columnar payloads in place)

Status: **proposed** (2026-07-04). Owner: unassigned. Priority: P3

## Why

The SPIRE `QuantCodec::score_ip_batch` RaBitQ arm copies every candidate's
code into a fresh `slab: Vec<u8>` via `extend_from_slice`
(`src/am/ec_spire/quantizer/mod.rs:517-536`) before calling
`score_rabitq_payload_slab`, which then builds another `CandidateBatch`
(`:736`), which the shared driver gathers into a `Vec<&[u8]>`
(`src/am/common/candidate_batch/mod.rs:736`). The V2 columnar leaf payloads
are **already contiguous** — the row scorer slices `columns.payloads` with
`chunks_exact(payload_stride)` into batch refs
(`src/am/ec_spire/candidates.rs:3075-3107`). The TurboQuant arm right above
(`quantizer/mod.rs:467-484`) passes the batch straight through with no copy;
the extra full-payload copy is RaBitQ-only. Net: four re-batch/copy layers for
SPIRE RaBitQ vs one for TQ.

Caveat, recorded so expectations stay honest: Tasks 77/78 measured SPIRE
latency as candidate-volume-bound (scoring ~87-88% of candidate-path CPU only
because too many candidates are admitted), so this is a per-candidate-cost
win, not a structural one.

## Scope

- Route the SPIRE RaBitQ batch path through the batch refs directly (mirroring
  the TQ arm), keeping the slab entry point (`score_batch` at
  `quantizer/mod.rs:352`) for callers that already own a contiguous slab.
- Keep the gamma==0 and stride validation checks; hoist them out of the copy
  loop rather than deleting them.
- A/B per CLAUDE.md on the ec_spire RaBitQ lane at 10k/50k/100k
  (recall byte-identical expected; latency + storage reported).

## Out of Scope (hard)

- No change to the columnar leaf format or the coarse summary prefilter path
  (`score_zero_gamma_payload_chunks_max_prevalidated`).

## Gate / Exit Criteria

- Byte-identical recall, the copy layer gone (verified by code review +
  counters), and a measured ec_spire latency delta at 10k/50k/100k. An honest
  null result still closes the task (the copy is deleted either way).
