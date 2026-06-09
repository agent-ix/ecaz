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
   most from SVE-256; which from AVX2; which platform-neutral.
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
7. **Status flips** on Tasks 93–98 to `complete` referencing
   this task; status flip on Task 99 to `complete` last.
8. **Memory updates**: project-level memory file documenting
   the block-kernel pattern as the operating convention.

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
4. Per-ISA comparison table: Graviton 3 (SVE-256) vs Intel
   (AVX2) wins by quant.
5. Structural exclusions documented with source evidence.
6. ADR-077 PROPOSED → ACCEPTED.
7. Status flips on Tasks 93–98 to `complete`.
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

### Phase 4 — ADR-077 + status flips + memory

- Draft ADR-077 capturing the closing decisions.
- Flip Tasks 93–98 statuses.
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

## Coordination

- **Depends on Tasks 87, 91, 92, 93, 94, 95, 96, 97, 98** all
  reaching `complete` status before Task 99 closes.
- **Required by no follow-up task.** Task 99 is the project
  closeout.

## References

- All of Tasks 87, 91, 92, 93–98.
- ADR-071 (unified quantizer interface)
- ADR-072 (index-local codec adapters)
- ADR-076 (universal block kernel pattern — Task 92)
- ADR-077 (block kernel completeness closing record — this task)

## Estimated size

Small-medium. 2–4 weeks for one coder, dominated by ADR-077
authorship and reviewer rounds. Phase 1–3 are aggregation,
not new measurement.
