# Task 104 packet 001 — day-one aarch64 validation pass + qjl32 NEON production-scorer alignment

- Task: `plan/tasks/104-apple-silicon-m5-bench-optimization-lane.md` (scope item 1, acceptance criterion 1)
- Branch: `task-104-m5-bench-optimization`
- Code commit under review: `16133580a`
- Base: latest main `2dcbb8cc0` (post Task 101/94-F8 width-cascade merge)
- Host: Apple M5 Pro, `aarch64-apple-darwin`
- Artifacts: `artifacts/` (see `artifacts/manifest.md`)

## Summary

Ran the Task 104 day-one validation pass — the G4 day-one smoke set plus a
full single-threaded `--lib` sweep — on the M5. This is the first real
aarch64 execution of every ARM path landed since the last M5 session.
Three findings, all root-caused and fixed in `16133580a`; the suite is now
green: **1484 passed / 0 failed** (`--skip pg_test`, single-thread), clippy
pg18 `-D warnings` clean.

## Finding 1 (the substantive one): qjl32 NEON production scorer off-contract

`b0efa19d9` ("Align Task 97 qjl32 production dispatch tolerance") reshaped
the **AVX2** production scorer from multi-accumulator FMA to a
scalar-order mul + sequential-add shape and added dispatch-tolerance tests
gated at 4 ulp / 1e-6. The **NEON** scorer never got that treatment — it
kept exactly the shape the commit removed, and only ever type-checked on
x86 hosts. On the M5:

- the two dispatch-tolerance tests failed at 12 ulp / ~1.27e-6;
- the new 1000-candidate diagnostic quantifies the divergence of the old
  shape: **max 13600 ulp, max rel 9.4e-4, 298/1000 gate violations**
  (dim=1024, bits=4). Production QJL scoring on Apple silicon was
  meaningfully off the family's tolerance contract.

Fix mirrors the AVX2 precedent: 3-bit lane reshaped to scalar accumulation
order (NEON index-decode and vector multiplies kept), old shape preserved
as `score_ip_from_parts_neon_multi_accum_pre_task104_for_test` with a
print-only diagnostic, plus a hard 1000-candidate parity test on the
aligned path. Review focus: the reshape keeps per-lane product rounding
identical and only changes accumulation order; the `else` (non-3-bit) lane
and scalar tail are untouched.

## Finding 2: x86-biased ISA assert in qjl32 dispatch test

`qjl32_block32_matches_pre_slice_scorer_bits` asserted
`matches!(isa, Isa::Scalar | Isa::Avx2)` after production dispatch — fails
on any ARM host (would have failed G4 day one too). The NEON tolerance
loop above it passed at 4 ulp, so this was assert-only. Now accepts
`Neon | Sve2`.

## Finding 3: two stale tests broken on main independently of aarch64

- `ec_ivf` pq_fastscan counter test hardcoded the pre-F8 32-kernel/7-scalar
  split. Today's Task 101/94-F8 merge gives grouped-PQ full-coverage
  sub-width dispatch (39/0 on SIMD hosts); the shared `candidate_batch`
  test was updated to the ISA-aware pattern but this one was missed. Fixed
  to the same pattern. Note: the sibling SPIRE quantizer tests still pass
  with 32/7 because that surface's tail genuinely stays scalar.
- `ec_spire` payload-cap preflight test sent 65 PIDs against a cap that
  `553cd24ec` (2026-05-28) raised from 64 to 128 — broken on **every**
  host since, exposed by the full sweep. Now derives the oversized count
  from the cap accessor.

## Suite caveats (recorded for the M5 lane)

- `pg_test_*` live-PG tests excluded on macOS (known pgrx runtime
  blocker; Linux/G4 coverage).
- Multi-threaded `--lib` runs panic with "postgres FFI may not be called
  from multiple threads" in parallel-build/scan tests and poison the
  shared counter-test mutex, cascading PoisonErrors into unrelated tests.
  Single-thread (`--test-threads=1`) is the reliable full-sweep mode on
  this host; fresh-process runs confirm every cascade victim passes.

## SVE dormancy datum (scope item 4, partial)

`qjl32_sve_block32_matches_pre_slice_scorer_tolerance_when_available` and
the lut32/grouped-PQ SVE `when_available` hooks early-return on the M5
(no SVE), and the qjl32 production dispatch resolved to `Isa::Neon` —
first on-silicon confirmation that the Sve/Sve2 ladder stays dormant on
Apple. Full dispatch-ladder validation lands with the bench-matrix packet
(counter evidence at `isa=neon`).

## Sequencing note

This commit changes a production scoring path (NEON-only code). Per
acceptance criterion 5 it must land on main before the G4 trip. No shared
(x86) code is touched, so the Task 103 Intel cells do not require a
re-run; the AVX2 scorer byte-diff is empty.
