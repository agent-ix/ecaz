# Task 31 Corpus Prepare Profiles

Reviewer: please review this narrow CLI checkpoint before the Task 31 real
corpus staging packet relies on the new profiles.

## Scope

Task 31 needs real DBPedia 10k, 25k, and 100k one-index-per-table surfaces.
During staging, `ecaz corpus prepare` exposed only canonical 10k, 50k, and 990k
subset recipes. This checkpoint adds first-class 25k and 100k recipes instead
of deriving benchmark fixtures with ad hoc file slicing.

Code checkpoint commit: `bfee0e29`

## Change

Updated `crates/ecaz-cli/src/commands/corpus/prepare.rs`:

- added `ec_hnsw_real_25k` with `25000` corpus rows and `500` query rows
- added `ec_hnsw_real_100k` with `100000` corpus rows and `1000` query rows

The names intentionally keep the existing historical `ec_hnsw_real_*` corpus
profile naming convention. They are corpus subset recipes, not access-method
claims; the later `corpus load` step will use `--profile ec_ivf`.

## Validation

```sh
cargo test -p ecaz-cli corpus::prepare
```

Result: passed, `24` prepare tests.

`git diff --check -- crates/ecaz-cli/src/commands/corpus/prepare.rs` passed.

## Follow-Up

Reinstall the local operator binary with:

```sh
cargo install --path crates/ecaz-cli
```

Then continue `30167-task31-m5-real-corpus-staging` using
`/Users/peter/.cargo/bin/ecaz corpus prepare --profile ec_hnsw_real_25k` and
`--profile ec_hnsw_real_100k`.
