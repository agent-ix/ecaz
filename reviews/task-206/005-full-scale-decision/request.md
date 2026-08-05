---
agent: claude
role: coder
model: gpt-5
date: 2026-08-04
seq: 1
---

# Task 206 full-scale decision

This packet closes the reviewer-requested corrected measurement lane. The
release, uninstrumented A/B matrix is the decision evidence; the feature build
and telemetry lane are diagnostic only.

## Decision

Keep the production defaults at BW64/H8 with `head_seed_count=128`. The
`head_seed_count=200` arm is a viable alternate: it has the same recall in the
three decision-scale pairs and is about 2--3% faster at 50k and 100k, but the
10k result is mixed and the extra head work does not justify changing the
default without a broader workload. The current BW64/H8 point remains the
recommended Pareto point; no default or task specification change is made.

The release matrix covers recall, latency, and storage at 10k/50k/100k with
50 timed queries and 10 warmups. NFR-021 is conforming: the normalized
growth/bytes-per-record checks pass (maximum normalized growth 1.094707 versus
the 2.0 threshold). The larger physical-cluster total is recorded as a
control-plane/fixture accounting comparison, not substituted for the NFR
normalized storage criterion.

The owner-traversal control is preregistered and run separately with the
feature-enabled diagnostic extension. Its results are not used to claim a
production latency win. The telemetry rerun forwarded
`ec_distann.scan_profile_notice=on` and the parent parser captured child output,
but produced no runtime `ec_distann_scan_round` NOTICE records, so no per-round
numbers are claimed. This is recorded as an instrumentation follow-up rather
than fabricated attribution.

## Evidence

See `artifacts/result-summary.md` and `artifacts/manifest.md`. The immutable
release matrix is under the corrected preregistration packet at
`../004-corrected-closeout/artifacts/run/`; the telemetry and owner-control
lanes are under that packet as well.
