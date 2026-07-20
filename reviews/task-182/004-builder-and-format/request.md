---
task: 182
packet: 004-builder-and-format
role: coder
status: open
date: 2026-07-16
head: 43b3ace1a
---

# Review request: trained-head production contract

Task 182 is in progress. This checkpoint implements the frozen production
builder, format, compatibility, exact-head read selection, and inspection
contract for the Task 181 winner. The policy remains explicit and default-off.

## Selected candidate

- deterministic training-landmark selection inherited byte-for-byte from Task
  181: rank every source code for each training query, frequency-rank each
  query's top 32 with rank and vec_id tie-breaks, then fill unused cap slots
  from deterministic geometry landmarks;
- cap 4,096 and 32 returned seeds;
- exact inner-product scoring of every persisted landmark at query time;
- disjoint ordered training queries, never evaluation queries; and
- normal BW4/H100 global traversal with RaBitQ neighbor codes.

## Production build input

Production will not expose the benchmark policy/path GUCs or require the
`distann-head-attribution-benchmark` feature. The existing
`ec_distann_build_epoch(index, epoch, build_id)` remains the current sampled
head builder. A new explicit production overload consumes a training relation:

`ec_distann_build_epoch_with_training(index, epoch, build_id, training_relation)`

The relation contract is exactly two columns, `training_ordinal bigint` and
`vector real[]`. A build takes `AccessShareLock`, reads rows in strictly
ascending unique ordinal order under its active build snapshot, requires
exactly 200 finite vectors of the generation dimension, and computes a
domain-separated digest over ordinals and canonical f32 bytes. Missing,
duplicate, malformed, changed, or dimension-mismatched input fails before any
participant begins.

This API accepts a PostgreSQL relation rather than a filesystem path so normal
privilege, snapshot, locking, and deployment semantics apply. The training
relation is build input, not an epoch artifact; the immutable input digest and
selected head digest are epoch artifacts.

## Metadata and compatibility

Versioned build options add:

- `head_policy`: `current_sample_graph` or `training_landmarks_exact`;
- `training_query_count`; and
- `training_query_digest`.

New manifests/build specs/fingerprints bind these fields. The head-state row
also records them for inspection and cross-checks them against the manifest.
The selected persisted vectors and canonical head digest continue to bind the
actual query artifact.

Build-options decoding remains backward compatible: version 1 maps to
`current_sample_graph`, count zero, and an all-zero no-training digest. New
builds encode version 2. Existing generations retain their Vamana head search;
there is no silent reinterpretation. An unknown policy/version, nonzero count
with a zero digest, or policy/input mismatch fails closed.

The head graph remains persisted for both policies in this format revision so
existing lifecycle, integrity, cache, retirement, and storage accounting stay
uniform. `training_landmarks_exact` ignores the graph for normal seeding and
scores at most the manifest-bound sample cap.

## Replay and lifecycle

Before returning an existing candidate for the same build id, the trained
endpoint re-reads and hashes its declared training relation and compares policy,
count, and digest with the immutable candidate. A mismatch is
`EC_BUILD_ID_CONFLICT`; it cannot silently replay a candidate built from other
training input.

The head rows remain children of the immutable build candidate. Ready,
publication, active-pointer selection, scan-cache identity, retire, reclaim,
abort, and DROP continue to operate on the exact generation fingerprint and
existing foreign-key/lifecycle paths. No new relation handle or external file
survives the build transaction.

## Query and inspection behavior

The active manifest policy selects the production head algorithm:

- `current_sample_graph`: existing bounded Vamana head search;
- `training_landmarks_exact`: exact scoring of every persisted landmark,
  deterministically ordered by distance then vec_id and truncated to 32.

Benchmark seed-mode overrides remain feature-gated diagnostics. The normal
production read path never consults a policy GUC. Inspection and suite output
will report policy, training count/digest, sample count/digest, scoring mode,
cap, returned seed count, head bytes, and cached bytes.

## Decision boundary

This contract changes no default. Only the explicit trained build endpoint
creates the new policy. Promotion/default decisions wait for the production
10k/50k/100k A/B. Proposed NFR-017 targets remain informational rather than
hard gates.

## Implemented checkpoint

Commit `43b3ace1a` adds the explicit trained builder endpoint, canonical
training-relation digest, byte-compatible legacy options plus trained V2
options, manifest/head-state policy binding, the 0.1.1-to-0.1.2 catalog and
function upgrade surface, exact configured head scoring, and active-generation
inspection with catalog-versus-manifest cross-checks. The production endpoint
rejects a trained build unless the frozen cap is exactly 4,096 and the input is
exactly 200 valid rows.

The focused PG18 lifecycle test builds the policy twice from the same source
and training relation and obtains the same selected-head digest; exact replay
returns the immutable candidate, changed training input fails with
`EC_BUILD_ID_CONFLICT`, publication succeeds, and inspection attests policy,
scoring mode, count/digests, cap, seed bound, and sample count. Existing
benchmark-feature code also compiles after the production extraction. See
`artifacts/validation.log`.

No benchmark result is claimed in this packet. Task 182 packet 006 still owns
the required production 10k/50k/100k A/B before any promotion decision.
