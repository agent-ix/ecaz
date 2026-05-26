# Task 46/003: fuzz corpus minimize + commit (closes §Exit Criteria #4)

## Scope

Closes one Task 46 §Exit Criteria gate in full:

> 4. `fuzz/corpus/` is minimized and committed.

Tracks Task 46 §Approach 5:

> A `make fuzz-corpus-minimize` lane runs `cargo fuzz cmin` after
> each long campaign to keep the seed corpus bounded; minimized
> corpora are committed.

Two-commit slice — policy/tooling change separate from the corpus
content add so reviewer can read each diff cleanly:

1. `56783b5eb` — `.gitignore` removes the blanket `/fuzz/corpus`
   ignore (keeping `/fuzz/target` and `/fuzz/artifacts` ignored);
   `Makefile` adds `make fuzz-corpus-minimize` wrapping
   `cargo fuzz cmin` across every registered target.
2. `5d84cedc9` — initial commit of minimized corpora for the 9
   targets currently registered in `fuzz/Cargo.toml`.

Validation head: `5d84cedc9`.

## What changed

| Path | Commit | Purpose |
|---|---|---|
| `.gitignore` | `56783b5eb` | un-ignores `fuzz/corpus/` |
| `Makefile` | `56783b5eb` | adds `make fuzz-corpus-minimize` |
| `fuzz/corpus/**` (396 files) | `5d84cedc9` | minimized corpora |

No production code change. Tooling + committed test artifacts only.

## Evidence

### Corpus minimization numbers

Per `cargo fuzz cmin` output (the cmin loop preserves coverage by
keeping only inputs whose union spans the original feature/edge
set):

| target | pre files | pre size | post files | post size | reduction |
|---|---:|---:|---:|---:|---|
| `fuzz_parse_text` | 520 | 2.0 MB | **143** | **572 KB** | 3.6× / 3.6× |
| `fuzz_parse_text_structured` | 78 | 312 KB | 64 | 256 KB | 1.2× / 1.2× |
| `fuzz_unpack_mse` | 35 | 140 KB | 24 | 96 KB | 1.5× / 1.5× |
| `fuzz_unpack_mse_structured` | 125 | 500 KB | 85 | 340 KB | 1.5× / 1.5× |
| `fuzz_element_tuple_decode` | 32 | 128 KB | 31 | 124 KB | 1.0× / 1.0× |
| `fuzz_neighbor_tuple_decode` | 30 | 120 KB | 29 | 116 KB | 1.0× / 1.0× |
| `fuzz_diskann_metadata_decode` | 3 | 12 KB | 3 | 12 KB | unchanged |
| `fuzz_item_pointer_decode` | 2 | 8 KB | 2 | 8 KB | unchanged |
| `fuzz_vector_normalize` | 19 | 76 KB | 15 | 60 KB | 1.3× / 1.3× |
| **total** | **884** | **3.3 MB** | **396** | **1.6 MB** | **2.2× / 2.1×** |

Per-target cmin coverage lines (preserved coverage edges/features):

- `fuzz_parse_text`:           138 cov, 378 ft (143 spanning files)
- `fuzz_parse_text_structured`: 253 cov, 635 ft
- `fuzz_unpack_mse`:             51 cov, 102 ft
- `fuzz_unpack_mse_structured`: 213 cov, 681 ft
- `fuzz_element_tuple_decode`:  151 cov, 264 ft
- `fuzz_neighbor_tuple_decode`:  89 cov, 248 ft
- `fuzz_diskann_metadata_decode`: 80 cov,  80 ft
- `fuzz_item_pointer_decode`:    43 cov,  43 ft
- `fuzz_vector_normalize`:      357 cov, 455 ft

The structured targets I added in packets 001/002 retain their
2× / 1.7× coverage advantage over the raw siblings after cmin
(unpack_mse 51 → 213 cov; parse_text 138 → 253 cov).

### Artifact files

- `artifacts/cmin-parse-text.log` — standalone cmin run output for
  the largest target (`fuzz_parse_text`, the biggest reduction).
- `artifacts/cmin-batch.log` — sequential cmin batch over the
  remaining 8 targets.
- `artifacts/corpus-sizes-post-cmin.log` — file count / disk size
  per target after cmin, with the pre-cmin baseline in comments
  for diff.
- `artifacts/manifest.md` — packet metadata + key result lines.

## Reviewer focus

- Two commits, split exactly along the
  "policy change vs content add" line so the diff stays readable.
  `git show 56783b5eb` is 2 files / 16 lines; `git show 5d84cedc9
  --stat` is 396 binary corpus files / 372 LOC insertions (most
  files are 1-2 bytes; only `fuzz_parse_text` and the structured
  targets have non-trivial entries).
- The Makefile target uses `cargo +nightly fuzz cmin` per target;
  the same toolchain the existing `fuzz-*` targets use. No
  re-implementation of cmin logic — straight wrapper.
- Coverage preservation is a property of `cargo fuzz cmin` itself
  (it produces a spanning set over the original coverage). The
  per-target coverage lines above are what cmin reported as it
  finished; if any of those numbers regress in a future re-cmin,
  that signals real corpus drift, not a methodology change.

## Task 46 §Exit Criteria progress after this slice

| # | Criterion | Status |
|---|---|---|
| 1 | Every structured-input fuzz target uses Arbitrary | partial (2 of N) |
| 2 | `make sqlsmith-ecaz` nightly with seed corpus | 0% |
| 3 | Honggfuzz + AFL+ weekly with `make fuzz-cross-pollinate` | 0% |
| 4 | `fuzz/corpus/` minimized + committed | **✓ DONE** |
| 5 | `docs/hardening.md` engine-matrix section | 0% |

One of five gates closes here. Task 46 progress ≈ 25% complete.

## Out of scope (still open)

Per Task 46 §Approach / §Exit Criteria, still open after this slice:

1. SQLsmith ECAZ-grammar (§Approach 3 / §Exit #2)
2. Honggfuzz + AFL+ integration (§Approach 4 / §Exit #3)
3. `docs/hardening.md` engine-matrix section (§Exit #5)
4. More structured targets: `fuzz_topk_merge_structured`,
   `fuzz_spire_leaf_v2_roundtrip`, `fuzz_quant_encode_decode_roundtrip`
5. Deliberate-parser-bug regression evidence (Task 46 §Validation
   last bullet)
6. Structured error-path target paired with `parse_text_structured`
   (the open item the reviewer flagged on 002 about per-target vs
   aggregate §Validation reading — operator clarified "i dont care
   which choice" so 002 stands; this would be the per-target
   compliance path if anyone later opts for it)
