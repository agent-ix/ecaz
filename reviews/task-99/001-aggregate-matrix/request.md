# Review request — Task 99 packet 001: aggregate (AM × quant × ISA) matrix

- Task: 99 (cross-AM × quant × ISA block kernel completeness closeout)
- Phase: 1 (source closeout aggregation) + 2 (structural exclusion audit)
- Coder: Task 102/103 author lane
- Date: 2026-06-11
- Head: `df2765e32`

## What this packet contains

`artifacts/cross-am-quant-isa-matrix.md` — the AC1 deliverable: the
aggregate completeness matrix covering every shipped cell from Tasks 87,
93–98 plus the in-epic additions (101, 102, 103) and the Apple-silicon
supported-target column (104). The AC5 structural-exclusion audit is
folded into the same document (§4) since the cell walk is shared, with
f32 raw documented as the canonical no-kernel cell (§3, AC item 3).

No new measurement: pure aggregation with per-cell source citations.
Spot-verified the load-bearing numbers against source artifacts (see
`artifacts/manifest.md`).

## Specific review asks

1. **Completeness**: is any shipped cell from Tasks 87/93–98/101–104
   missing from §2? The structural-exclusion table (§4) intends to cover
   every remaining (AM × quant) combination — flag any cell that is
   neither in §2 nor §4.
2. **The two M5-surfaced gaps** (§6 items 3–4): SPIRE PqFastScan product
   gap and HNSW grouped-PQ coverage gap. This packet records them as
   open items feeding an owner decision; confirm that's the right
   treatment for the closeout (vs. blocking).
3. **ISA column framing** (§5): avx2 and neon marked complete,
   sve2/G4 marked as the single remaining column pending the trip.
4. Accuracy of any citation you spot-check.

## What follows (next packets)

- 002: per-ISA comparison + scoring-share vs end-to-end decoupling map
  (Phase 3, local+M5 data with explicit G4-pending column).
- 003: ADR-077 draft (Phase 4) — carries the F2/F4/F5 paragraphs the
  pre-closeout review (packet 000) assigned to it.
- 004: index × quant × mode profile SuiteConfig + local validation
  (item 9), then the single AWS trip (G4 + Intel lanes).
