# Task 99: Cross-(AM × Quant × ISA) Block Kernel Completeness Closeout

Status: proposed (2026-06-08)
Owner: coder (to be assigned). One coder.
Priority: 2 (project-level closeout for the kernel-completeness initiative)

## Why

Tasks 93–98 ship per-quant kernel families. Each one closes its
own per-(AM × ISA) matrix. Task 99 aggregates the full
(AM × quant × ISA) completeness matrix, lands the project-level
architectural record (ADR-077), and closes the kernel-
completeness initiative as a coherent body of work.

The kernel-completeness initiative spans:

- Task 87 — `CandidateBatch` plumbing + TQ-no-QJL-4-bit kernel
- Task 91 — `QuantCodec` trait migration
- Task 92 — block kernel infrastructure + ADR-076
- Tasks 93–98 — per-quant kernel families
- Task 99 — project closeout (this task)

Without Task 99, the project ships seven independent task
closeouts with no coherent matrix view. The point of the
initiative was the completeness matrix; Task 99 produces it.

## Scope

### In scope

1. **Aggregate (AM × quant × ISA) matrix** across all Phase III
   closeouts. Rows = AMs (HNSW, IVF, SPIRE, DiskANN); columns =
   quants (TQ-4bit, TQ-2bit, TQ-QJL, RaBitQ, grouped-PQ,
   Hamming, HNSW TiledLut, HNSW Int8Approx); cells = per-ISA
   scoring-share + end-to-end deltas + recall preservation
   confirmation.
2. **Structural exclusions documented**: cells that don't exist
   (e.g., grouped-PQ on HNSW if absent, Hamming on SPIRE if
   absent) marked `n/a` with source-evidence link.
3. **f32 raw exclusion documented** as the canonical "no kernel
   needed" cell across all AMs.
4. **Per-ISA Graviton vs Intel comparison**: which quants gain
   most from SVE on Graviton 4, with measured vector length
   recorded; which from AVX2; which platform-neutral.
5. **Scoring-share vs end-to-end decoupling map**: identifies
   surfaces where scoring-share saturates (kernel does its job)
   but end-to-end doesn't move because other stages dominate
   (HNSW small-frontier cells, SPIRE pipeline-dominated paths).
   This is the honest framing of where kernel wins matter and
   where they don't.
6. **ADR-077: Block Kernel Completeness Closing Record**
   capturing:
   - the universal block-kernel pattern is now the default for
     compressed-domain scoring across the project;
   - any new quant added from this point ships its kernel
     family per the Task 92 skeleton template;
   - the (AM × quant × ISA) matrix is the project-level
     coverage gate for kernel-completeness future work.
7. **Status audit** on Tasks 93–98 confirming each per-family
   task already closed itself with a packet-local matrix; append
   cross-references where useful, but do not retroactively own
   their completion flips. Status flip on Task 99 to `complete`
   last.
8. **Memory updates**: project-level memory file documenting
   the block-kernel pattern as the operating convention.
9. **Complete index × quant × mode profile + TQ-mode
   reevaluation input (added 2026-06-09).** Beyond aggregating
   the per-task closeouts, run one suite-driven profile of all
   indexes × quants × modes on shared fixtures so the project
   can reevaluate TQ mode policy (ADR-025 bit allocation, the
   no-QJL 4+0 carve-out, whether 4+1 or a 2-bit surface earns
   adoption) on post-kernel cost data. ADR-025 deliberately
   stays PROPOSED until this profile exists; Task 96 stays
   deferred until then. Design requirements for the profile:
   - **Dimension coverage**: no-QJL exists only at dim 1536, so
     a 1536-only profile cannot exercise QJL lanes. Include at
     least one non-tiled dimension (synthetic fixture if needed)
     or the TQ-mode comparison is vacuous.
   - **Absent cells marked, not skipped**: use the Task 92
     `kernel_status` markers (`structurally_absent`,
     `missing_kernel`) so 2-bit and other unshipped cells report
     honestly.
   - **Batch-on/off as an axis** wherever a kernel path trades
     away cutoff pruning (IVF PqFastScan suffix-max bound per
     Task 94 packet 024 F1), so the profile captures the
     kernel-vs-pruning interaction rather than averaging it.
   The reevaluation decision itself (ADR-025 flip or revision,
   any new storage surface) is follow-up scope informed by this
   profile, not owned by Task 99.

### Out of scope

- New kernel work. All kernels are Tasks 93–98.
- AVX-512 variants. Follow-up if measurement post-Task-99
  justifies.
- Apple silicon (M-series) production variants. M5 NEON is for
  development validation, not production.
- Any new quant added during the initiative — those open as
  separate tasks following the same pattern.

## Acceptance criteria

1. Aggregate matrix `artifacts/cross-am-quant-isa-matrix.md`
   covers every shipped cell from Tasks 87, 93–98.
2. Per-AM behavioral parity: recall byte-equal vs pre-kernel
   baseline at every cell, citing the source closeout packets.
3. Per-AM end-to-end deltas at every cell with attribution
   (kernel-bound vs other-stage-bound).
4. Per-ISA comparison table: Graviton 4 SVE (measured vector
   length recorded) vs Intel AVX2 wins by quant.
5. Structural exclusions documented with source evidence.
6. ADR-077 PROPOSED → ACCEPTED.
7. Status audit confirms Tasks 93–98 are complete, with source
   closeout packets linked from the aggregate matrix.
8. Project memory updated.

## Phases

### Phase 1 — Source closeout aggregation

- Collect per-task closeout matrices from Tasks 87, 93–98.
- Build the aggregate `cross-am-quant-isa-matrix.md`.

### Phase 2 — Structural exclusion audit

- Walk every (AM × quant) cell. Confirm presence/absence in
  current source. Mark `n/a` where structurally absent with
  source pointer.

### Phase 3 — Per-ISA comparison + decoupling map

- Build the Graviton vs Intel comparison table.
- Identify scoring-share-saturated-but-end-to-end-flat cells.
  Map them to the AM-stage that dominates (graph traversal,
  pipeline routing, IO, etc.).

### Phase 4 — ADR-077 + status audit + memory

- Draft ADR-077 capturing the closing decisions.
- Audit Tasks 93–98 statuses and append cross-references where
  useful.
- Update project memory.

### Phase 5 — Closeout

- Reviewer approves aggregate matrix + ADR-077.
- Status flip Task 99 → `complete`.

## Per-AM validation gate

Not applicable directly — this task aggregates evidence rather
than producing new measurement. Acceptance is reviewer
agreement that the matrix is complete and the closing record
accurately represents what shipped.

## Stop conditions

- If any Phase III task (93–98) closes with unresolved Stop
  Conditions (e.g., a kernel didn't meet the ≥ 1.5× per-ISA
  gate on a given AM), Task 99 documents those explicitly
  rather than papering over them. The matrix shows partial
  cells; ADR-077 names them as the boundary of what the
  initiative delivered.


## Absorbed deferrals from Tasks 93/95/98 closeouts (2026-06-10)

- Task 93 (owner decision 2026-06-10): the Graviton 4 SVE lane — the one
  surface with real 32-wide block coverage (IVF, ~99%) where SVE could add
  value beyond NEON's 2.7-3.6x — and the Intel AVX2 compile/runtime/bench
  validation of the landed rabitq32 AVX2 backend.

- Task 95: the AVX2-vs-hardware-POPCNT question for hamming32 (Intel lane;
  expected return bounded by the measured NEON 1.10-1.17x).
- Task 98: AVX2 variants for tiled_lut32/int8_approx32 (Intel lane;
  vpmaddubsw named for int8). Key profile fact: HNSW exact-mode payoff is
  governed by partial-width behavior — >=32-wide flushes are <0.1% of the
  distribution at 10k/50k/100k.
## Coordination

- **Depends on Tasks 87, 91, 92, 93, 94, 95, 96, 97, 98** all
  reaching `complete` status before Task 99 closes.
- **Required by no follow-up task.** Task 99 is the project
  closeout.

## References

- All of Tasks 87, 91, 92, 93–98.
- `spec/adr/ADR-071-unified-quantizer-interface.md`
- `spec/adr/ADR-072-index-local-quantized-codec-adapters.md`
- ADR-076 (universal block kernel pattern — Task 92)
- ADR-077 (block kernel completeness closing record — proposed,
  authored by this task)

## Estimated size

Small-medium. 2–4 weeks for one coder, dominated by ADR-077
authorship and reviewer rounds. Phase 1–3 are aggregation,
not new measurement.
