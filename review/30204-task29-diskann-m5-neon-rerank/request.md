# ec_diskann Apple-Silicon NEON Exact Rerank Kernel

Reviewer: please review this Apple-Silicon-specific ec_diskann checkpoint
and its packet-local A/B measurement.

## Scope

This packet measures committed head `dceda057`
(`Add NEON exact rerank inner product to ec_diskann`) against the
current `origin/main` head `e5f380a1`.

The hypothesis was the same one that produced the IVF M5 win in
`review/30201-task31-m5-quality-neon-rerank`:

- `src/am/ec_diskann/ambuild.rs::source_inner_product(...)` had an
  AVX2+FMA fast path on `x86_64` and fell back to a scalar loop on
  Apple Silicon.
- The SQL ordered scan rerank path goes through
  `routine.rs::exact_heap_rerank_distance(...)` ->
  `routine.rs::with_heap_source_vector(...)` ->
  `ambuild::source_inner_product(...)`.
- Adding an `aarch64` NEON specialization should reduce per-rerank-row
  cost on Apple hardware without changing recall.

## Code Checkpoint

- code commit: `dceda057` (`Add NEON exact rerank inner product to ec_diskann`).
- shape: 16-lane NEON main loop with four parallel `vfmaq_f32` accumulators,
  4-lane tail, scalar remainder. Mirrors
  `src/am/ec_hnsw/source.rs::inner_product_neon`, which is the kernel
  shape that produced the IVF M5 win.

Focused validation before measurement:

- `cargo check --all-targets --no-default-features --features pg18`
- `cargo test --no-default-features --features pg18 --lib am::ec_diskann::ambuild`
  (3 tests pass, including the new
  `source_inner_product_neon_matches_scalar_at_loop_boundaries` test
  that exercises the 16-lane main loop, the 4-lane tail, and the scalar
  remainder).

No broader cargo or pgrx test sweep was run for this packet; the slice
is a narrow architecture-specific math-kernel change and the required
validation target is the M5 packet rerun.

## Fixture Caveat

The Task 29 real-10k fixture (`target/real-corpus/ec_hnsw_real_10k`) was
not available locally on this Apple machine, so the packet uses a fresh
synthetic corpus generated via `ecaz corpus generate`:

- 10000 unit-sphere `dim=1536` vectors (seed `42`).
- 200 query vectors (seed `7`).

This is NOT a faithful Task 29 substitute. Synthetic vectors at this
dimension are nearly equidistant, recall@10 falls to `0.16-0.33`, and
per-query latency on the resulting index is dominated by scan + heap
fetch overhead rather than by the exact rerank kernel. The numbers
below are best read as a kernel correctness + smoke A/B, not a
quality-lane promotion signal.

Both arms ran against the same on-disk index, built once under the
scalar binary; only the loaded `ecaz.dylib` differed. See
`artifacts/manifest.md` for binary sha256s and full commands.

## Result

Recall and NDCG are identical across the two binaries, which confirms
the NEON kernel matches scalar within float tolerance (the existing
`source_inner_product_dispatch_matches_scalar` test plus the new
loop-boundary test already lock this in at the unit level):

| L | recall@10 | NDCG@10 |
|---:|---:|---:|
| 64 | 0.1650 | 0.8298 |
| 200 | 0.2665 | 0.8811 |
| 800 | 0.3260 | 0.9036 |

Latency, 200 iterations per L, concurrency=1, `--force-index`:

| L | scalar mean | NEON mean | scalar p50 | NEON p50 | scalar p95 | NEON p95 | scalar p99 | NEON p99 |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 64 | 2.23 ms | 2.27 ms | 2.18 ms | 2.22 ms | 2.51 ms | 2.57 ms | 2.92 ms | 2.76 ms |
| 200 | 2.96 ms | 2.78 ms | 2.86 ms | 2.73 ms | 3.48 ms | 3.08 ms | 4.49 ms | 3.20 ms |
| 800 | 4.12 ms | 3.83 ms | 4.03 ms | 3.77 ms | 4.77 ms | 4.13 ms | 5.21 ms | 4.88 ms |

Per-arm stddev: scalar 0.29 / 0.39 / 0.42 ms, NEON 0.25 / 0.21 / 0.30 ms.

## Interpretation

This does NOT clear the handoff promotion bar:

- L=64 mean and p50 moved by `+0.04 ms`, well inside stddev. The L=64
  p99 was actually `0.16 ms` better under NEON, but mean/p50 were
  flat-to-worse, so calling that a kernel win would be tail-only.
- L=200 mean / p50 improved by `0.18 / 0.13 ms`, p95 / p99 by
  `0.40 / 1.29 ms`. Mean / p50 improvement is comparable to scalar
  stddev (`0.39 ms`), so it is not clearly outside noise.
- L=800 mean / p50 improved by `0.29 / 0.26 ms`, p95 / p99 by
  `0.64 / 0.33 ms`. Mean / p50 improvement is again roughly stddev-sized.

The handoff explicitly says:

- "Treat mixed or noisy results as negative unless they clearly promote".
- "Do not claim a win from tail-only movement if the main metrics regress".

That applies here. On this fixture the NEON kernel is a correctness-
preserving narrow change with possibly small p95/p99 gains that I
cannot cleanly separate from run-to-run noise on a 200-iteration pass,
and it does not produce the kind of clean across-the-board win that
the IVF M5 NEON kernel produced in `30201`.

The most likely reason is structural rather than the kernel itself
being wrong. At default `rerank_budget=64`, this fixture's per-query
heap-fetch + scan cost is `~2-4 ms`, while the rerank kernel itself
(64 rows x 1536 dim) is single-digit microseconds. There just is not
enough kernel work in this fixture to surface a clean signal even at
rerank_budget defaults.

## Recommendation

Treat this checkpoint as a non-promotion on this fixture, but keep the
code change. The change is narrow, is independently unit-tested for
parity with scalar, follows the same successful template as the IVF M5
win, and is on a path that the handoff specifically called out as the
strongest current Apple-Silicon-specific candidate.

The next useful step is measurement-driven, not more polish on the same
path:

- rerun this A/B against a real 10k or 100k diskann fixture (the Task 29
  `task29_diskann_real10k` shape, or a Task 31-style real100k diskann
  prefix), where per-query rerank cost is a larger fraction of total
  query time. That should distinguish "kernel does nothing here because
  rerank is not the bottleneck on this fixture" from "kernel does
  nothing on Apple Silicon at all".
- if the kernel still does not move main metrics on a real-data
  fixture, the next Apple hypotheses (per the handoff) are:
  - exact rerank source decode overhead, and
  - heap fetch / cache locality in the rerank path,
  picked by measurement rather than speculation.

I am explicitly NOT promoting this packet on tail-only movement on a
synthetic fixture.

## Artifacts

- `artifacts/manifest.md` (full SHAs, commands, fixture provenance)
- `artifacts/install-pg18-scalar.log`, `artifacts/install-pg18-neon.log`
- `artifacts/load-diskann.log`
- `artifacts/recall-scalar-table.log`, `artifacts/recall-neon-table.log`
- `artifacts/recall-scalar-cli.log`, `artifacts/recall-neon-cli.log`
- `artifacts/latency-scalar-table.log`, `artifacts/latency-neon-table.log`
- `artifacts/latency-scalar-cli.log`, `artifacts/latency-neon-cli.log`
- `artifacts/truth_synth10k_k10.json`
- `artifacts/corpus-generate.log`, `artifacts/queries-generate.log`
