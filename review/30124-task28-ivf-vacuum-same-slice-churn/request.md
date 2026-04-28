# Task 28 IVF Same-Slice Churn Diagnostic

## Scope

This packet checks whether the nlists=64 growth seen in packets 30120/30121/30123 is caused only by a shifting delete/refill distribution.

The earlier churn script deletes the original second half in cycle2 but refills with the first-half embedding pattern again. This diagnostic repeatedly deletes and refills the same first-half embedding slice, keeping the live-list population more stable.

## Result

At head `e9ca634c`:

- nlists=32 converged through cycle3 at `4,464,640` bytes.
- nlists=64 grew from `4,472,832` bytes at build to `4,825,088` bytes at cycle3.
- nlists=64 cycle3 refill was `124,588.292 ms`.

Comparison to packet 30123 drifting-shape churn:

- nlists=64 cycle3 size improved from `4,964,352` bytes to `4,825,088` bytes.
- nlists=64 cycle3 refill improved from `134,251.989 ms` to `124,588.292 ms`.

## Interpretation

Distribution drift contributes to the prior nlists=64 growth, but it is not the whole problem. Even same-slice churn still grows beyond the build size and remains slow.

The next A3 design should stop treating list-local backward scans as the whole reuse story. A persistent or rebuildable free-page sidecar that can reuse empty posting capacity across list boundaries is the likely next step for sustained churn.

## Artifacts

- `artifacts/ivf_same_slice_churn_smoke.sql`
- `artifacts/ivf_same_slice_churn_smoke.log`
- `artifacts/manifest.md`
