# Task 222 completion audit

Audit date: 2026-08-23. This is a coder-side evidence audit, not an outside
review verdict. Authoritative implementation behavior is `c9f79be4a`; the
suite reuse-attestation correction is `f1351d2db`.

## Plan-review findings

| Finding | Required outcome | Current evidence | Disposition |
| --- | --- | --- | --- |
| 001-1: ordering operand | Mechanically exclude only the sole proved resjunk sort operand; retain it for visible/qual/ambiguous use | `payload_projection.rs` performs the exact operator/Var/query-value matcher; `custom_scan.rs` derives against the final executor tree, copies the scan/target list, elides exactly one private entry, and re-derives without the exemption on any mismatch. Packet 002's focused PG18 log executes id-only, visible-distance, qual-use, generic, repeated, and correlated cases. Packet 003/004 counters prove the standard mask is id-only. | Satisfied |
| 001-2: system columns | Preserve `EC_UNSUPPORTED_PROJECTION`, never all-column fallback | `collect_relation_attributes` errors for negative attnums; the focused test executes `ctid`, `xmin`, and row-locking cases and passes. | Satisfied |
| 001-3: concrete derivation trees | Enumerate final `plan.targetlist`, `plan.qual`, and `custom_exprs[0]`; fail closed for unsupported query-value shapes | `derive_payload_attribute_mask` consumes exactly those surfaces; unsupported/null/unproved expression shapes select typed all-column fallback. | Satisfied |
| 001-4: reusable typed API | Export and retain `Exact(attnums)` versus `AllColumns(reason)` | `PayloadAttributeMask` is typed and stored in `DistannCustomScanExecState`; downstream Tasks 223/229 can distinguish a proof from a fallback. | Satisfied |

The non-blocking plan notes are also closed: the isolated gate is numerical
(at least 1 ms or 5%), result identity is byte-level, EPQ/concurrent update is
in the focused test, and the zero-payload classification records that the row
fetch remains the visibility/tombstone check.

## Packet 002 review findings

Feedback sequence 03 accepted the restoration of `PARAM_EXEC`/LATERAL and the
executor-local copy direction, while leaving the mask/projection invariant
open. Commit `c9f79be4a` closes that invariant at one BeginCustomScan decision
point: omission advances only after a private copy is re-derived and exactly
one target is elided; otherwise the vector is retained. It also removes the
temporary trace logging, reports actual projection status, and gates both mask
narrowing and elision behind the benchmark control. The focused PG18 rerun at
that SHA passes 1/1 with 2,578 tests filtered; packet 002's manifest routes the
passing log and the separately landed expression-context and snapshot-lifetime
fixes.

## Task acceptance criteria

| AC | Required proof | Authoritative evidence | Result |
| --- | --- | --- | --- |
| 1 | Test-pinned exact/fail-closed derivation and system-column hard error | Packet 002 request, manifest, `pg18-focused-rescan.log`, and implementation `c9f79be4a` | Proven |
| 2 | Control-equivalent semantic/failure behavior, including historical qual hazard and ordering adversaries | Packet 002 covers visible/qual/whole-row/system-column, null/toast, local/remote, cached/generic Params, LATERAL rescan, multi-window rejection, EPQ/update, and remote failure; packet 003 independently passes all nine materialization scenarios | Proven |
| 3 | Same-generation 100k gate with byte-identical ids/order and useful latency/byte reduction | Packet 003: recall 0.9265/0.9265, common prediction SHA `156fc23a84231be13a193b9b7406181f5bef386941e6ad3535cdb5ef537e525b`, warm 17.1 -> 10.7 ms (-37.43%), payload 167,404.76 -> 66.6 B/scan | ADVANCE proven |
| 4 | 10k/50k/100k recall, latency, storage, topology, and NFR evidence | Packet 004's three successful suite manifests, three normalized results JSONLs, direct logs, summaries, predictions, head membership, and `decision.md` | Proven |

## Full-matrix admissibility

Every scale has one fresh physical generation shared by its A/B arms, a clean
release extension attested unanimously at `c9f79be4a`, three physical owners,
zero non-owned records, zero orphan vectors, zero coordinator-resident
unsharded payload, and arm-identical storage. Control/candidate ordered
prediction files have the same SHA within each scale. The sole variant delta
is `payload_projection=false/true`, satisfying the registered NFR-021 and
same-generation NFR-022 contracts.

No implementation, test, benchmark, artifact, cleanup, or status-bookkeeping
deliverable remains on the coder side. Packets 002-004 intentionally remain
review-open until an outside reviewer records a verdict.
