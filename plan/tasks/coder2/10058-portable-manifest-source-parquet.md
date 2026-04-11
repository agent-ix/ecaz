# Task: Portable `source_parquet` Field in Real-Corpus Manifest

Motivation: Review 222 feedback observation N4 flagged that
`scripts/qdrant_dbpedia_to_tsv.py` records the parquet input path via
`os.path.abspath(args.parquet)` at line 410. The first official DBpedia
manifest is about to be committed alongside the first recorded benchmark
result, and the committed manifest will therefore carry a per-developer
absolute path like `/home/peter/dev/datasets/...` that does not exist on any
other reviewer's machine. That is cosmetic for now but will be noisy the
moment a second reviewer tries to verify the first run. Fix it before the
first manifest gets committed and the migration cost goes up.
Priority: batch 3
Status: in review
Branch: task/10058-portable-manifest-source-parquet

## Prompt

Make the real-corpus manifest portable across reviewers by recording
dataset identity separately from the local fetch path.

### Step 1 — add a portable dataset identity field

In `scripts/qdrant_dbpedia_to_tsv.py`, update `_write_manifest` (around
`qdrant_dbpedia_to_tsv.py:288`) to record, in addition to the current
`source_parquet` path:

- `source_parquet_basename`: the basename only (directory or single-file
  basename — strip the absolute path). For a directory input, use the
  directory name.
- `source_parquet_shard_basenames`: a sorted list of the per-shard
  parquet file basenames that were actually iterated (i.e. the
  basenames of the `_resolve_parquet_files` output). This is the
  portable equivalent of "which shards did this run read".

Keep `source_parquet` as the current absolute path. Do not remove it —
reviewers running locally still find it useful for debugging. The contract
change is "the portable fields are the ones you verify against; the
absolute path is a local-debug hint only".

Update the existing `source_dataset` field's docstring / help text to
make clear it is still the human-readable dataset label (currently
defaults to
`"Qdrant dbpedia-entities-openai3-text-embedding-3-large-1536-1M"`), not
a path. No rename needed.

### Step 2 — update the loader's manifest verifier

In `scripts/load_real_corpus.py`, `_verify_manifest` (around line 239)
should:

- accept manifests **with or without** the new fields (older manifests
  must still load)
- if the new fields are present, verify that
  `source_parquet_basename` and `source_parquet_shard_basenames` are
  strings / list-of-strings respectively (not absolute paths with `/`)
- deliberately **not** verify that `source_parquet` exists on disk
  (that is the whole point of stripping it from the check surface)

Do not break any existing verification — row count, SHA-256, first/last
id, basename, dimension, prefix, manifest_version all stay exactly as
they are.

### Step 3 — add a fixture test

The existing smoke in review 221 created a tiny synthetic manifest in
`/tmp` and exercised the loader's verification. Add a `tests/fixtures/`
asset or an inline string fixture that covers:

- a manifest WITH the new fields (loader accepts and verifies)
- a manifest WITHOUT the new fields (loader still accepts — backwards
  compatible)
- a manifest with the new fields pointing at absolute paths (loader
  rejects with a clear error, because the portable fields must be
  portable)

Python `unittest` or `pytest` is fine — match whatever test style the repo
already uses for Python scripts. If there is no Python test harness yet,
put the fixture under `scripts/tests/test_manifest_portability.py` with
a minimal `if __name__ == "__main__": main()` runner and a
`./scripts/tests/run.sh` wrapper.

### Step 4 — document the portable-identity contract

In `docs/RECALL_REAL_CORPUS.md`, the manifest section (around line 94,
`### Manifest File: <basename>_manifest.json`) currently lists
`dataset/source metadata`. Expand that bullet to be explicit:

- `source_dataset`: human-readable label
- `source_parquet`: local absolute path used at conversion time, for
  debugging only
- `source_parquet_basename`: portable basename, verified by the loader
- `source_parquet_shard_basenames`: sorted list of per-shard basenames,
  verified by the loader

One line per field is enough. The doc's whole job here is to make the
committed-first-run manifest reviewable on a different machine.

### Step 5 — regenerate the scratch smoke manifest

If the repo checks in any sample/smoke manifest (review 221 used a
`/tmp` synthetic one and did not commit it, but double-check the
current tree), regenerate it under the new script to pick up the new
fields. Do not commit the real DBpedia manifest from this task — that
is coder-1's job when the first gate number lands.

## Design notes

- Backwards compatibility for the loader is required. If the loader
  rejects old manifests, existing reviewers lose access to already-staged
  fixtures.
- The new fields are additive in the JSON. Order keys alphabetically the
  same way the current manifest does (`json.dump(..., sort_keys=True)`
  already handles this).
- Do not change the `manifest_version` number. This is a purely additive
  field change that older verifiers ignore gracefully. Bumping the
  version would force a loader-and-manifest lock-step upgrade for no
  gain.
- Do not try to make `source_parquet` relative. Relative to what? A
  basename is the only truly portable form here; if someone needs the
  full path they can reconstruct it from the basename + their own
  dataset root.

## Out of scope

- Any change to the canonical selection rule, TSV format, or SHA-256
  hashing of corpus/queries files.
- Any change to `manifest_version`.
- Committing a real DBpedia manifest.
- Wider cleanup of the real-corpus scripts. This task is exclusively
  about portability of `source_parquet`.

## Validate

```bash
python3 -m py_compile scripts/qdrant_dbpedia_to_tsv.py scripts/load_real_corpus.py
python3 scripts/qdrant_dbpedia_to_tsv.py --help
```

Manifest round-trip smoke (no real parquet needed — reuse the synthetic
manifest fixture the loader tests exercise):

```bash
./scripts/tests/run.sh  # or however the new fixture is wired
```

Branch from current upstream main. Push branch for review.
