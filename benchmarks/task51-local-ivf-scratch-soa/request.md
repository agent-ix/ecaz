# Benchmark Request: Local IVF Scratch SoA Smoke

## Scope

Benchmark packet:

- `benchmarks/task51-local-ivf-scratch-soa/`

Code commit measured:

- `a22ca84531379581855613a2968a2ca8aca14a5b` - opt-in IVF scratch SoA batch decode

This is a local PG18 smoke suite against the preserved 990k IVF/RaBitQ table
and index from packet 009. It does not rebuild the corpus, does not use AWS,
and does not run vchord or pgvectorscale.

## Result

Suite status:

```text
completed=6 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Main finding: the opt-in scratch-SoA path is functional and preserves recall,
but the measured local win is small and does not meet the Task 51 Exp 3 gate.

- Recall parity held: both static and scratch SoA report recall@10 `0.9750`,
  recall p10 `0.9000`, and NDCG@10 `0.9986`.
- Latency p50 improved `603.7 ms -> 590.5 ms`, about 2.2%.
- EXPLAIN execution improved `586.336 ms -> 570.902 ms`, about 2.6%, with
  identical posting/candidate counts.
- This is below the Exp 3 local gate of at least 20% candidates/sec improvement.

## Decision

Do not promote this scratch-SoA prototype to AWS as a standalone optimization.
Also do not pursue Posting Layout v2 from this evidence: Task 51 says Layout v2
requires the scratch SoA prototype or counters to prove posting decode/scan is
a primary bottleneck, and this packet does not show a material scan gain.

See `manifest.md` for artifact details and exact commands.
