# Release-Rebuild PG18 RaBitQ / IVF / SPIRE Rerun

## Scope

Follow-up to packet 397
(`reviews/task-50/397-current-head-pg18-rabitq-ivf-spire-sweep`), which
flagged a 5–10× local latency regression vs the 2026-05-19 baseline and
hypothesized that the installed PG18 extension was a debug build rather than
release. This packet rebuilds the extension release-mode and re-runs the
identical RaBitQ suite to confirm or refute that hypothesis.

## Build / install

```
cargo pgrx install --release --no-default-features --features pg18 \
  --pg-config /home/peter/.pgrx/18.3/pgrx-install/bin/pg_config
```

(`make install` defaulted to pg14 and failed; pg18 had to be passed
explicitly. The Makefile target should be hardened — see Issues below.)

Installed `.so` verification after the install:

```
2026-05-21 22:41:10  17,202,072  /home/peter/.pgrx/18.3/pgrx-install/lib/postgresql/ecaz.so
2026-05-21 22:41:09  17,202,072  target/release/libecaz.so
cmp INSTALLED == target/release/libecaz.so (exit 0, byte-identical)
```

Packet 397's installed `.so` was 248,904,952 B (byte-identical to
`target/debug/libecaz.so`). Same source, different build mode.

## Result — hypothesis confirmed

Recall is unchanged from packet 397 (as expected — debug vs release does not
change scoring semantics). **Latency snaps back to baseline territory across
both lanes:**

| Lane             | nprobe | p50 release (398) | p50 baseline (2026-05-19) | p50 debug (397) | release vs baseline | release vs debug |
| ---------------- | ------ | ----------------- | ------------------------- | --------------- | ------------------- | ---------------- |
| IVF/RaBitQ 10k   | 8      | **4.86 ms**       | 5.00 ms                   | 48.7 ms         | −2.8 %              | **−90.0 %**      |
| IVF/RaBitQ 10k   | 16     | **8.08 ms**       | 8.86 ms                   | 75.9 ms         | −8.8 %              | −89.4 %          |
| SPIRE/RaBitQ 10k | 8      | **33.5 ms**       | 39.0 ms                   | 226.2 ms        | −14.1 %             | −85.2 %          |
| SPIRE/RaBitQ 10k | 16     | **65.2 ms**       | 74.2 ms                   | 396.5 ms        | −12.1 %             | −83.6 %          |

Means and tails track the same pattern (see `artifacts/results.jsonl`). The
~10× IVF and ~6× SPIRE regression reported in packet 397 is entirely
explained by the installed extension being a debug build.

**No source regression from the task-50 unsafe-block consolidation work is
detectable on the local 10k RaBitQ lanes.** SPIRE 10k is in fact slightly
faster than the May-19 baseline; IVF is within run-to-run noise.

## Issues / follow-ups

1. **`make install` defaults to pg14 and silently picks a wrong feature.**
   The Makefile target on line 145 is `cargo pgrx install --sudo --release`
   with no `--features pg18`/`--pg-config`. On this host the first
   `pg_config` on `$PATH` is the system pg14, so the install fails with
   `the package 'ecaz' does not contain this feature: pg14`. Recommend the
   Makefile target take a `PG=18` variable and pass `--features pg$(PG)
   --pg-config $(HOME)/.pgrx/$(PG).*/pgrx-install/bin/pg_config`. Tracking
   here so the next coder doesn't get burned the same way.
2. **It is easy to silently install a debug `.so` over a release one.**
   `cargo pgrx install` (no `--release`) was almost certainly run during
   today's merge-validation cycle and overwrote the working release `.so`.
   Suggest either:
   - a tiny CLI guard (`ecaz dev check-extension`) that reads
     `pg_extension` + `pg_config --libdir/ecaz.so` size and warns if it
     looks like a debug build (>100 MB or contains `debug_info`); or
   - making the benchmark suite emit a `build_mode` field into the
     `suite-manifest.json` so any review evidence with debug-mode numbers
     is self-flagging.
3. **Suite still uses TCP `localhost` and reduced sample sizes vs the
   May-19 baseline** (`queries_limit=50`, `iterations=50` vs 200/1000;
   `--host localhost` vs `/home/peter/.pgrx`). Carried over from packet
   397 for direct A/B comparison; for the AWS-readiness measurement,
   restore the baseline-parity values.
4. **SPIRE not exercised above 10k here** (per the ≤25k project rule). A
   25k SPIRE pass should be added before AWS optimization, and 25k IVF /
   25k SPIRE numbers should be diffed against the May-19 25k baseline.

## Artifacts

See `artifacts/manifest.md`.
