# Task 202: ec_distann Cross-ISA Ordered Identity

Status: **proposed — portability gate** (2026-07-29). Priority: P2
correctness/release follow-up after Task 199.

Task 199 explicitly waived cross-ISA comparison of one shared generation's
ordered `(distance, vec_id)` sequence. This task closes that waiver without
changing production behavior. It is a portability and release-evidence task,
not a latency optimization.

## Goal

Prove that the same canonical ec_distann generation, query identities, release
configuration, and ordering contract produce identical ordered results on
PG18 x86_64 and aarch64 hosts for both owner traversal and the Task 199
coordinator traversal replica.

The comparison must use one generation identity, not two independently built
graphs whose construction differences could be mistaken for ISA behavior.
If PostgreSQL relation files cannot safely move between the hosts, export and
restore the immutable generation through the extension's canonical
format-preserving surface while retaining one attested generation digest and
recording the import path. The task must not silently compare different
generations.

## Entry contract

- Task 199's accepted PG18 release and current replica/owner semantics are the
  control.
- Both hosts use the same committed extension SHA, release profile, PG18
  major version, corpus/query identities, head policy, BW/H, neighbor codec,
  final ranking, and tie-break rule.
- The generation manifest, canonical record digest, epoch identity, replica
  content digest, and query fixture are attested before measurement.
- Any host, profile, generation, query, or policy mismatch invalidates the
  comparison rather than being normalized after the fact.

## Validation matrix

Run the same ordered-result comparison for x86_64 and aarch64 at 10k/50k/100k
using checked-in `ecaz bench suite` configurations. At each scale compare:

1. owner traversal versus owner traversal;
2. coordinator replica versus coordinator replica;
3. owner versus replica on each ISA;
4. exact `(distance, vec_id)` sequences for at least top-k 10, 32, and 100;
5. distinct recall and paired result identity against the same ground truth;
6. deterministic tie ordering, NaN/zero/near-tie cases, and empty/short result
   cases; and
7. missing, corrupt, stale, or unsupported-generation behavior, which must
   fail closed consistently on both ISAs.

The primary portability verdict is ordered identity, not merely recall. A
recall match with a different ordering is a failure for the Task 199 contract.
Report any distance ulp differences separately and state whether they cross a
top-k boundary or change the deterministic `(distance, vec_id)` order.

## On-disk and lifecycle checks

- Decode every relevant immutable generation record and replica record on both
  ISAs through the normal PG18 read path.
- Verify canonical wire/page fields, endian-explicit values, lengths, offsets,
  quantizer metadata, epoch/fingerprint identity, and replica content digest.
- Exercise build/import, restart, owner fallback, replica selection, stale
  invalidation, and reclaim/rebuild labeling on both hosts.
- Confirm that a portability failure cannot silently promote a replica or
  produce partial-result success.

Any format, serialization, quantizer, or final-order change required to pass
the gate is a separate implementation task and ADR/spec change; do not patch
around the mismatch inside this evidence task.

## Decision

Pass only if both ISAs produce identical ordered top-k results for the shared
generation and all failure/lifecycle cases remain fail-closed. If a difference
is found, classify it as canonical serialization, decode arithmetic, SIMD
rounding, graph/replica content, or final-order tie-breaking before proposing a
fix. A negative result is a valid closeout, but it must identify the owning
follow-up and block any cross-ISA portability claim.

The task changes no production default, stored format, replica policy, or
latency path by itself.

## Required review packets

1. `reviews/task-202/001-shared-generation-contract/`;
2. `reviews/task-202/002-cross-isa-decode-and-identity/`;
3. `reviews/task-202/003-lifecycle-and-release-verdict/`.

## Non-goals

- Independent x86_64 and aarch64 graph builds as the primary comparison;
- new SIMD optimization or quantizer tuning;
- changing the Task 199 replica storage/lifecycle contract;
- claiming paper-level cross-ISA portability from recall alone; and
- selecting a latency optimization, which belongs to Task 201.

## References

- Task 199 productionization and its explicit cross-ISA waiver;
- Task 42 on-disk format invariants;
- Task 47 recall/cost gates;
- Task 48 build matrix and soak;
- ADR-085 / ADR-086; and
- NFR-007, NFR-017, NFR-018, and NFR-020.
