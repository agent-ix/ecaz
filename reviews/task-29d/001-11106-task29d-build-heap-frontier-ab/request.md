# Task 29d Build Heap-Frontier A/B

## Request

Review the release-mode A/B for the build-side heap-frontier experiment.

Current branch head after recording the decision:
`d4cbcd86c24a9f73cb91e9da3688f588a20665e8`

The measured experimental working tree was current Task 29d head with the old
heap-frontier build commits applied without landing them:

- `d2e0e9fc` — `Use heap frontier for Vamana build search`
- `36f0c3d5` — `Fix Vamana build frontier truncation`

## Summary

Do not reland the build-side heap-frontier experiment.

The release-mode A/B replicated the earlier debug-mode regression:

| checkpoint | total_ms | build_persist_ms | core_graph_ms | pass0_ms | pass1_ms |
| --- | ---: | ---: | ---: | ---: | ---: |
| active-mask baseline (`11104`) | `70,678` | `69,000` | `67,571` | `20,737` | `46,832` |
| heap-frontier A/B (`11106`) | `75,492` | `73,242` | `71,617` | `21,933` | `49,683` |
| delta | `+6.8%` | `+6.1%` | `+6.0%` | `+5.8%` | `+6.1%` |

The Task 29d decision matrix said a ≥5% loss means the asymmetry is real and
the reverted code should stay reverted. This run clears that bar: total build
time regressed by `4.814s`, and the core graph phase regressed by `4.046s`.

The likely explanation remains the one from the plan: build-side frontier sizes
are too small and too frequently pruned for heap maintenance to amortize,
whereas the scan-side heap frontier works on a different workload shape.

## Recommendation

Leave `d2e0e9fc` and `36f0c3d5` out of the landing branch. Treat 29d-1 as
complete and move to 29d-2, the L=64 scan latency profile.

No production code change is included in this packet. The local PG18 scratch
server was restored to the current non-experimental branch head after the A/B.

## Validation

- `cargo fmt --check` passed before the A/B.
- `cargo test --lib am::ec_diskann::vamana -- --nocapture` passed with the
  experimental heap-frontier patch applied: `9 passed; 0 failed`.
- `git diff --check` passed before committing this packet.

## Artifacts

See `artifacts/manifest.md`.
