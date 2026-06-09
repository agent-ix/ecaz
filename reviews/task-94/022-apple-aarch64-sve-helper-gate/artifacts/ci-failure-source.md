# Existing CI Failure Source

- PR: https://github.com/agent-ix/ecaz/pull/19
- Existing failed check inspected; no CI rerun was requested.
- `pg18 / stable`: https://github.com/agent-ix/ecaz/actions/runs/27230037274/job/80407424509
- Run head: `45dab30558b7c5f75b4ddb907a461b3e07f1915a`
- Runner: `macos-14-arm64`

After packet 021 gated the SVE assembly off Apple aarch64, the next Apple
aarch64 job advanced past the previous Mach-O assembly error and failed on the
remaining SVE-only helper:

```text
error: function `centroid_index` is never used
--> src/quant/grouped_pq_block/sve.rs:149:4
= note: `-D dead-code` implied by `-D warnings`
```

Root cause: `centroid_index` is only used by `score_block32_sve_impl`, which was
correctly gated to non-Apple aarch64 in packet 021. The helper retained the
broader `#[cfg(target_arch = "aarch64")]` gate.

Fix: apply the same non-Apple aarch64 gate to `centroid_index`.
