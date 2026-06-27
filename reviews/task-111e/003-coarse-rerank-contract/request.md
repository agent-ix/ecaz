# Task 111e: Coarse Rerank Contract Reloptions

## Summary

This packet adds the explicit Task 111e contract knobs on top of the existing
`storage_format = 'coarse_rerank'` preset:

```text
coarse_format = 'rabitq'
coarse_bits = 1
rerank_placement = 'table'
rerank_format = 'f32'
```

The current implementation normalizes `auto` to that baseline for
`coarse_rerank`, keeps the existing dense RaBitQ-1 + heap-f32 scan behavior,
and rejects unsupported contract combinations early:

- `coarse_bits` wider than 1 is rejected for this mode;
- `rerank_placement = 'index'` is rejected until index-side payloads land;
- compact rerank formats (`rabitq2`, `rabitq4`, `rabitq8`, `turboquant`) are
  parsed but rejected for `coarse_rerank` until their implementation packets.

The contract is also exposed in `ec_ivf_index_admin_snapshot` via
`coarse_format`, `coarse_bits`, `rerank_placement`, and `rerank_format`.

## Code Under Review

- `src/am/ec_ivf/options.rs`
- `src/am/ec_ivf/admin.rs`
- `src/lib.rs`
- struct constructor updates in `build.rs`, `cost.rs`, `insert.rs`, `page.rs`,
  and `scan.rs`

## Validation

Artifacts are under `reviews/task-111e/003-coarse-rerank-contract/artifacts/`.

```text
cargo test -q coarse_rerank --lib --no-default-features --features pg18
5 passed; 0 failed; 2131 filtered out

cargo test -q metadata_roundtrip --lib --no-default-features --features pg18
8 passed; 0 failed; 2128 filtered out

cargo test -q scratch_soa_batch_decode_gate --lib --no-default-features --features pg18
1 passed; 0 failed; 2135 filtered out
```

## Review Ask

Please review whether this contract surface is the right narrow baseline before
adding quantized rerank variants and index-side placement.
