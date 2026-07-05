# Review Request: Reject `--reloption` keys that collide with native CLI flags

Branch: `adr034-diskann-rebased`
Author: coder-2
Target:

- `crates/ecaz-cli/src/commands/corpus/load.rs`

## What this packet is

`ecaz corpus load` accepts both native CLI flags (`--m`, `--ef-construction`,
`--storage-format`, and the implicit `build_source_column` for HNSW) and a
generic `--reloption key=value` passthrough. Today the passthrough is
appended *after* the built-ins in `plan_index_jobs`, so

```
ecaz corpus load --prefix X --m 16 --reloption m=32
```

silently emits a reloption list with `m` twice — Postgres rejects that at
`CREATE INDEX`, and when it doesn't, the `--reloption` value quietly wins
over what the operator asked for on the command line. Either outcome is
worse than a clear up-front error.

This packet rejects these collisions before we touch the database and
points the operator at the native flag equivalent.

## What changed

### `crates/ecaz-cli/src/commands/corpus/load.rs`

- New helper `reloption_flag_collisions(profile, reloptions, storage_format)`
  that returns the intersection of `--reloption` keys and CLI-managed
  keys for the active profile:
  - HNSW: `m`, `ef_construction`, `build_source_column` (always managed
    — the built-in set is emitted unconditionally for the M sweep).
  - Any profile: `storage_format`, but only when `--storage-format` was
    passed. If the operator uses `--reloption storage_format=...` alone,
    that's fine — the CLI isn't also setting it.
- Call sites: after the unknown-reloption warning in `run()`, before
  planning index jobs. A collision is a hard error (not a warning)
  because silent precedence here is the UX failure we're trying to
  prevent.
- Error message format:
  ```
  --reloption m=... conflicts with --m; --reloption storage_format=...
  conflicts with --storage-format. Use the native CLI flag or drop the
  --reloption, not both
  ```
- Five new unit tests covering:
  - HNSW `m` collision
  - HNSW `ef_construction` + `build_source_column` collisions
  - `storage_format` collision only when `--storage-format` is set
  - DiskANN `m=` reloption *not* flagged (DiskANN has no `--m` flag;
    `profile.unknown_reloption_keys` already handles this separately)
  - Empty result when there's no overlap

## Why this slice

- Wholly inside `crates/ecaz-cli/src/commands/corpus/load.rs`; no files
  in the native-build-lane conflict surface are touched.
- Closes a concrete operator-UX gap that would otherwise surface as an
  opaque Postgres error at `CREATE INDEX` time — the operator wouldn't
  know the duplicate came from their own invocation.
- Stacks cleanly on 11054 (unknown-reloption warning) and 11055
  (default_sweep): same `--reloption` surface, same "catch mistakes in
  the CLI, not in Postgres" philosophy.

## Test evidence

```
$ cargo test -p ecaz-cli 2>&1 | tail -3
test result: ok. 182 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out;
finished in 0.01s
```

Up from 180 in packet 11055 (+5 collision tests, -3 removed duplicates…
actually +5 net; 177 + 5 = 182).

## Follow-ups intentionally not in this packet

- Per-profile `managed_reloption_keys` on `IndexProfile` so the
  collision set is declarative rather than hand-coded in `load.rs`.
  Worth doing once a third profile lands; today the two AMs are
  readable inline.
- Applying the same guard to a hypothetical `ecaz corpus rebuild`
  command. The rebuild flow doesn't exist yet.
- Surfacing the native-flag→reloption mapping in `--help`. Clap's
  help macro doesn't make runtime-resolved help text easy; deferred.
