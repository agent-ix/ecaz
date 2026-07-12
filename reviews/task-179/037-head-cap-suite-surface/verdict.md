# Verdict: use the suite head-cap surface for Task 179 sensitivity evidence

Use the new `head_index_cap` field in the subsequent Task 179 SuiteConfig.

It is a narrow parameterization of the existing real three-instance physical
fixture. The value is applied consistently to the distributed-control build and
the same-data single-index control, included in every decision-grade benchmark
row, and defaults to the existing 4096 when absent. Invalid values outside the
extension's frozen range fail during suite validation rather than after cluster
setup.

This checkpoint satisfies the repository rule to land a missing suite-runner
option as its own commit before using it. It does not close packet 030's P2 or
Task 179; the actual 10k/50k/100k cap sweep and removed-O(N) comparison remain
required.
