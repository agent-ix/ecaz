# Task 104: Apple Silicon (M5) Full Bench Matrix + NEON Optimization Lane

Status: proposed (2026-06-11; operator decision — Apple silicon is a
**supported target**, not a dev-validation-only host. Supersedes the
"M5 NEON is for development validation, not production" posture in
Task 99's out-of-scope list for everything this task owns.)
Owner: coder (M5 session). Runs on the operator's Apple M5.
Priority: 2 (parallel with Task 99 local phases; any kernel changes it
produces must land before the G4 trip — see Sequencing)

## Why

Two reasons, one practical and one strategic:

1. **G4 de-risk.** The Task 102 NEON shuffle-repack kernels, the SVE
   dispatch ladder, the DiskANN TQ batch arm, and every ARM code path
   added since the last M5 session have never executed on real
   aarch64 — they are compile-gated off x86 and the Intel desktop only
   proves they type-check. Every compile error, parity failure, or
   NEON performance regression caught on the M5 is one not debugged on
   a paid Graviton instance.
2. **Apple silicon is a supported target.** Operators run this
   extension on M-series hardware. That deserves the same evidence
   discipline as the production lanes: a complete index × quant ×
   option matrix measured on the actual hardware, with honest
   `kernel_status` markers, not an untested "NEON probably works"
   shrug. The M5 lane is recorded alongside — not in place of — the
   pinned Graviton 4 and AWS Intel production lanes.

## Scope

### In scope

1. **Day-one aarch64 validation pass** (first slice, before any
   benching): full parity/test suite on the M5 — the lut32 NEON
   transpose unit test, all per-ISA parity tests (lut32, qjl32,
   int8_approx32, rabitq32, hamming32, grouped-PQ), `candidate_batch`
   counter tests, and the DiskANN TQ batch counter test. This is the
   same smoke set planned for G4 day one; it must be green here first.
2. **Full M5 bench matrix — all indexes × all quants × all options**
   (the Task 99 item 9 thoroughness standard, applied to the M5):
   - Indexes: HNSW, IVF, SPIRE, DiskANN.
   - Quants: TQ no-QJL 4-bit (full_lut and int8_approx exact modes;
     tiled_lut only as a retired-cell confirmation), TQ-QJL, RaBitQ
     bits=1 (and the bits=4/8 storage lanes where an AM exposes them),
     grouped-PQ, binary/Hamming sidecar; f32 raw documented as the
     canonical no-kernel cell.
   - Options: candidate-batch scoring on/off at every cell where the
     GUC exists, exact-score-mode sweep on HNSW, standard parameter
     sweeps (`ef_search`, `nprobe`, `list_size`), rerank modes where
     applicable.
   - **Dimension coverage**: at least one non-tiled dimension fixture
     so the QJL lanes are actually exercised (no-QJL only exists at
     1536) — same design requirement as the Task 99 profile.
   - **Absent cells marked, not skipped**: Task 92 `kernel_status`
     markers (`structurally_absent`, `missing_kernel`, retired).
   - Counter attribution at every kernel-on cell: `isa=neon` with
     `scalar_candidates=0`, width histograms recorded.
3. **NEON optimization where the matrix says so**: any family whose
   M5 scoring-share falls below the per-ISA floor gate (1.5× vs the
   same-head scalar anchor) is in-scope to optimize on the M5, with
   the established kernel-evidence gates (parity per family contract,
   recall byte-equal where the contract is bit-exact, counters,
   end-to-end). The lut32 repack port is the first suspect to confirm
   (it replaced a measured-good v1 NEON shape with the AVX2-proven
   repack shape; if the repack regressed on real NEON silicon, fix or
   revert-to-v1 here, not on G4).
4. **Apple-specific dispatch + environment validation**: Apple
   silicon has no SVE — confirm the Sve/Sve2 detection ladder cleanly
   resolves to `Isa::Neon` and the SVE entry points stay dormant
   (`score_partial_sve` returns None); document macOS/pgrx environment
   deltas that affect benching (core pinning/QoS, scheduler effects of
   P/E cores on latency percentiles, any pager differences worth
   recording in manifests).
5. **M5 matrix deliverable**: a packet-local
   `m5-index-quant-option-matrix.md` with per-cell scoring-share,
   end-to-end deltas, recall, and kernel_status — formatted so Task 99
   can cite it as the Apple-silicon supported-target column next to
   the G4 and AWS Intel production columns.

### Out of scope

- Graviton 4 / AWS evidence of any kind (Task 99 profile lanes; the
  M5 lane informs but never substitutes for G4).
- SVE/SVE2 work (cannot execute on Apple silicon).
- AVX-512 and all Intel work (Task 103 closed the Intel column).
- New quant families or storage formats.
- macOS packaging/distribution work (this task is bench + kernel
  evidence, not release engineering).

## Acceptance criteria

1. Day-one parity/test suite green on the M5 (logs packeted) before
   any bench cell is cited.
2. Full index × quant × option matrix complete with `kernel_status`
   markers for every absent/retired cell and a non-1536-dim fixture
   exercising the QJL lanes.
3. Every kernel-on cell shows `isa=neon`, `scalar_candidates=0`, and
   a width histogram; recall byte-equal at every cell whose family
   contract is bit-exact (tolerance families per ADR-076 documented
   explicitly).
4. Every family at ≥1.5× scoring-share vs the same-head scalar anchor
   on M5, or a documented stop-condition/optimization outcome.
5. Any kernel code changes are landed on main and reviewed **before**
   the G4 evidence trip (single-trip economics), with Intel
   regression checks (the Task 103 cells) re-run for any shared-code
   change.
6. The M5 matrix is citable from Task 99 as the Apple-silicon
   supported-target column.

## Sequencing and coordination

- Runs on the operator's M5, in parallel with Task 99's local phases
  (aggregation, ADR-077 drafting, profile SuiteConfig authoring) on
  the Intel desktop. Separate branches; no overlapping file edits.
- **Hard gate: M5-driven kernel changes land before the G4 trip.**
  A kernel change after the trip invalidates paid ARM evidence — the
  same rule that pinned the quantized-LUT deferral.
- The pg_test debug-install trap applies on the M5 exactly as on the
  desktop: `ecaz dev install ecaz-pg-test --pg 18` (release,
  SHA-asserted) → restart → `ecaz_build_profile()` probe before every
  bench run; no cargo test between install and bench. Suite preflight
  enforces the release backend.
- All benches suite-driven (FR-038) with packet-local SuiteConfigs and
  manifests under `reviews/task-104/`. M5 fixtures: the staged DBpedia
  corpora under `data/task31_m5_dbpedia_staged/` (real 10k/50k/100k)
  plus synthetic fixtures for QJL/non-1536 and any missing
  storage-format lanes (own prefixes, one index per table).

## References

- Task 99 (production closeout; this task supplies its Apple-silicon
  column and removes M-series from its out-of-scope shadow)
- Task 102 packets 001/002 (the NEON repack port under first real
  validation here)
- Task 103 packets 001–003 (Intel precedent for the gap-closure +
  evidence pattern)
- Task 92 / ADR-076 (kernel_status markers, tolerance lanes)
- `docs/block-kernel-development.md`

## Estimated size

Medium. One coder-session on the M5: 1–2 days for validation + matrix
if nothing regresses; longer only if NEON optimization work is
triggered by the floor gate.
