# Task 111h / Packet 043 Review Request: Compact Exact-Dequant Score Mode

This packet requests review for commit
`1ed1cd9e55e825d9e2b739db168baf5ea749d526`
(`task111h: add compact exact dequant rerank mode`).

## Scope

Packet 041 found that the compact-format closeout never ran the second fidelity
lever named by packet 024: exact-dequant scoring. This checkpoint adds that
lever to the common persisted rerank scorer rather than adding a format-specific
one-off.

Changes:

- Adds `rerank_exact_dequant = 1` as a compact rerank score-mode reloption.
- Resolves score mode from mutually exclusive flags:
  - default estimator / format scorer,
  - RaBitQ least-squares,
  - exact-dequant.
- Persists the score mode in metadata byte `22` as a three-value enum:
  `0 = estimator/default`, `1 = rabitq least_squares`, `2 = exact_dequant`.
- Bumps IVF metadata format version from v8 to v9 because byte `22` now has an
  expanded persisted value domain.
- Implements RaBitQ exact-dequant scoring as
  `||o|| * <q, x_dec> / ||x_dec||`.
- Implements TurboQuant exact-dequant scoring as an MSE-dequantized vector dot
  over the persisted code bytes. The existing TurboQuant default scorer remains
  the default MSE/QJL score path.
- Threads the mode through scalar sidecar scoring, contiguous batch scoring,
  and TurboQuant borrowed payload-ref scoring.
- Updates fixture/docs/upgrade matrix so v8 is legacy rejected and v9 is the
  current writable IVF format.

## Validation

Packet-local logs are under `artifacts/`.

```text
CARGO_INCREMENTAL=0 cargo test --no-default-features --features pg18 exact_dequant --lib
CARGO_INCREMENTAL=0 cargo test --no-default-features --features pg18 --test on_disk_fixtures ivf_metadata
CARGO_INCREMENTAL=0 cargo test --no-default-features --features pg18 --test upgrade_matrix
CARGO_INCREMENTAL=0 cargo test --no-default-features --features pg18 --test size_of_assertions
CARGO_INCREMENTAL=0 cargo check --no-default-features --features pg18
```

All passed.

## Non-Claims

This packet does not claim that exact-dequant improves recall or latency. It
only lands the scorer/metadata lever required for the corrected 111h sweep.
The next benchmark packet still needs to run RaBitQ4/RaBitQ8/TurboQuant with
the new mode and the requested clip/matched-recall matrix.
