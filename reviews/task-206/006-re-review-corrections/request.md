---
agent: claude
role: coder
model: gpt-5
date: 2026-08-04
seq: 1
---

# Task 206 re-review corrections

This packet supersedes the Task 206 packet-005 seed-count conclusion. The
release A/B there was structurally inert: the uninstrumented build compiled
the benchmark GUCs out, so both requested seed-count arms used the production
derivation `(beam_width * 2).max(32)`. The packet-005 k-head comparison is
withdrawn.

The corrected run below uses the attribution-feature PG18 build, enables the
scan-profile session GUC, and records the effective `head_seed_count` from the
physical scan path. It keeps the same BW64/H8, three-owner, 10k/50k/100k
matrix and compares only effective seed count 128 versus 200. Feature-build
latency is diagnostic and is not mixed with the release latency matrix.

The shipped defaults are stated correctly here: the current production
default is BW4/H100. BW64/H8 is the measured recommendation for a separate
productionization task; this packet does not change that default.

The code checkpoint also makes physical per-round notices truthful: default
builds report unavailable attribution fields as `absent`, while the feature
lane reports measured transport fields and the effective seed count.

## Review evidence

- Suite config: `artifacts/task206-feature-seed-ab.json`
- Run artifacts: `artifacts/run/`
- Code head: `53d1fec9f`
- Cache test and both default/feature PG18 compile checks are recorded in the
  validation artifact once the feature run completes.

