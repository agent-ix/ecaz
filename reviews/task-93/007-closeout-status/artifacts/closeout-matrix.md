# Task 93 Closeout Status Matrix

## Measured (AM × corpus × ISA), all recall-byte-equal

| AM | corpus | scalar reference | NEON kernel | block coverage |
|---|---|---|---|---|
| ivf | real10k np8/np32 | 793 / 515 ns/cand | 223 / 191 ns/cand (3.55× / 2.69×) | ~99% in 32-blocks |
| ivf | real100k np32 | 364 ns/cand | 126 ns/cand (2.90×) | ~99.9% |
| hnsw | real10k/50k/100k | — (entered at NEON phase) | 230 / 171 / 137 ns/cand | partial-width (avg ~22) |
| diskann | real10k/50k/100k | — | 285 / 267 / 236 ns/cand | partial-width (avg ~10) |

Per-ISA ≥2× gate: PASS on every measured NEON cell vs the packet-002
scalar reference. End-to-end: parity-or-better at every cell after the
partial-width dispatch (packets 003/004/006, interleaved rechecks).

## Acceptance criteria status

1. Module with scalar+NEON+SVE+AVX2: scalar+NEON live and measured; AVX2
   code landed reusing production primitives (bit-equal-by-construction
   argument), runtime/measurement on the Intel lane; SVE routes through
   NEON (never reports Isa::Sve until a real kernel runs).
2. Per-AM batch routing ≥32: done for IVF/HNSW/DiskANN (+ partial-width
   for graph AMs — reviewer-confirmed semantics, packet 004).
3. Recall byte-equal every (AM × corpus) cell: PASS (9 cells).
4. ≥2× scoring share per ISA: PASS for NEON everywhere measured.
5. e2e p50/p95/p99 no regression: PASS (rechecked cells).
6. pg_test surfaces: deferred to Linux per the macOS dyld policy; live
   bench cells cover the scan paths against a real PG18 backend.
7. No unsafe outside ISA modules; # Safety docs: PASS (reviewer-verified
   through packet 005's lineage).
8. Closeout matrix: this artifact.

## Open lane decisions (explicit requests)

- **SVE / Graviton 4**: unlike Tasks 95/98, Task 93 has a real 32-block
  surface (IVF, ~99% coverage), so an SVE kernel could add value beyond
  NEON's 2.7–3.6×. Two options for the owner: (a) authorize the Graviton
  lane (packet 001's Phase C plan, snapshot+destroy); (b) accept
  NEON-routing on SVE hosts as the shipped behavior and fold the SVE
  question into Task 99's profile. The kernels are correct either way.
- **AVX2 / Intel desktop**: code is landed and bit-equal-by-construction
  with the production AVX2 batch path; the lane needs `cargo test
  rabitq32` + the packet-003-shaped suite run on that host.

Status flip to complete is appropriate once packets 005/006 are approved
and the two lane decisions above are recorded (either as runs or as
documented deferrals to Task 99).
