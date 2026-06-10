# Task 95 Packet 003: Closeout Matrix + SVE Scope-Down

No new code. Aggregates packets 001/002 against the task's acceptance
criteria (`artifacts/closeout-matrix.md`) and makes two calls for review:

1. **Scope down Phase C (Graviton SVE)** on measured grounds: hardware
   popcount in the scalar path bounds any same-algebra SIMD variant well
   below the 1.5× per-ISA bar (NEON: 1.17×/1.10×), and the consuming
   surface batches at ~10 candidates average. Mirrors the accepted Task 96
   stop-condition and Task 98 Phase A scope-down precedents.
2. **Close Task 95 with the AVX2-vs-POPCNT question as the sole deferred
   measurement** (Intel lane), per the criterion-4 stop-condition carve-out
   ("document and continue, do not back out") — the kernels preserve
   integer-exact parity, recall is byte-equal, and end-to-end improves.

If accepted, the status flip to complete (with the documented deferral)
follows reviewer approval of packets 001–003; happy to restructure if the
reviewer prefers holding the flip for the Intel measurement.
