# Task 229 packet 002 — reviewer seq-02 disposition

Source checkpoint: `56a1b37fc632cee8a12dd3e0c32b138afdea3466`.

1. **Persisted-byte rejection tests — closed.** The focused payload-sidecar
   suite now exercises maximum-count mismatch, empty and over-16 covers,
   attnum zero/duplicate/reversal, interior NUL, empty namespace/type/send/
   receive identity, nonempty collation, type/width disagreement, truncated
   descriptor, non-UTF-8 identity, invalid requested TID, and a genuinely
   truncated null bitmap. The test also retains padding, value truncation,
   trailing-byte, row-TID echo, and `vec_id` echo cases.
2. **Per-row allocation cost — closed.** `fixed_binary_width` now takes borrowed
   namespace/name strings, so descriptor validation creates no temporary schema
   attribute or cloned identity strings. Payload encode/decode operate on a
   descriptor already validated at resolution or decode and do not re-run
   immutable identity validation for each row. Descriptor fields remain
   crate-private; the persisted descriptor enters through strict decode and the
   build descriptor through strict resolution.
3. **Dead-code caveat — carried accurately.** The codec is now wired into
   generation descriptor V3 encode/decode, T2 generation construction, manifest
   binding, and build-candidate validation. Entry payload encode/decode still
   has no production row caller because the physical heap/index pair is the
   next authorized checkpoint; request seq-03 does not use clean compilation as
   proof that the future read/write path exists.
4. **T2 reloption drift — closed.** T2 stores `resolved_payload_cover` from the
   preflight/registration-lock window and moves that exact value into the
   generation descriptor. There is no second `relation_options` read or cover
   resolution after `replay_registration`.
5. **Claim precision — closed.** Request seq-03 does not claim the encode-side
   258-byte guard is independently reachable, and it does not claim the
   per-attribute schema loop rather than the fingerprint check is what the
   schema-mutation test exercises.
