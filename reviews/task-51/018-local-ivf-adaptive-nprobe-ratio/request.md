# Review Request: Task 51 Exp 5 Adaptive Nprobe Ratio Follow-Up

Code under review:

- `7e215f5edf9bc4e8dd906bc2d36f861ae9f00b61` - Add IVF adaptive nprobe margin ratio signal

Benchmark packet:

- `benchmarks/task51-local-ivf-adaptive-nprobe-ratio/`

This packet addresses reviewer feedback on packet 014:

- `ecaz bench recall` now reports `recall_worst`
- suite config exercises a non-time adaptive signal:
  `ec_ivf.adaptive_nprobe_score_margin_ratio_bps`
- only IVF/RaBitQ was run; no vchord, no pgvectorscale

Local suite status:

```text
[suite:task51-local-ivf-adaptive-nprobe-ratio] completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Main result:

- Recall and worst-query recall match static for ratio thresholds 2500, 10000,
  and 50000 bps on q=100.
- Latency does not produce a useful win. `ratio=2500` is worse than static
  across p50/p95/p99. Stricter ratios are mixed or near-static, not a
  promotion candidate.

Recommendation:

- Accept this as the Exp 5 closeout.
- Do not promote adaptive nprobe to AWS.
- Keep the implementation default-off as diagnostic/experimental code unless a
  future task defines a better policy.

Known validation note:

- `cargo test -p ecaz --lib adaptive_nprobe` built but could not execute in
  this local shell because the pgrx test binary failed dynamic lookup with
  `undefined symbol: LockBuffer`, matching the existing local pgrx-test
  limitation. CLI tests and the live PG18 suite passed.
