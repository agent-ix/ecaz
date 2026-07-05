# Review Request: Real Corpus Prefix Rename

**Requester:** coder1
**Date:** 2026-05-16
**Head SHA:** `29edf9b3528776c1d8912a403ba2b34af968dc5b`
**Review focus:** hard-break rename from misleading `ec_hnsw_real_*` corpus
profile/artifact names to access-method-neutral `ec_real_*` names.

## Purpose

The real-corpus subset profiles were named `ec_hnsw_real_*`, and that prefix
was reused in staged TSVs, manifests, example table names, and suite inputs.
That incorrectly suggested HNSW was the access method even when the load or
benchmark profile was `ec_ivf`, `ec_diskann`, or `ec_spire`.

This slice makes the corpus profile/artifact namespace access-method-neutral.

## Changes

- `ecaz corpus prepare` now exposes canonical subset profiles:
  `ec_real_10k`, `ec_real_25k`, `ec_real_50k`, `ec_real_100k`, and
  `ec_real_ann_benchmarks_anchor`.
- Legacy `ec_hnsw_real_*` profile names are not accepted as aliases.
- Prepared TSV and manifest basenames now follow the `ec_real_*` prefix, and
  generated manifests record the same neutral prefix.
- Active benchmark suites, scripts, integration tests, and docs now reference
  the new prepared artifact names.
- The AWS representative load script now separates the prepared artifact prefix
  (`ec_real_100k`) from the database load prefix (`ec_spire_aws_repr_1m`) and
  passes the manifest explicitly with `--allow-manifest-mismatch`.
- Historical `review/` packets and raw logs were intentionally left unchanged.

## Validation

- `cargo test -p ecaz-cli commands::corpus::prepare::tests`
- `cargo test -p ecaz-cli parses_fetch_prepare_suite_config`
- `jq empty crates/ecaz-cli/suites/*.json scripts/spire-aws/*.json`
- `git diff --check`
- `rg -n "ec_hnsw_real" crates scripts tests docs -g '!target'`
  - remaining hits are only the negative alias test in
    `crates/ecaz-cli/src/commands/corpus/prepare.rs`

## Reviewer Notes

This is a breaking CLI/data-path rename. Existing local staged corpora under
old filenames need to be regenerated or renamed by operators before running
updated suite configs.
