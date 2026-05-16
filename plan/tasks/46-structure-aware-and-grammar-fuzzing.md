# Task 46: Structure-Aware Fuzzing and ECAZ-Grammar SQLsmith

Status: **proposed** — moves the fuzz lanes from byte-level libFuzzer over
decoders to structure-aware fuzzing of higher-level inputs and grammar-aware
fuzzing of SQL inputs that exercise ECAZ scan paths.

## Scope

Three additions to the existing fuzz stack:

1. **Structure-aware libFuzzer** via `arbitrary::Arbitrary` derive on real
   ECAZ input types, so the fuzzer generates valid-shaped inputs and
   spends its budget on edge cases inside the shape, not on rejecting
   prefix bytes.
2. **ECAZ-grammar SQLsmith** — SQL generators that bias toward
   `<-> ` / `<#>` operators, `ORDER BY embedding <-> 'query'`, partial /
   expression indexes, CustomScan plan shapes, prepared statements with
   parameterized vectors, and DDL paths (CREATE / REINDEX / VACUUM)
   interleaved with queries.
3. **Honggfuzz / AFL+** as alternate engines for high-value targets to
   cross-pollinate corpora.

## Why

The current `cargo-fuzz` targets all consume raw bytes:

```rust
fuzz_target!(|data: &[u8]| {
    let _ = ItemPointer::decode(data);
});
```

For decoders this is correct — the input *is* bytes. For higher-level
inputs (a `(dim, bits, seed, codes)` tuple as in
`fuzz_targets/parse_text.rs`, or a `VamanaMetadataPage`) the fuzzer spends
most cycles producing inputs that are rejected at the first length check.
A structure-aware target asks `arbitrary::Arbitrary` for a valid-shape
input and the fuzzer mutates within the shape — same code coverage in a
fraction of the time.

Stock SQLsmith generates legal SQL but biases toward what *PostgreSQL
core* exercises. ECAZ's CustomScan, vector operators, and SPIRE remote
paths see a tiny fraction of the schedule. A grammar that biases toward
ECAZ-specific patterns finds planner / executor bugs faster.

Honggfuzz and AFL+ explore differently (persistent vs. forkserver,
different mutators). Running them periodically against the same target
finds inputs libFuzzer misses, and vice versa.

## Approach

1. **`arbitrary` derive on input types.** Add `#[derive(Arbitrary)]` (or
   manual impl where bounds are non-trivial) on:
   - `ProdQuantizer` construction parameters with valid `(dim, bits)`
     ranges,
   - `VamanaMetadataPage` fields,
   - `SpireLeafPartitionObjectV2Meta`,
   - `ItemPointer` (trivial),
   - Top-k merge inputs (two sorted candidate lists).
   Convert existing targets to use these, keep the raw-byte targets for
   decoders.
2. **Structured fuzz targets.** Add:
   - `fuzz_topk_merge_structured` — generate two random sorted lists,
     assert merged-truncate equals sort-then-truncate.
   - `fuzz_spire_leaf_v2_roundtrip` — encode → decode → assert equal.
   - `fuzz_quant_encode_decode_roundtrip` — encode → decode → assert
     within tolerance.
3. **ECAZ-grammar SQLsmith.** Two options:
   - Patch SQLsmith with an ECAZ grammar module (upstream-feasible if the
     project accepts it).
   - Write a small Rust SQL generator under `crates/ecaz-sqlgen/` that
     produces seeded SQL and feeds it to PG18; SQLsmith remains a
     companion. Generator templates:
     - `SELECT ... ORDER BY <embedding column> <op> $1 LIMIT n` with op
       in {`<->`, `<#>`, `<=>`} and n drawn from production range,
     - `CREATE INDEX ... USING ec_diskann (col)` followed by random
       INSERT / SELECT / VACUUM,
     - prepared statement with bound vector parameters of varying dim,
     - partial / expression indexes over the vector column,
     - `REINDEX CONCURRENTLY` interleaved with queries.
4. **Honggfuzz / AFL+ targets.** Reuse the existing `fuzz_targets/` with
   Honggfuzz feature flag and an AFL+ build script. Periodic runs cross-
   pollinate corpora into `fuzz/corpus/`.
5. **Corpus management.** A `make fuzz-corpus-minimize` lane runs
   `cargo fuzz cmin` after each long campaign to keep the seed corpus
   bounded; minimized corpora are committed.
6. **Make lanes:**
   - `make fuzz-structured` — runs structured targets for `FUZZ_SECONDS`.
   - `make fuzz-honggfuzz` — Honggfuzz over the high-value targets.
   - `make fuzz-afl` — AFL+ over the high-value targets.
   - `make sqlsmith-ecaz` — runs ECAZ-grammar generator against a live
     PG18 cluster.
   - `make fuzz-cross-pollinate` — runs all engines and merges corpora.

## Validation

- Structured fuzz targets achieve ≥ 5× higher feature/edge coverage per
  second than the equivalent raw-byte targets, measured by `cargo fuzz
  coverage`.
- ECAZ-grammar SQLsmith produces no PG `PANIC` lines and no `pg_amcheck`
  failures over a 10-minute run; any failure lands in the packet.
- Honggfuzz / AFL+ find at least one input that libFuzzer's existing
  corpus did not (or document that the corpora converge — also a
  signal).
- A deliberately introduced parser bug (e.g., off-by-one in
  `VamanaMetadataPage::decode`) is caught by the structured target
  within seconds.

## Exit Criteria

- Every fuzz target that consumes a structured input uses
  `Arbitrary`-derived inputs.
- `make sqlsmith-ecaz` runs nightly with a documented seed corpus.
- Honggfuzz and AFL+ campaigns run weekly with `make fuzz-cross-
  pollinate` merging corpora.
- `fuzz/corpus/` is minimized and committed.
- `docs/hardening.md` documents the engine matrix and corpus management.

## Dependencies

- Task 34 (existing fuzz stack) is the prerequisite.
- ECAZ-grammar SQLsmith depends on the live PG18 environment from Tasks
  37–38.
- Independent of Tasks 36, 39–45.
