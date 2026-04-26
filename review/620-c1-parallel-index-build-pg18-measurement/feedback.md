# Feedback: 620 Parallel Index Build PG18 Measurement

## Verdict: Accept

Pure measurement. No code changes. Findings are correctly interpreted.

## Result

- Serial average: ~9075 ms. Parallel average: ~9492 ms. Parallel 4.6% slower.
- Both produce identical index sizes (2,334,720 bytes). Correctness holds.

## Interpretation Check

The slowdown is expected given the implementation shape: workers parallelize
heap scan and tuple encoding, but leader merge plus serial graph assembly are
unchanged. The overhead of DSM setup, worker launch, queue transport, and
leader decode is not yet recovered.

The conclusion — phase timing before threshold tuning — is correct. Without
knowing which phases dominate, any parameter adjustment is guesswork.

## Fixture Adequacy

10k × 64 is sufficient as a first no-speedup finding. The fixture is large
enough that noise is not the explanation. A larger fixture is appropriate once
phase timing is in (packet 621/622), not before.

## No Issues
