# Review Request: Task 17 Real-Corpus Loader AM-Generic Refactor

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `scripts/load_real_corpus.py`
- `scripts/tests/test_load_real_corpus_storage_format.py`

## What this packet is

This packet finishes the half-done refactor of `scripts/load_real_corpus.py`
that was in flight on this branch when we rebased onto the post-ecaz /
post-PG18 `origin/main`. Before this packet, the script would not even parse
its own argument surface into the rest of the body: the `--index-profile`
flag had been announced in the docstring and the `INDEX_PROFILES` registry
was added, but several downstream helpers still called the pre-refactor
signatures and the `tqvector` / `encode_to_tqvector` world was still hardwired
in `_ensure_corpus_table` and `_load_corpus`.

After this packet, the script:

- Is AM-generic through an `IndexProfile` record that captures the
  `USING` access method, operator class, embedding column type, encoder
  function, and per-AM capability flags.
- Ships `ec_hnsw` and `ec_diskann` profiles wired to the current `ecvector` /
  `encode_to_ecvector` world and the real operator-class names
  (`ecvector_ip_ops`, `ecvector_diskann_ip_ops`) that `sql/bootstrap.sql`
  actually registers.
- Accepts pass-through `--reloption key=value` for AM-specific tunables
  (`graph_degree`, `alpha`, `list_size`, …) that the loader has no business
  knowing about on a per-AM basis.
- Keeps the HNSW m-sweep semantics intact for backward compatibility and
  guards against `--m` being passed against a non-HNSW profile.
- Rewires the corpus/index helpers to thread the profile through:
  `embedding <type>` column, `encode_to_*` encoder, `USING <am> (embedding
  <opclass>)`, reloption list.

## Why this slice

The user's request on this branch is explicit: DiskANN work needs to share
the real-corpus loading lane with HNSW rather than fork a parallel script.
Finishing the in-flight refactor cleanly — including tests that exercise the
new profile machinery — is a prerequisite for running DiskANN recall/latency
evaluation against the same staged corpora HNSW already uses.

This packet is deliberately scoped to the loader. Other `scripts/` files
still reference `tqvector` / `encode_to_tqvector` (e.g. `bench_sql_latency.sh`,
`vacuum_concurrency_scratch.sh`); those are follow-up packets, not part of
this slice.

## What changed

### `scripts/load_real_corpus.py`

- Added the `IndexProfile` dataclass (`name`, `access_method`, `operator_class`,
  `embedding_type`, `encoder_function`, `supports_build_source_column`,
  `supports_m_sweep`) and the `INDEX_PROFILES` registry with `ec_hnsw` and
  `ec_diskann` entries.
- Added `--index-profile`, `--reloption key=value` (repeatable), and
  `_validate_index_profile` / `_validate_reloption` validators.
- Replaced the hardcoded `tqvector` / `encode_to_tqvector` usage in
  `_ensure_corpus_table` and `_load_corpus` with profile-driven
  `embedding_type` and `encoder_function`.
- Rewrote `_ensure_index` and `_index_exists_with_options` to take a
  reloption list plus the resolved profile, rather than the old
  `(m, ef_construction, storage_format)` triple.
- Rewrote `_build_index_sql` to take `(access_method, operator_class,
  reloptions)` and emit a `WITH (...)` clause only when reloptions are
  present (DiskANN runs today do not need a reloption list to succeed).
- Added `_format_reloption_sql_value` so numeric/boolean reloptions stay
  unquoted and string reloptions are single-quoted with `''`-escaping.
- Added `_build_hnsw_reloption_sweep` / `_dedupe_int_sweep` helpers so the
  HNSW m-sweep path stays self-contained and the DiskANN path is a single
  `<prefix>_idx` job.
- `main()` now:
  - Resolves the profile up front.
  - Rejects `--m` for profiles without `supports_m_sweep` with a clear
    error rather than silently building one index per m value.
  - Threads the profile into all table / index / loader helpers.
  - Logs a post-run summary that names the profile used.

### `scripts/tests/test_load_real_corpus_storage_format.py`

- Renamed the HNSW-only helpers in the tests to go through the new
  `_format_hnsw_reloptions` + `_expected_index_reloptions(list)` surface.
- Added coverage for:
  - DiskANN `_build_index_sql` using the `ec_diskann` opclass and emitting
    no `WITH (...)` clause when reloptions are empty.
  - Reloption SQL quoting (numerics unquoted, strings quoted).
  - `_validate_index_profile` / `_validate_reloption` rejection paths.
  - Profile metadata consistency with the access-method / opclass names
    registered in `sql/bootstrap.sql`.

## Test evidence

```
$ python3 scripts/tests/test_load_real_corpus_storage_format.py
..........
----------------------------------------------------------------------
Ran 10 tests in 0.001s

OK
```

`python3 scripts/load_real_corpus.py --help` renders both the HNSW
backward-compat form and the new `--index-profile ec_diskann` example from
the module docstring.

## Follow-ups intentionally not in this packet

- `scripts/bench_sql_latency.sh`, `scripts/vacuum_concurrency_scratch.sh`,
  and `scripts/bench_tqvector_sql_overhead_breakdown.sh` still carry pre-rename
  `tqvector` / `encode_to_tqvector` references. They are out of scope here and
  will be addressed in a later packet once the DiskANN recall lane is wired up
  and we know which of those benchmarks need to stay HNSW-only vs. become
  AM-generic.
- `docs/RECALL_REAL_CORPUS.md` may want a short note that `--index-profile`
  exists; left for the DiskANN recall-doc packet so the two land together.
