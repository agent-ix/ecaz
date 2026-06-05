# Task 65b Packet 015: Parallel Batch Policy

This packet covers the code slice that addresses packet 014 reviewer flags
F-1/F-2: oversized batch sizes on real10k-class builds needed an AM-visible
policy, and the alpha-growth tradeoff needed to appear in build output rather
than only in measurement notes.

## Code Change

New code commit: `76777656c` (`Surface DiskANN parallel batch policy`).

The slice:

- resolves `parallel_build_batch_size` into requested/effective values;
- caps the effective batch size to `64` for small builds with `n <= 10000`,
  matching the validated real10k recall ceiling from packet 014;
- leaves larger builds uncapped, so the measured real100k `w8/b768` path
  remains available;
- logs a build-policy NOTICE that names the small-build cap, alpha-growth
  policy, and stale-read proxy;
- extends the complete ambuild timing NOTICE with
  `parallel_requested_batch_size`, effective `parallel_batch_size`,
  `parallel_alpha_growth_disabled`, and `parallel_stale_read_ppm`;
- documents the reloption string so operators know requested and effective
  batch can differ;
- adds a focused unit test proving a small build requested at batch `96`
  reports requested `96` and effective `64`.

## Validation

Packet-local validation metadata is in `artifacts/manifest.md`.

- `cargo fmt --check`: passed.
- `cargo check -p ecaz --lib --no-default-features --features pg18`: passed.
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::build::tests::task65b_`: passed, 6 tests.
- `cargo test -p ecaz --lib --no-default-features --features pg18 am::ec_diskann::vamana::tests::task65b_`: passed, 5 tests.

## Gate Status

This is a closeout-supporting slice, not full Task 65b closure.

- Packet 014 real10k `w8/b64` remains the current real10k gate row:
  `1.080s`, Recall@10 L200 `0.9950`.
- Packet 014 real100k `w8/b768` remains the current real100k gate row:
  `29.771s`, Recall@10 L200 `0.9700`.
- This packet makes the real10k recall guard actionable in code for
  small-build batch requests above the validated ceiling.

Remaining closeout blockers still include the older worker-zero Task 65 head
byte-equality proof and the broader task-level measurement/feedback rollup.
