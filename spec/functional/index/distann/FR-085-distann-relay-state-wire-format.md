---
id: FR-085
title: Distann Relay-State Wire Format
type: FR
status: PROPOSED
relationships:
  - target: "ix://agent-ix/ecaz/FR-081"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-082"
    type: "depends_on"
    cardinality: "N:1"
  - target: "ix://agent-ix/ecaz/FR-079"
    type: "depends_on"
    cardinality: "N:1"
---
# FR-085: Distann Relay-State Wire Format

## Description

Relayed query state SHALL travel as a versioned, self-describing binary value
(`DISTANN_RELAY_STATE_V1`, bytea) that carries everything a receiving node
needs to resume the FR-081 beam search: identity/attestation, the query, the
budgets, the beam (doubling as the visited set), accumulated results, scan
counters, and — in direct mode — the return address (ADR-086 D2).

## Behavior

- The format SHALL open with a magic tag and a `version` field; a receiver
  encountering an unknown version SHALL raise a non-retriable error (never
  skip or best-effort parse).
- The state SHALL carry, in order: flags (return mode; `incomplete` marker);
  epoch number and the 16-byte epoch fingerprint (FR-082); the index name as
  a regclass-castable string (the existing cross-node index handle, FR-079);
  the raw full-precision query vector; search params (`beam_width`,
  `effective_top_k`, `hop_rounds_remaining`, `relay_depth_remaining`,
  optional `code_threshold`); beam entries `(vec_id, code_dist,
  expanded_flag)`; hits `(vec_id, exact_dist)`; scan + relay counters; and,
  when the return mode is direct, `coordinator_node_id` and `query_id`.
- The beam entry array SHALL be the visited-set representation: the FR-081
  beam is append-only and `expanded ⊆ enqueued`, so `(vec_id, expanded)`
  over all entries reproduces both sets on the receiver.
- The quantized query SHALL NOT travel: each node recomputes its
  `DistannPreparedQuery` from local codebooks; the epoch fingerprint attests
  codebook/metadata identity (ADR-086 D2).
- heap_tids SHALL NOT travel in any field (they are node-local, FR-079);
  hits carry `(vec_id, exact_dist)` only. Row materialization remains a
  coordinator-driven post-search step in every mode.
- The receiving node SHALL validate the epoch fingerprint before
  interpreting any field beyond the header, with the same
  mismatch-is-retriable semantics as `ec_distann_expand_nodes` (FR-079,
  FR-082).
- The remaining **expansion budget** (BW×H, ADR-086 D8) SHALL travel in the
  state and SHALL be the authoritative loop bound in every mode;
  `hop_rounds_remaining` is derived bookkeeping (a drain round may expand
  fewer than BW records, so rounds×BW ≠ expansions — the expansion count
  governs, per NFR-019).
- A receiver SHALL structurally validate every received state before use
  (ADR-086 D11 distrust posture): array lengths within the session-derived
  caps, budgets (`hop_rounds_remaining`, `relay_depth_remaining`, expansion
  budget) no greater than the session-configured maxima, no duplicate beam
  vec_ids, `expanded` flags consistent (expanded ⊆ enqueued), and a
  well-formed return address when the direct flag is set. A state failing
  validation SHALL raise a non-retriable error.
- Direct-mode return conninfo SHALL be resolved from the shared roster via
  `coordinator_node_id`; the state never carries raw conninfo strings.
- The serializer SHALL record the encoded size in the relay counters
  (`state_bytes_max`, `state_bytes_total` per query); the documented size
  envelope is stated in [NFR-021](../../../non-functional/NFR-021-distann-relay-resource-bounds.md).

## Acceptance Criteria

| ID | Criteria | Verification |
|----|----------|--------------|
| FR-085-AC-1 | serialize → deserialize round-trip reproduces the beam, visited set, hits, params, counters, and flags exactly | Test |
| FR-085-AC-2 | Unknown version tag raises a non-retriable error | Test |
| FR-085-AC-3 | Fingerprint mismatch is detected before any state-dependent work and maps to the retriable epoch-mismatch class | Test |
| FR-085-AC-4 | No heap_tid appears anywhere in the encoded state (format inspection over a multinode scan) | Test |
| FR-085-AC-5 | Encoded size stays within the NFR-021 envelope at default BW/H/R on the fixture, and `state_bytes_max`/`state_bytes_total` report it | Test (counter assertion) |
| FR-085-AC-6 | Structurally invalid states (duplicate beam vec_ids, budgets above session maxima, oversized arrays, malformed return address) are rejected with a non-retriable error before any index read | Test |

## Dependencies

- **Upstream**: [FR-081](./FR-081-distann-query-orchestration.md),
  [FR-082](./FR-082-distann-epoch-lifecycle.md),
  [FR-079](./FR-079-distann-remote-expansion-protocol.md); ADR-086 D2/D8
- **Downstream**: [FR-086](./FR-086-distann-relay-endpoint-local-drain.md),
  [FR-087](./FR-087-distann-stack-return.md),
  [FR-088](./FR-088-distann-direct-return.md),
  [FR-089](./FR-089-distann-relay-depth-hybrid-resume.md)
