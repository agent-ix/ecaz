# Triage: header.rs mutation campaign

Result: **35 caught, 0 missed, 0 timeouts.** No new tests required.

The existing careful suite (packets 021 + 028 + 029 + 044 + 046 + 047)
already discriminates every operator swap and body replacement on
`src/am/ec_spire/storage/header.rs`. The header's round-trip via every
partition object kind (Routing / Leaf / Leaf V2 / Delta / TopGraph /
Chain) plus the direct `partition_object_header_rejects_invalid_identity`
test from packet 021 covers every guard in
`validate_for_format_version`, `encode_with_format_version`, and
`decode_prefix_with_format_version`.

## Per-mutation map

All 35 mutations target one of three function bodies:

- `SpirePartitionObjectHeader::encode` (3 body-replacement mutations)
- `SpirePartitionObjectHeader::encode_with_format_version`
  (3 body replacements + 1 `!= -> ==` operator swap)
- `SpirePartitionObjectHeader::encode_after_validation`
  (3 body replacements)
- `SpirePartitionObjectHeader::validate_for_format_version`
  (1 body replacement + 4 operator swaps)
- `SpirePartitionObjectHeader::decode_prefix_with_format_version`
  (1 body replacement + 4 operator swaps + 1 delete-token + others)

Each is killed because:

- Body replacements (`-> Ok(vec![])`, `vec![0]`, `vec![1]`,
  `Ok(Default::default())`, `Ok(())`) produce bytes that fail the
  round-trip decode (length mismatch, magic mismatch, kind mismatch),
  or fail validation directly. Every leaf/delta/routing/top_graph
  encode-then-decode test in packets 021 onward catches these.
- Operator swaps in `decode_prefix_with_format_version` (4 `!= -> ==`
  on magic / format_version / kind / reserved checks): flipped
  predicates accept what should be rejected, surfaced by the
  routing/leaf encode-then-decode tests via header-validation errors.
- Operator swaps in `validate_for_format_version`: every encode call
  goes through this validate; flipping its guards either rejects
  known-good headers (round-trip fails) or accepts known-bad
  identities (`partition_object_header_rejects_invalid_identity`
  catches it).

See `artifacts/manual-verification.log` for the per-mutation KILLED
verdict.

## Verification artifacts

- `artifacts/header-mutants-enumerated.txt` — 35-mutation enumeration
  from `cargo mutants --Zmutate-file`.
- `artifacts/run-spire-mutations.py` — generic per-file verification
  helper now used by every cascade packet (was per-file in 046/047).
- `artifacts/manual-verification.log` — **35 KILLED, 0 MISSED, 0
  PATCH-FAIL.**
- `artifacts/post-verification-tests.log` — `cargo test
  --manifest-path hardening/careful/Cargo.toml --lib`: **534 passed,
  0 failed** after every mutation reverted.

Source file byte-for-byte identical to pre-packet state
(`diff /tmp/header_original.rs src/am/ec_spire/storage/header.rs`
returned empty).
