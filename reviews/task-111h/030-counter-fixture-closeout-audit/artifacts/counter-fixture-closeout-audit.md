# Task 111h Counter and Fixture Closeout Audit

This packet audits already-committed Task 111h code and packet evidence. It
does not add new validation. The goal is to avoid marking checklist rows closed
from chat memory alone.

## Checklist Mapping

| Task row | Evidence | Status |
| --- | --- | --- |
| `207`: Add EXPLAIN/admin/counter coverage for placement, format, payload bytes, pages read, decode time, and scoring time. | `src/am/common/explain.rs:523`, `527`, `627`, `631`, `639`, `651`, `657`, `663`, `669`, `675`, `681`, `685`; recorders at `explain.rs:473-514`; source/index recording in `src/am/ec_ivf/scan.rs:2558-2567`, `2647-2653`, `2683-2685`, `2717-2726`; debug snapshot fields at `scan.rs:3868-3983`; admin fixture at `src/tests/ec_ivf.rs:1236-1283`; counter fixture at `src/tests/ec_ivf.rs:1424-1593`; packets `006`, `009`, `011`, `014`; reviewer feedback `016/feedback/2026-06-20-01-reviewer.md`. | Covered for the IVF EXPLAIN plus admin/debug surfaces used by Task 111h. |
| `209`: Add PG18 correctness fixtures for create/insert/update/delete/vacuum, mixed old/new postings, and snapshot-visible rerank payloads. | Create/build/admin snapshot: `src/tests/ec_ivf.rs:1236-1283`. Persisted compact/no source conversion/counters: `1424-1593`. Live insert: `1596-1662`. Mixed direct-pointer/full-chain fallback: `1665-1748`. Partial final group: `1751-1815`. Delete/vacuum tombstone and live survivor rerank: `2223-2294`. Packets `009`, `010`, `014`, `015`, `016`; reviewer feedback `016/feedback/2026-06-20-01-reviewer.md`. | Partially covered. Build/create, insert, delete/vacuum, mixed fallback, partial groups, and persisted compact bytes are covered. I did not find a dedicated update-path fixture or explicit MVCC/snapshot-visible rerank payload fixture, so the checklist row should remain open for those cases. |
| `205`: Implement or explicitly benchmark away owned per-survivor payload copies and double-copy batch-scoring slabs in the compact index path. | Current batched path allocates `payload_slab` at `src/am/ec_ivf/scan.rs:2661`, copies payloads at `2673`, and records copied bytes at `2683-2685`. The rabitq4 fixture asserts `rerank_payload_slab_bytes_copied == rerank_payload_bytes_scored` at `src/tests/ec_ivf.rs:1557-1565`. | Open. This was instrumented but not eliminated or benchmarked away. |

## Counter Coverage Details

EXPLAIN coverage is not inferred from benchmark logs. The properties are present
in the scan explain surface:

- placement and format: `Rerank Placement`, `Rerank Format`
- timing: `Rerank Payload Decode Elapsed Us`, `Rerank Payload Score Elapsed Us`
- source bytes: `Rerank Source Bytes Read`
- index-side page/byte reads: `Rerank Index Group Header Pages Read`,
  `Rerank Index Payload Segment Pages Read`,
  `Rerank Index Group Metadata Bytes Read`,
  `Rerank Index Header Payload Bytes Read`,
  `Rerank Index Segment Payload Bytes Read`
- scored/copy bytes: `Rerank Payload Bytes Scored`,
  `Rerank Payload Slab Bytes Copied`

The PG18 debug snapshot mirrors those fields, which lets tests assert that
source/f32 uses heap source bytes while compact index-side formats score
persisted payload bytes without source-vector rereads.

## Fixture Coverage Details

The committed PG18 fixture set covers the main persisted-index lifecycle:

- `test_ec_ivf_index_placement_compact_admin_snapshot` creates each compact
  index placement format through the common admin surface.
- `test_ec_ivf_index_placement_fewer_rerank_bytes` compares source/f32 against
  index f16, rabitq4, and rabitq8 on the same rerank frontier and asserts zero
  source-vector reads for the compact index-side variants.
- `test_ec_ivf_index_placement_insert_maintains_packed_group` verifies an
  after-build insert is reranked from its packed payload and ranks first.
- `test_ec_ivf_index_placement_mixed_fallback_chain` forces a missing direct
  group TID and verifies full-chain fallback returns the same outputs as the
  direct hot path.
- `test_ec_ivf_index_placement_partial_final_group` verifies a final group with
  `valid_count < rerank_width` emits and scores only valid slots.
- `test_ec_ivf_index_placement_vacuum_tombstones_packed_group_slot` verifies a
  vacuumed packed slot is not returned while surviving rows still rerank from
  packed payloads.

I did not find an equally direct fixture for an UPDATE that changes a vector and
then proves the compact rerank payload associated with the visible tuple version
is refreshed. I also did not find an explicit MVCC snapshot fixture that holds a
snapshot across concurrent tuple changes and proves compact rerank payload
visibility matches heap tuple visibility. Those remain real correctness gaps
for the wording in task row `209`.

## Review Evidence

The outside review in
`reviews/task-111h/016-rerank-partial-final-group/feedback/2026-06-20-01-reviewer.md`
verified packets `009-016` and `020`, including insert chain relink, mixed
fallback, partial final groups, cycle guard, byte counter semantics, and
decode/score timing split. This audit relies on that review for the conclusion
that those already-reviewed fixes are correct, but it does not extend that
review to the still-open update/MVCC and copy-cost gaps.

## Remaining Non-Closed Items

This audit leaves these Task 111h follow-ups open:

- legacy `0x2A` direct-TID sidecar benchmark baseline
- table-owned persisted compact payload storage evidence or replacement
  rationale
- owned per-survivor payload copy / batched slab-copy cleanup or benchmark-away
  evidence
- update-path compact payload fixture
- explicit MVCC/snapshot-visible compact rerank payload fixture
- cold/remote evidence, if final promotion claims need to generalize beyond
  the committed warm-cache local suites
