## Feedback: PqFastScan Empty Bootstrap And Small Builds

Read `seed_group_codebook_from_small_samples` at `src/am/build.rs:907`,
`default_pq_fastscan_flush_output` at `:1060`,
`bootstrap_empty_pq_fastscan_flush_output` at `src/am/insert.rs:1524`,
and the empty-index branch in `run_insert_with_adapter` at `:456-486`.

### What's right

- **Closes two real lifecycle gaps in one packet.** Small-cardinality
  builds (<16 rows) and empty-index first-insert were both hard
  rejects; both now have real success paths. Task 15 can't claim
  "first-class format" with either of these still failing.
- **Empty-insert bootstrap reuses the build payload path.**
  `build::default_pq_fastscan_flush_output(...)` +
  `build::write_data_pages(...)` means the first-insert grouped
  layout is byte-for-byte identical to the build-time grouped
  layout. That sidesteps the "insert and build drift" class of bug
  entirely.
- **Bootstrap runs under `with_locked_metadata_page`.** That
  serializes concurrent bootstrappers by the same mechanism the
  scalar first-insert path uses. Race loser rechecks metadata and
  falls through to the normal grouped insert path, which by then
  has persisted codebooks. Correct.
- **Small-sample fallback is deterministic.**
  `seed_group_codebook_from_small_samples` wraps samples across 16
  centroid slots using `(seed + centroid_index) % sample_count`.
  Reindexing the same small table gives the same codebook. Important
  for the reindex-is-deterministic contract from packet 361.
- **The codebook-specific error message for metadata-without-
  codebooks is narrower and correct.** "tqhnsw PqFastScan metadata
  is missing persisted grouped codebooks" is the right framing —
  empty index is no longer treated as "unsupported," only as "not
  yet bootstrapped."

### Concerns

1. **Codebook quality at very small N is essentially random.**
   Repeating a handful of samples across 16 centroid slots produces
   a usable shape but not a useful quantization. The index can
   *run*, but recall at very small N will be poor and grouped
   scoring will be close to uniform noise. That's a fine trade for
   "small tables don't fail to build," but a user running a 10-row
   test corpus may see surprising recall. One line in the README /
   test-output comment about "below ~16 rows the grouped codebook
   is degenerate — latency-sensitive tuning requires a larger
   training set" would set expectations.

2. **Bootstrap holds the metadata exclusive lock while writing data
   pages.** Preexisting pattern in the scalar first-insert path, so
   consistent. Worth confirming explicitly: nothing inside
   `build::write_data_pages` acquires any lock that could need a
   metadata lock above it (lock-order inversion risk). Would flag as
   a concrete verification item rather than a blocker.

3. **Race-loser path does full re-entry into `run_insert_with_adapter`.**
   The recursive call at `insert.rs:474-486` is correct but worth a
   comment — a reader encountering it cold wonders if this can
   loop. It can't (once metadata is populated, the recursion hits
   the fast path), but the invariant is load-bearing.

4. **No concurrency test.** The bootstrap race is the interesting
   case — two concurrent inserts on an empty grouped index must
   produce exactly one set of codebooks and both rows must appear.
   No test verifies that. Structural tests prove the single-inserter
   path; the race needs explicit coverage before merge.

5. **Linker gap.** The new bootstrap test and the 4-row build test
   are both load-bearing. Neither ran locally.

### Observation

This is the packet that makes "first-class PqFastScan" honest. Before
this, `WITH (storage_format='pq_fastscan')` on a small or empty table
worked up to the point of first insert and then failed — which is
exactly the sort of partial implementation that erodes trust.
