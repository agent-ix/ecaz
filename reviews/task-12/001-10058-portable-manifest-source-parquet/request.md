# Review Request: Portable `source_parquet` Field in Real-Corpus Manifest

Scope:
- `scripts/qdrant_dbpedia_to_tsv.py` — `_write_manifest`, `main`, `--source-dataset` help text
- `scripts/load_real_corpus.py` — `_verify_manifest`
- `scripts/tests/test_manifest_portability.py` — new fixture tests (first file in `scripts/tests/`)
- `scripts/tests/run.sh` — minimal test runner wrapper (first file in `scripts/tests/`)
- `docs/RECALL_REAL_CORPUS.md` — manifest-field section

Task: `plan/tasks/coder2/10058-portable-manifest-source-parquet.md`
Motivation trail: review 222 feedback observation N4.

## Problem

`scripts/qdrant_dbpedia_to_tsv.py` previously recorded the parquet input
path via `os.path.abspath(args.parquet)` at line 410. The first official
DBpedia manifest is about to be committed alongside the first recorded
benchmark result, and the committed manifest would therefore carry a
per-developer absolute path like `/home/peter/dev/datasets/...` that does
not exist on any other reviewer's machine. That is cosmetic right now but
would be noisy the moment a second reviewer tries to verify the first
run. Task 10058 asks for a fix before the first manifest gets committed
and the migration cost goes up.

## Change summary

### Step 1 — add portable identity fields to the writer

`scripts/qdrant_dbpedia_to_tsv.py:_write_manifest` now records two
additional JSON fields alongside the existing `source_parquet` absolute
path:

- `source_parquet_basename` — the basename only. Computed with
  `Path(source_parquet).name`, which handles a trailing slash on
  directory inputs correctly (`Path("/a/b/").name == "b"`), unlike
  `os.path.basename`.
- `source_parquet_shard_basenames` — sorted list of per-shard parquet
  file basenames that were actually iterated (basenames of the
  `_resolve_parquet_files` output). This is the portable equivalent of
  "which shards did this run read".

`source_parquet` is deliberately kept as the current absolute path — a
local-debug hint only — because reviewers running locally still find it
useful. The `--source-dataset` help text now states explicitly that it is
a "human-readable dataset label (not a path)".

The new fields are additive. `manifest_version` is intentionally NOT
bumped: older verifiers ignore unknown JSON keys gracefully and bumping
the version would force a loader-and-manifest lock-step upgrade for no
gain. This matches the task's "Design notes" section.

### Step 2 — teach the loader to verify portable fields

`scripts/load_real_corpus.py:_verify_manifest` now:

- accepts manifests with or without the new fields (older manifests must
  still load — backwards compatible)
- when the new fields are present, checks that:
  - `source_parquet_basename` is a string and does not contain `/` or `\`
  - `source_parquet_shard_basenames` is a list of strings and no entry
    contains `/` or `\`
- **deliberately does not** try to verify that `source_parquet` exists on
  disk — that is the whole point of stripping it from the check surface

All other existing checks (row count, SHA-256, first/last id, basename,
dimension, prefix, manifest_version) are unchanged.

### Step 3 — fixture test

New file `scripts/tests/test_manifest_portability.py` uses
`unittest.TestCase` and stages tiny synthetic corpus/query TSVs plus a
synthetic manifest under a temp directory. Eight tests cover:

1. `test_manifest_with_portable_fields_is_accepted` — happy-path new
   manifest; the absolute `source_parquet` points at a non-existent path
   to prove the verifier does NOT touch it.
2. `test_manifest_without_portable_fields_is_accepted` — backwards-
   compatible path; older manifests must keep loading.
3. `test_manifest_with_absolute_basename_is_rejected` — portable-ness
   check catches an absolute path in the basename field with a clear
   error.
4. `test_manifest_with_absolute_shard_is_rejected` — same, but for one
   shard entry.
5. `test_manifest_with_non_string_basename_is_rejected` — type check.
6. `test_manifest_with_non_list_shard_basenames_is_rejected` — type
   check.
7. `test_writer_emits_portable_fields_for_file_input` — writer-side
   round-trip. Stubs `parquet_files` directly, verifies that the
   emitted basename strips the directory part and that shard basenames
   are sorted (the writer sorts internally, so passing
   `[shard-00001, shard-00000]` must round-trip to sorted output).
8. `test_writer_basename_handles_trailing_slash` — regression guard
   around the `Path(...).name` choice.

The repo had no Python test harness yet, so per the task's instructions
the file lives at `scripts/tests/test_manifest_portability.py` with a
minimal `main()` runner and a `scripts/tests/run.sh` wrapper. When a real
Python test harness lands, the wrapper can be replaced.

### Step 4 — docs

`docs/RECALL_REAL_CORPUS.md` manifest section now expands the
"dataset/source metadata" bullet into one line per field:

- `source_dataset` — human-readable label, not a path
- `source_parquet` — local absolute path, debugging hint only, not
  verified by the loader
- `source_parquet_basename` — portable basename, verified
- `source_parquet_shard_basenames` — sorted per-shard basename list,
  verified

It also states the "present-but-non-portable values are rejected,
absent values are accepted" contract so a future reviewer does not
accidentally introduce an absolute path into the portable fields.

### Step 5 — scratch smoke manifest

`find . -name "*manifest.json"` (excluding `target/` and `.git/`) returns
nothing in the tree, and no shell script references a checked-in
manifest, so there is nothing to regenerate. The task anticipated this:
review 221 used a `/tmp` synthetic manifest and did not commit it.

## Validation

From the task's "Validate" section:

```
python3 -m py_compile scripts/qdrant_dbpedia_to_tsv.py scripts/load_real_corpus.py scripts/tests/test_manifest_portability.py
# → compile ok

python3 scripts/qdrant_dbpedia_to_tsv.py --help
# → --source-dataset help now reads
#   "Human-readable dataset label stored in the manifest (not a path)."

./scripts/tests/run.sh
# → Ran 8 tests in 0.008s — OK
```

All eight tests pass. The real DBpedia manifest is intentionally **not**
committed in this task — that is coder-1's job when the first gate number
lands.

## What reviewers should check

1. Is the `Path(...).name` choice for `source_parquet_basename` the right
   one? It handles trailing slash on a directory input; `os.path.basename`
   does not.
2. Is rejecting a present-but-non-portable value strict enough? The task
   asks for the loader to verify portable fields are "strings /
   list-of-strings respectively (not absolute paths with `/`)". I also
   reject backslashes because a Windows-authored manifest would be just
   as non-portable, and I reject a list of non-strings (not just a list).
3. Is it acceptable to write the fixture tests as `unittest.TestCase`
   rather than adding a `pytest` dependency? The repo currently has no
   Python test harness, so this is "whatever is simpler to run without
   installing anything".
4. Is the `run.sh` wrapper the right shape for now, or should the tests
   be wired into the existing `Makefile`? I left it as a standalone
   wrapper because the Makefile does not currently have a Python target
   and adding one felt out of scope.

## Out of scope

- Any change to the canonical selection rule, TSV format, or SHA-256
  hashing of corpus/queries files.
- Any change to `manifest_version`.
- Committing a real DBpedia manifest.
- Wider cleanup of the real-corpus scripts.
