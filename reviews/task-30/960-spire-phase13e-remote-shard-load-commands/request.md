# Review Request: SPIRE Phase 13e Remote Shard Load Commands

## Summary

This checkpoint turns the distributed placement artifact into an executable
remote materialization plan. The loader can now build corpus-only remote shard
indexes, and the distributed placement JSON includes exact `ecaz corpus load`
argument vectors for each remote node.

Code commit: `bbbc22aceb6dcf46555e43ed72db6dfa7226aed0`

## Changes

- Added `ecaz corpus load --corpus-only` for remote shard materialization
  without requiring or creating query tables.
- Added `ecaz corpus load --index-name` for single-index profiles so remote
  indexes can match the coordinator descriptor's `remote_index_regclass`.
- Extended distributed placement output with:
  - dimension, bits, seed, storage format, and reloptions
  - per-node `remote_prefix`
  - per-node `remote_load_args`
- Generated remote load args include `--profile ec_spire`, `--corpus-only`,
  `--index-name`, `--dim`, `--bits`, `--seed`, storage format, and reloptions.
- Marked the completed Phase 13e task-file items for distributed config intake
  and empty/local-only smoke-gate rejection.

## Validation

See `artifacts/manifest.md`.

- `cargo test -p ecaz-cli commands::corpus::load::tests`
- Result: 35 passed, 0 failed

## Scope Notes

This still does not connect to remote PostgreSQL instances automatically, fetch
remote endpoint identities, register descriptors on the coordinator, or publish
coordinator placement rows. It does remove the query-table blocker from remote
shard loads and gives the operator/automation an exact production CLI command
surface for each remote shard.
