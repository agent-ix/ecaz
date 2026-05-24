# Review Request: Local IVF Adaptive Nprobe Smoke

## Scope

Code commits under review:

- `80f3476c265d1d1281449242c1315ecc81ecb8c6` - opt-in IVF adaptive nprobe
- `2ce73bdc3e82e840a7d6a15e7b36d066e8fddce7` - benchmark harness support

Benchmark packet:

- `benchmarks/task51-local-ivf-adaptive-nprobe/`

This is a local PG18 smoke suite against the preserved 990k IVF/RaBitQ table
and index from packet 009. It does not rebuild the corpus, does not use AWS,
and does not run vchord or pgvectorscale.

## Result

Suite status:

```text
completed=8 failed=0 skipped=0 dry_run=0 missing_artifacts=0 stale=0
```

Main finding: the current adaptive policy is implemented and measurable, but it
is not ready to promote.

- `gap=1000` improves local p50 at nprobe 64 (`290.1 ms -> 225.8 ms`) but
  drops recall (`0.9570 -> 0.9490`) and recall tail (`p10 0.8900 -> 0.8000`).
- `gap=10000` and `gap=100000` preserve recall on this q=100 smoke, but behave
  close to static probing and do not produce a material p50 win.

## Caveats

- q=100 is a smoke waiver, not final adaptive-policy evidence.
- This packet does not report worst-query recall; the current runner reports
  p10/p50/p90 but not worst-query recall.
- A debug local extension install was canceled before benchmark evidence was
  collected. The authoritative run is after release local install:
  `benchmarks/task51-local-ivf-adaptive-nprobe/artifacts/suite-run-release.log`.
- This result says "do not promote the current adaptive policy yet"; it does
  not close Task 51.

See `artifacts/manifest.md` for packet-local artifact details.
