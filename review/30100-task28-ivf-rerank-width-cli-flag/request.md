# Task 28 IVF rerank width CLI flag

## Scope

This packet covers commit `8d3106c` (`ecaz-cli: add IVF rerank width bench flag`).

The ecaz benchmark CLI can now pin IVF heap-rerank frontier width while sweeping `ec_ivf.nprobe`:

- `ecaz bench recall --profile ec_ivf --sweep ... --rerank-width ...`
- `ecaz bench latency --profile ec_ivf --sweep ... --rerank-width ...`

`--rerank-width` accepts the same values as the extension GUC:

- `-1`: use the index `rerank_width` reloption.
- `0`: rerank the full probed frontier.
- positive values: bound heap-rerank to that frontier width.

The flag is rejected for non-IVF profiles so HNSW and other lanes do not silently carry an unrelated IVF GUC.

## Code paths

- `crates/ecaz-cli/src/commands/bench/recall.rs`: adds `--rerank-width`, validates it for IVF, and sets `ec_ivf.rerank_width` on the recall connection.
- `crates/ecaz-cli/src/commands/bench/latency.rs`: adds the same flag and sets it on each worker connection before timed queries.
- `crates/ecaz-cli/src/profiles.rs`: records `rerank_width` and `pq_group_size` as known IVF reloptions.

## Validation

Focused validation run:

- `cargo test -p ecaz-cli`
- `git diff --check`

All passed.

## Why this matters for Task 28

The next IVF sweeps can vary `nprobe` with `--sweep` and hold rerank width with a normal CLI flag. This avoids repeated `ALTER INDEX` calls and keeps measurement commands review-packet friendly with `--log-output`.
