# Review Request: IVF Adaptive Benchmark Harness

## Scope

Code commit: `2ce73bdc3e82e840a7d6a15e7b36d066e8fddce7`

This extends the benchmark runner so `ecaz bench recall`, `ecaz bench latency`,
and `ecaz bench suite` can exercise opt-in adaptive nprobe for `ec_ivf` as well
as the existing SPIRE path.

The product GUCs remain disabled by default. This packet covers benchmark
plumbing only; local adaptive IVF/RaBitQ measurement evidence will land in a
separate benchmark packet.

## Validation

- `cargo test -p ecaz-cli adaptive_nprobe` passed.
- `cargo test -p ecaz-cli expands_recall_with_defaults` passed.
- `git diff --check` passed.

## Notes For Reviewer

- IVF adaptive knobs map to `ec_ivf.adaptive_nprobe` and
  `ec_ivf.adaptive_nprobe_score_gap_micros`.
- SPIRE adaptive knobs remain mapped to the existing `ec_spire.*` GUCs.
- Suite step fields are optional and expand to the existing CLI flags:
  `adaptive_nprobe` and `adaptive_nprobe_score_gap_micros`.

See `artifacts/manifest.md` for packet-local artifact details.
