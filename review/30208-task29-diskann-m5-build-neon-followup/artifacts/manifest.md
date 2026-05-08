# Artifact Manifest

Packet: `review/30208-task29-diskann-m5-build-neon-followup`

Lane: ec_diskann Apple-Silicon NEON build-time A/B + reviewer
suggestion 2 / 3 disposition.

Hardware: Apple M5 (`aarch64-apple-darwin25.2.0`), local PG18 18.3
(Homebrew), socket `/Users/peter/.pgrx`, port `28818`.

## Code SHAs

- scalar baseline: `e5f380a1` (current `origin/main`), installed
  binary sha256 `fc71290a464ddaefe6559f6485bc6c834be3f3bdc2c0e525cdb74b05779ecb7d`.
- NEON: branch `ec-diskann-apple-neon-rerank` post-`64645bfc`,
  installed binary sha256
  `20d6c4e2d2c9839bddd334f61c6f139147a71ec4d3f12e0a35400f7646509cd4`.

The NEON binary is the same code state as `30205`'s post head plus
the `30207-01` test tightening (`64645bfc`). The kernel under test
is `ambuild::source_inner_product`, used both at build time
(`build_and_persist_vamana` distance closure) and at query time
(`exact_heap_rerank_distance`). At build time it runs many more dot
products than at query time, so the reviewer's hypothesis was that
the build-time effect should be larger than the query-time effect.

## Fixture

Same `m5_diskann_real10k` real DBpedia-style 1536d 10k-row fixture
used in `30205` / `30206`. Three new prefixes were created back-to-
back so the build elapsed numbers are not contaminated by warmup or
ordering effects:

- `m5_diskann_real10k_neon_build` — built under NEON binary.
- `m5_diskann_real10k_scalar_build` — built under scalar binary.
- `m5_diskann_real10k_neon_build_2` — second NEON build for
  stability.

All three use identical reloptions: `graph_degree=32`,
`build_list_size=100`, `alpha=1.2`. The same corpus / queries TSVs
under `fixtures/m5_diskann_real10k/`.

## Build A/B result

| build | code | elapsed | speedup vs scalar |
|---|---|---:|---:|
| `m5_diskann_real10k_scalar_build` | `e5f380a1` (scalar) | `32.61 s` | 1.00x baseline |
| `m5_diskann_real10k_neon_build` | NEON | `6.74 s` | `4.84x` (`-79.3%`) |
| `m5_diskann_real10k_neon_build_2` | NEON | `6.89 s` | `4.73x` (`-78.9%`) |

NEON mean `6.81 s` vs scalar `32.61 s` => `4.79x` speedup, `-79.1%`
elapsed. Both NEON passes are within `0.15 s` of each other (~2%
variance), well outside any plausible system-noise band that would
explain the scalar / NEON gap.

## Recall correctness check

| index | L=64 recall@10 | L=200 recall@10 | L=800 recall@10 |
|---|---:|---:|---:|
| `m5_diskann_real10k_scalar_build` | 0.9965 | 0.9970 | 0.9975 |
| `m5_diskann_real10k_neon_build` | 0.9965 | 0.9970 | 0.9975 |

Bit-identical recall and NDCG (`0.9999`) between the two arms. The
NEON kernel produces an index of the same effective quality as the
scalar reference at this fixture, so the build-time speedup is not
a quality-vs-speed tradeoff.

## Reviewer suggestion disposition

### Suggestion 1 — measure Apple-Silicon build impact

Addressed above. Real, repeatable Apple-Silicon win at build time:
**`32.61 s -> 6.81 s`, `4.79x`, `-79.1%`**. This is the largest
Apple-Silicon-specific effect from the NEON kernel measured so far,
roughly an order of magnitude larger than the rerank-time effect on
the kernel-stress lane in `30205` (`-1.8 ms` p50 = `-11%`).

The kernel itself is unchanged from `dceda05`; this packet only
records the previously-unmeasured build path. No new code change is
needed; the win is already delivered by the existing committed NEON
specialization once it is installed.

### Suggestion 2 — group rerank fetches by heap block; consume same-page runs while pinned

Investigated; deferred with prerequisites recorded.

The current rerank loop calls `table_tuple_fetch_row_version` per
row through `scan_state::fetch_heap_row_version`. Holding a buffer
pin across a same-page run would require bypassing
`table_tuple_fetch_row_version` and rolling a manual fetch path:

- `ReadBufferExtended(...)` + `LockBuffer(BUFFER_LOCK_SHARE)` once
  per page,
- `PageGetItemId` + `PageGetItem` per row inside the page,
- `HeapTupleSatisfiesVisibility(snapshot, ...)` per row to honor MVCC,
- HOT-chain follow via `heap_hot_search_buffer` for rows that have
  been HOT-updated,
- `LockBuffer(BUFFER_LOCK_UNLOCK)` + `ReleaseBuffer` once per page,
- direct varlena access to the source attribute (or `slot_getsomeattrs`
  on a freshly populated slot) for the IP kernel.

That is structurally a much larger change than the heap-TID sort
that landed in `30205`, AND the warm-cache savings are bounded
above by what `table_tuple_fetch_row_version` currently spends on
buffer pin / unpin against an already-cached page (one shared-buffer
table lookup plus an atomic pin-counter increment / decrement per
row). At `rerank_budget=800` and roughly half the rerank rows
sharing a block on a heap-TID-sorted batch, that is on the order of
`~30 us` saved per query, well inside the per-pass `0.5 ms` stddev
already seen in `30205`. The non-trivial benefit shows up only on
**cold cache** (where the saved work is not "pin/unpin" but "do not
re-issue a TOAST or heap-page read"), which is the same regime that
already gates the deferred cold-cache prefetch revisit (suggestion 3).

Recommendation: do not implement the same-page-run path until a
cold-cache harness exists and shows that heap-fetch I/O — not pin
overhead — is the dominant per-row cost. At that point, both this
suggestion and the async-overlapping prefetch from `30206` should
be evaluated together against the cold-cache numbers.

### Suggestion 3 — only revisit prefetch with a cold-cache harness or larger-than-shared-buffers corpus

Acknowledged; the suggestion itself is a deferral guidance, and
`30206` / `30207` already record the cold-cache prerequisite for
the prefetch revisit. This packet adds suggestion 2 as a second
hypothesis blocked on the same prerequisite, so when a cold-cache
harness lands the next round can evaluate three candidates at once
(prefetch, async-overlap prefetch, same-page-run grouping) instead
of one.

## Commands

```
ecaz --log-file artifacts/install-pg18-neon.log dev install ecaz-pg-test --pg 18

ecaz ... --log-file artifacts/load-neon.log corpus load \
  --prefix m5_diskann_real10k_neon_build \
  --corpus-file fixtures/m5_diskann_real10k/m5_diskann_real10k_corpus.tsv \
  --queries-file fixtures/m5_diskann_real10k/m5_diskann_real10k_queries.tsv \
  --profile ec_diskann --bits 4 --seed 42 \
  --reloption graph_degree=32 --reloption build_list_size=100 --reloption alpha=1.2

# Switch to scalar branch + reinstall scalar binary, then:
ecaz ... --log-file artifacts/load-scalar.log corpus load \
  --prefix m5_diskann_real10k_scalar_build [same args]

# Switch back to NEON tip + reinstall NEON, then:
ecaz ... --log-file artifacts/load-neon-confirm.log corpus load \
  --prefix m5_diskann_real10k_neon_build_2 [same args]

# Recall correctness check:
ecaz ... bench recall --prefix m5_diskann_real10k_{scalar,neon}_build \
  --profile ec_diskann --k 10 --sweep 64,200,800 --force-index \
  --truth-cache-file review/30204-task29-diskann-m5-neon-rerank/artifacts/truth_real10k_k10.json \
  --log-output artifacts/recall-{scalar,neon}-built-table.log
```

## Artifact list

- `manifest.md`
- `install-pg18-neon.log`, `install-pg18-scalar.log`, `install-pg18-neon-confirm.log`
- `load-neon.log`, `load-scalar.log`, `load-neon-confirm.log`
- `recall-scalar-built.log`, `recall-scalar-built-table.log`
- `recall-neon-built.log`, `recall-neon-built-table.log`
