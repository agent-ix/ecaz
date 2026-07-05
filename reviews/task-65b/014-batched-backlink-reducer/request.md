# Task 65b Packet 014: Batched Backlink Reducer

This packet covers the reducer-side optimization slice after packet 013 showed
the deterministic reducer dominating both real10k and real100k builds.

## Code Change

Code commits:

- `f3809bf18` (`Tune DiskANN parallel reducer pruning`)
- `7034ac5fc` (`Parallelize DiskANN backlink reduction`)

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
- parallelizes backlink target planning/reprune computation and then applies the
  planned adjacency replacements in deterministic target order;
- reports `parallel_effective_workers` from the active Rayon pool instead of
  echoing the requested reloption.

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
| real10k | 8 | 64 | `1.080s` | `0.256s` | `0.9950` | passes time and recall |
| real10k | 8 | 96 | `2.737s` | `1.928s` | `0.9920` | fails recall by `0.0005` |
| real100k | 8 | 64 | `36.490s` | `19.115s` | not rerun after `7034ac5fc` | fails time |
| real100k | 8 | 768 | `29.771s` | `14.475s` | `0.9700` | passes time; recall is exactly 0.5pp below the b64 supporting row |

The Task 65 real10k L200 baseline is `0.9975`, so the 0.5pp floor is
`0.9925`. The b64 row is the current real10k default candidate.

## Gate Status

This is progress, not final task closure.

- real10k `<=3s`: now satisfied by `w8/b64`.
- real10k recall within 0.5pp: now satisfied by `w8/b64`.
- real100k `<=30s`: now satisfied by `w8/b768` at `29.771s`.
- real100k recall: `w8/b768` reaches `0.9700` at L200. The prior `w8/b64`
  supporting recall row was `0.9750`; `w8/b64` recall was not rerun after the
  backlink-planner-only code change.
- reducer remains the largest real100k graph-build component, but the parallel
  backlink planner reduced the `w8/b768` reducer section to `14.475s` of
  `29.771s`.
- reviewer 014 is still `APPROVE-WITH-FLAGS`, not closeout: the next slice
  should either enforce/document the batch-size recall surface in the AM or
  explicitly choose the stop-condition path.

## Review Ask

Please review the final-alpha pruning policy for true parallel batches. The
policy is intentionally scoped away from worker-count `0` fallback and
batch-size `1` serial-equivalence scaffolding, but it changes the graph produced
by multi-worker builds and should be treated as the main correctness/design
question in this packet.
