# Review Request: Task 29 DiskANN rebase verification

Branch: `task29-diskann-initial-tuning`
Author: coder-1
Target:

- `src/am/common/training.rs`
- `origin/adr034-diskann-rebased` rebased onto current `main`

## What This Packet Is

This is the first Task 29 integration checkpoint after recovering the fresher
DiskANN branch onto current `main`.

The goal was not to benchmark or optimize yet. The goal was to make the
rebased DiskANN branch compile against the landed Task 28 IVF/shared-surface
changes so the tuning lane has a trustworthy starting point.

## What Changed

### Branch Recovery

Created local branch `task29-diskann-initial-tuning` from
`origin/adr034-diskann-rebased` and rebased it onto current `main`.

Resolved integration conflicts by preserving:

- landed Task 28 IVF SQL/bootstrap and `ecaz-cli` behavior
- the full `src/am/ec_diskann/` implementation and task-17 review packets
- current `ecaz-cli` real-corpus, truth-cache, and explicit connection-flag
  surfaces

Obsolete deleted script-lane commits were not resurrected; the branch keeps the
current `ecaz-cli`-owned workflow from `main`.

### Compile Drift Fix

DiskANN expected two binary-sidecar helpers through
`am::common::training`, while current `main` exposed the underlying behavior
through `quant::rabitq` and kept the old HNSW wrappers private.

Added thin shared wrappers in `src/am/common/training.rs`:

- `persisted_binary_sidecar_word_count`
- `derive_persisted_binary_words`

This keeps DiskANN from depending on HNSW-private build helpers and matches the
shared AM boundary introduced by the multi-AM work.

## Validation

Ran:

```sh
git diff --check
cargo check --all-targets --no-default-features --features pg18
```

Result:

- `git diff --check` passed
- PG18 compile check passed after the shared training-wrapper fix

## Follow-Ups

- Run focused PG18 DiskANN callback tests next.
- Inspect and refresh any DiskANN-specific `ecaz-cli` bench/corpus workflows
  before taking new baseline measurements.
- Start fresh local real-corpus baseline packets only after the build/test
  surface is clean.
