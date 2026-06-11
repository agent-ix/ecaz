# Task 103 Packet 001: Intel Baseline Matrix + Two Dispositions

No code change under review. This packet is the measurement baseline for
Task 103 (`plan/tasks/103-intel-avx2-kernel-gap-closure.md`) and proposes
closing two of its four scope items on the evidence.

## Proposed disposition 1 (AC2): retire/deprioritize tiled_lut32 — no AVX2 build

On the canonical real-10k 1536-dim lane, batch-on, byte-identical recall:
tiled_lut p50 6.66 / 10.00 ms vs full_lut 4.52 / 6.76 ms (47–48% slower),
2,994 ns/candidate scalar walk vs the lut32 AVX2 kernel's 492–546 ns/c.
tiled_lut's cache rationale is void at 1536d (the LUT is L1-resident,
ADR-025), and post-Task-102 full_lut dominates it everywhere we can
measure. Building AVX2 for a losing lane is waste; proposal: mark the
tiled_lut Intel cell **retired/deprioritized** in the Task 99 matrix with
this packet as source evidence. (Whether to remove the mode GUC value is a
separate operator decision — not proposed here.)

## Proposed disposition 2 (AC3): skip hamming32 AVX2 — POPCNT stands

Scalar hardware-POPCNT scores binary candidates at **11.5–11.8
ns/candidate**; scoring is ~0.5% of DiskANN query time on the real-50k
lane, and warm-order batch-on/off end-to-end is within noise
(3.89/4.62 vs 4.00/4.51 ms). Even a perfect 2× AVX2 nibble-popcount kernel
moves ≤0.3% end-to-end. Proposal: record **skip** with these numbers,
mirroring Task 95's measured Graviton SVE scope-out. (The first-run 11.4 ms
batch-on cell was a cold-cache ordering artifact; the packet documents it
and the warm recheck that supersedes it.)

## AC1 baseline (int8_approx AVX2 kernel — proceeding)

int8_approx batch-on currently runs **entirely scalar** at 918–923
ns/candidate and loses to batch-off (4.52 vs 4.19 ms) — batching overhead
with no kernel payoff. Its scalar rate already beats lut32's scalar rate,
so the AVX2 kernel (next slice) is the highest-value remaining Intel item.
Target per the task gates: ≥2× the 919 ns/c scalar anchor.

## Review focus

1. Are the two dispositions adequately evidenced for the Task 99 matrix
   (retired + skip cells with source links), or should either lane get a
   deeper sweep first?
2. The cold-cache artifact handling (first-run cell superseded by the
   warm-ordered recheck) — acceptable, or rerun the full binary cell pair
   once more?

Remaining Task 103 slices after this packet: int8_approx32 AVX2 kernel
(AC1), rabitq32 AVX2 validation + bench (AC4).
