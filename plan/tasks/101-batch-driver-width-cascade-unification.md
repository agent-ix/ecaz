# Task 101: Batch-Driver Width-Cascade Unification

Status: complete (2026-06-10, closeout
`reviews/task-101/004-release-ac5-rerun/` — reviewer approval
`50a86029c` "Task 101 ACs all met": generic width-cascade unification
landed, full-coverage sub-width dispatch backported to lut32/grouped-PQ,
release-backend AC5 rerun verified cell-by-cell with batch-on faster at
all six IVF cells and the debug-backend suite guard approved. Merged to
main via PRs #22–#24. Status line flipped 2026-06-11; the flip commit
was missed at closeout time.)
Owner: coder (to be assigned; the Task 93/95/98 partial-width author is
the natural fit). Phase III.
Priority: 1 (closes the epic's own consistency debts before closeout)

## Why

The 91–98 consolidation left four generations of sub-width batch
dispatch coexisting in `src/am/common/candidate_batch.rs` (1,951
lines, seven hand-rolled `score_*_batch_for` drivers):

| Generation | Families | Tail handling |
|---|---|---|
| 1 | lut32, grouped_pq_block | 32-blocks only; tails scalar |
| 2 | qjl32 | 32-blocks + 8-octets |
| 3 | rabitq32 | partial SIMD 1..=31 |
| 4 | tiled_lut32, int8_approx32 | arbitrary-width runs |

The generation-1 cost is measured: DiskANN grouped-PQ runs at ~3% SIMD
coverage (6.5k kernel vs 145k–273k scalar candidates, Task 94 packet
025). The copy-divergence cost is also measured: the
no-partial-score-write prevalidation contract (Task 94 packet 012)
landed in five of seven drivers and silently missed the original
lut32/no-QJL driver.

Source of record:
`reviews/task-99/000-pre-closeout-architecture-review/feedback/`
(findings F1, F2, F3, F6).

## Scope

### In scope

1. **Generic width-cascade batch driver** in
   `src/am/common/candidate_batch.rs`: prevalidate → ⌊n/32⌋ blocks →
   octet/partial sub-width → scalar remainder, parameterized by
   (validator, block scorer, partial scorer, `QuantCodecKind`).
   Full-coverage dispatch becomes the default property of the shared
   driver, not a per-family choice. All seven families migrate onto
   it; per-family code reduces to a parameter block plus its kernel.
2. **Partial/octet dispatch backport** to lut32 and grouped_pq_block
   (the gen-1 families). Use the flush-width histogram
   (`record_flush_width`) to record predicted-vs-achieved coverage in
   the packet evidence.
3. **Prevalidation backport (F3)**: the no-partial-write contract
   holds for every family by construction once the driver owns it.
4. **Counter kinds for Task 98 modes (F2)**: add
   `QuantCodecKind::TurboQuantTiledLut` and
   `QuantCodecKind::TurboQuantInt8` (QUANT_COUNT 5→7); tiled/int8
   batch helpers record their own kinds. Task 87 compat filter is
   unaffected (keys on `TurboQuant` only). Follows the
   `TurboQuantQjl` precedent (Task 97 packet 002).
5. **File split**: `candidate_batch.rs` →
   `candidate_batch/{mod,counters,drivers}.rs` (or equivalent), with
   the counter matrix and the driver separated.

### Out of scope

- Kernel inner-loop changes (Task 94's F8 shuffle-repack slice owns
  the grouped-PQ kernel rate; this task only fixes how kernels are
  fed).
- New quant modes, new storage surfaces.
- HNSW `quantizer.rs` extraction (separate follow-up; ADR-041).

## Acceptance criteria

1. One shared driver; seven families migrated; no behavioral change
   outside sub-width dispatch (bit-exact per family against each
   family's existing anchor contract).
2. lut32 + grouped_pq sub-width coverage measured on the existing
   local fixtures: DiskANN grouped-PQ SIMD coverage from ~3% to ≥80%;
   SPIRE/HNSW lut32 tails correspondingly reduced. Counter rows
   attribute partial runs under the dispatched ISA per the
   ratified semantics (Task 93 packet 004 / Task 97 packet 016/018).
3. Shape-error prevalidation proven for every family (malformed
   mid-batch candidate scores nothing, counters untouched) — extend
   the Task 94 packet 012 test pattern across families.
4. `quant=turboquant_tiled_lut` / `quant=turboquant_int8` direct rows
   visible end-to-end (SQL snapshot → CLI lines → suite results.jsonl)
   with the Task 87 compat line unchanged.
5. End-to-end no regression beyond noise on the existing local suite
   fixtures for every AM × family cell the packets already measure.
6. Recall byte-equal at every measured cell.

## Sequencing

In-epic, before the Graviton 4 evidence passes: the ARM evidence
should be collected once, against the final dispatch shape. The Task
94/97 G4 runbooks remain valid; their execution packets record the
post-101 head.

## References

- reviews/task-99/000-pre-closeout-architecture-review/ (F1/F2/F3/F6, F8 split)
- Task 93 packet 004 (partial-width precedent + counter semantics)
- Task 97 packets 016/018 (octet precedent + ladder outcome)
- Task 94 packets 012 (prevalidation), 025 (coverage measurements)
- ADR-076; ADR-077 (to be authored by Task 99)
