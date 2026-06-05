# Task 65b Packet 014: Batched Backlink Reducer

This packet covers the reducer-side optimization slice after packet 013 showed
the deterministic reducer dominating both real10k and real100k builds.

## Code Change

New code commit: `f3809bf18` (`Tune DiskANN parallel reducer pruning`).

The slice:

- removes duplicate candidate sorting before `robust_prune`, since
  `robust_prune` already sorts;
- batches adjacency replacement for non-overflow backlink appends instead of
  replacing the `Arc<[u32]>` row once per appended backlink;
- moves parallel pivot out-neighbor commits through the owned-Vec replacement
  path;
- keeps growing-alpha pruning for `batch_size=1` serial-equivalence scaffolding,
  while true parallel batches prune directly at the configured final alpha to
  avoid a repeated intermediate-alpha distance pass.

Worker-count `0` fallback remains on the serial Task 65 path. Worker-count `1`
serial-equivalence tests remain green.

## Validation

Packet-local logs are under `artifacts/`; metadata is summarized in
`artifacts/manifest.md`.

- `cargo fmt --check`: passed.
- `cargo check -p ecaz --lib --no-default-features --features pg18`: passed.
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana::tests::task65b_`: passed, 5 tests.
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::build::tests::task65b_`: passed, 5 tests.

## Measurement Summary

The real10k gate now has a passing candidate:

| fixture | workers | batch | build total | reducer | recall@10 L200 | gate |
|---|---:|---:|---:|---:|---:|---|
| real10k | 8 | 64 | `2.873s` | `2.041s` | `0.9950` | passes time and recall |
| real10k | 8 | 96 | `2.737s` | `1.928s` | `0.9920` | fails recall by `0.0005` |
| real100k | 8 | 64 | `139.356s` | `122.176s` | `0.9750` | fails time |

The Task 65 real10k L200 baseline is `0.9975`, so the 0.5pp floor is
`0.9925`. The b64 row is the current real10k default candidate.

## Gate Status

This is progress, not final task closure.

- real10k `<=3s`: now satisfied by `w8/b64`.
- real10k recall within 0.5pp: now satisfied by `w8/b64`.
- real100k `<=30s`: not satisfied; latest `w8/b64` is `139.356s`.
- real100k recall: held at `0.9750` for `w8/b64`.
- reducer remains the bottleneck at real100k scale (`122.176s` of `139.356s`).

## Review Ask

Please review the final-alpha pruning policy for true parallel batches. The
policy is intentionally scoped away from worker-count `0` fallback and
batch-size `1` serial-equivalence scaffolding, but it changes the graph produced
by multi-worker builds and should be treated as the main correctness/design
question in this packet.
