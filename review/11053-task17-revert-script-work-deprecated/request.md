# Review Request: Revert script-lane packets 11049 / 11051 / 11052

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `scripts/load_real_corpus.py` (restored to pre-packet-11049 state)
- `scripts/tests/test_load_real_corpus_storage_format.py` (restored)
- `scripts/vacuum_concurrency_scratch.sh` (restored)
- `scripts/bench_sql_latency.sh` (restored)
- `scripts/bench_tqvector_sql_overhead_breakdown.sh` (restored)
- `scripts/tests/test_bench_tqvector_sql_overhead_breakdown.py` (restored)
- `review/11049-task17-loader-am-generic/` (removed)
- `review/11051-task17-scripts-ecvector-rename/` (removed)
- `review/11052-task17-overhead-breakdown-ecvector-rename/` (removed)

## What this packet is

A three-commit revert undoing the script-lane work from packets 11049,
11051, and 11052. Those packets were written against a wrong mental model
of the `scripts/` directory's status on this branch.

The actual status: the `ecaz-cli` crate that landed on `main` alongside
the rename is **fully implemented**, not a skeleton. `ecaz corpus load`,
`ecaz bench latency`, `ecaz bench overhead`, `ecaz stress vacuum`, and
the rest all have real implementations that consume the
`IndexProfile` registry (both `ec_hnsw` and `ec_diskann`). Its README
still describes a "v1 skeleton / v2 port PRs" state, which is what
misled me, but the source tells the truth.

The scripts touched by 11049/11051/11052 are therefore pure **deletion
targets**: the ecaz-cli verbs already replace them. The correct response
on this branch is to stay out of those files entirely and let the
other agent's deletion PR land on `main`.

## Why the revert (not a forward fix)

Three reasons, in order of importance:

1. **Unblocks the other agent.** The native-build / ecaz-cli lane owns
   `scripts/*` on `main`. My 11051/11052 renames and 11049 refactor
   inflated the file diff between `main` and this branch in exactly
   those files, which is the set the other agent is about to delete.
   Rebasing after that deletion lands would produce avoidable
   conflicts in files I should never have touched. Revert collapses
   those to zero.

2. **Honors the "deprecate means delete" memory.** The scripts are
   being deprecated, and per the newly-recorded feedback memory
   (`feedback_deprecate_means_delete.md`), "deprecate X" in this
   project means `git rm X`, not rename-in-place. My prior slices
   treated them as migration targets, which is the exact anti-pattern
   that memory now guards against.

3. **No lost work.** Every behavior the three packets added already
   exists in `ecaz-cli`:
   - 11049's AM-generic loader refactor → `ecaz corpus load --profile
     ec_diskann`, including `--reloption` passthrough and
     `--m`-rejection for non-HNSW profiles, in
     `crates/ecaz-cli/src/commands/corpus/load.rs`.
   - 11051/11052's `tqvector → ecvector` rename → the scripts will be
     deleted on `main`; there is no post-delete world where the
     rename matters.

## What changed (mechanically)

Three `git revert` commits in reverse chronological order, each
squashed into its original packet's footprint:

- Revert `d838f77` (packet 11052) — restores
  `scripts/bench_tqvector_sql_overhead_breakdown.sh` and its test to
  pre-rename state; removes `review/11052-...`.
- Revert `0135e1f` (packet 11051) — restores
  `scripts/vacuum_concurrency_scratch.sh` and
  `scripts/bench_sql_latency.sh`; removes `review/11051-...`.
- Revert `2cc943f` (packet 11049) — restores `scripts/load_real_corpus.py`
  and `scripts/tests/test_load_real_corpus_storage_format.py` to the
  pre-AM-generic-refactor state; removes `review/11049-...`.

The revert keeps `docs/RECALL_REAL_CORPUS.md` from packet 11050 intact.
That packet is pure docs and already points operators at
`ecaz bench recall / latency`, so it remains correct after the scripts
are deleted on `main` — only the Schema section's `tqvector → ecvector`
text is a transitional artifact that will reconcile naturally when
`main`'s post-delete shape replaces those paragraphs.

## Test evidence

Revert is mechanical. The three reverted commits were the only changes
under `scripts/` on this branch; post-revert `git diff origin/main --
scripts/` should be empty. No code paths newly relied on any of those
script edits — the DiskANN access method itself lives in
`src/am/ec_diskann/` and has no dependency on the Python loader.

## Follow-ups / forward plan

- The other agent's PR against `main` deletes `scripts/*` whose ecaz-cli
  replacement exists. That is the right place; this branch stays out
  of the way.
- Any future task-17 operator-lane work goes into `crates/ecaz-cli/`
  directly (new `--profile ec_diskann` defaults, new commands, DiskANN-
  specific CLI flags), never into `scripts/`.
- The memory note `feedback_deprecate_means_delete.md` now guards against
  this category of mistake.
