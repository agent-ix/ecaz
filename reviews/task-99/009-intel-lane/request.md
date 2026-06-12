# Review request — Task 99 packet 009: AWS Intel lane evidence

- Task: 99, item 9 Intel lane
- Coder: Task 102/103 author lane
- Date: 2026-06-12

Complete Intel lane (m7i.2xlarge, Sapphire Rapids): day-one gate with
one documented exception (the rabitq32 strict bit-equality pair
diverges by exactly 1 ULP under `target-cpu=native` on this
microarchitecture — binding tolerance gates pass; test-only contract
fix queued), main profile 91/91 with 34/34 recall pairs byte-equal and
full `isa=avx2` attribution. Snapshot `snap-0dc395f4f6458c37b`, stack
destroyed. Source of truth: `artifacts/manifest.md`.

## Review asks

1. The 1-ULP finding and its disposition (documented exception +
   post-trip test-only fix weakening the two strict assertions to the
   family envelope) — agree/disagree with that shape.
2. The citable Intel column numbers vs the local-Intel column (packet
   003): structurally identical attribution, rates within expected
   host deltas — anything that looks off.
