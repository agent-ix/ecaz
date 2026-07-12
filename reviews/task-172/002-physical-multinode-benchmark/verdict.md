# Verdict: Task 179 evidence complete; Task 172 remains open

## Task 179 closeout evidence

This packet supplies the measurement requirement referenced by Task 179 acceptance
criterion 13: same-commit physical-versus-single ec_distann recall, latency, and
storage at 10k / 50k / 100k, driven entirely by `ecaz bench suite`.

- Physical recall equals single-instance recall at every scale (delta 0.0000),
  meeting the `single - 0.001` relative gate.
- Physical latency is measured at every scale. Mean recall-query overhead is
  11.68× / 10.56× / 9.36× versus the single arm; this is a serious performance
  finding, not hidden by a correctness-only verdict.
- Physical cluster generation storage is measured at every scale. Raw-vector
  amplification is 3.9512× / 4.0454× / 4.0635×. The 50k and 100k points slightly
  exceed 4.0× and therefore do not support an NFR-018 promotion claim.
- Every topology gate proves exact global coverage, one row per record, zero
  non-owned residue, and zero orphans. At 100k the owner counts are
  33,195 / 33,432 / 33,373; maximum deviation from the mean is 0.296%.
- Both remote owners pass explicit frozen-row materialization probes at every
  scale, and the physical lane uses no replicated graph build/prune path.

Subject to outside-reviewer acceptance, this is sufficient factual evidence to
close Task 179. A poor latency or storage result is a product finding; it does not
erase the completed physical placement implementation or its measured A/B matrix.

## Task 172 status

Task 172 itself remains open and must not be promoted from this packet. Missing
full Task 172 surfaces include:

- the required throughput/concurrency curve;
- first-class per-query remote expand/materialize call and byte counters;
- benchmark-mode versus full-metrics overhead audit;
- broader query samples (this packet uses 10 recall queries and 5 latency iterations);
- per-process CPU/RSS/IO attribution and a defensible 1m/10m capacity model; and
- remediation/retest of the slight 50k/100k NFR-018 storage overage and severe
  distributed latency overhead.

The single-arm latency distribution also contains a cold first-query outlier while
the artifact is labeled `warm` (p50 is a few milliseconds but p95/p99 are seconds).
The raw tables are preserved; follow-up Task 172 work must add explicit warmup rather
than reinterpret these samples.

