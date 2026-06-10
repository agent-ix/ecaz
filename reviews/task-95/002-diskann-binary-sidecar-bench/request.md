# Task 95 Packet 002: DiskANN Binary-Sidecar Bench Cells

Measurement packet for packet 001's hamming32 kernels on the DiskANN
binary-sidecar prefilter (PqFastScan fixtures, `prefilter_kind=auto`).
One small code addition rides along: bench_api/criterion hooks for the
off-path scoring-share measurement.

## Commits

- `4a67d05b0` (packet 001) — code under test.
- bench hooks + criterion rows (this packet's lineage) —
  `hamming32_block32_scalar_reference` / `hamming32_block32_dispatch` in
  `bench_api` (`feature = "bench"` gated) and a
  `quant/hamming32_block32` Criterion group.

## Results (full numbers and run notes in `artifacts/manifest.md`)

- **Recall byte-equal** at both corpora (0.9938 / 0.9719, identical
  between cells).
- **Counters**: `surface=diskann quant=binary isa=neon` rows cover every
  prefilter candidate (39,703 / 74,900); kernel-off cells emit zero rows.
- **End-to-end**: kernel-on p50 faster at both corpora (10k −21%, 100k
  −3%) — driven by the words-based batch path eliminating the
  per-candidate word↔byte `Vec` conversions, not by SIMD margin.
- **Per-ISA stop-condition disclosure**: NEON measures **1.17×** (1536
  bits) and **1.10×** (12288 bits) over the scalar reference in the
  off-path Criterion bench — below the 1.5× threshold because scalar
  `u64::count_ones` is already hardware popcount on this host. Documented
  and continued per the task's stop conditions; parity is integer-exact,
  the end-to-end gate passes, and the kernel sits on the path Task 99
  consumes.
- **Process note**: the first suite run was discarded after a mid-run
  extension install (shared pgrx tree with the reviewer) split the cells
  across two builds; the cited run is sha-verified start to finish. Worth
  a convention: agents on this machine should avoid `ecaz dev install`
  while another agent's bench suite is running, and packets should record
  the installed dylib sha (this one does).

## Review request

Please review the bench evidence, the stop-condition disclosure framing,
and the bench_api hook shape. Remaining Task 95 phases follow the Task 93
lane plan (SVE → Graviton pending authorization; AVX2-vs-POPCNT question →
Intel lane; closeout after).
