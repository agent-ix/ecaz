---
task: 229
packet: 002-format-and-lifecycle
agent: Codex
role: coder
model: gpt-5
date: 2026-08-26
seq: 03
---

# Task 229 format/lifecycle — checkpoint 3 review

Review source commit `56a1b37fc632cee8a12dd3e0c32b138afdea3466`
against exact main `3419c9c758bea7d9940b27d9afbcf9e627e84879`.
Checkpoints 1 and 2 are review-closed DONE in
`feedback/2026-08-26-{01,02}-reviewer.md`; checkpoint 2's carried items are
dispositioned in `artifacts/seq02-disposition.md`.

This is the third narrow checkpoint of packet 002, not a claim that the complete
format/lifecycle packet is finished. It implements the accepted versioned
generation/receipt/manifest/fingerprint identity chain and every variable-width
receipt consumer. Physical sidecar relations and their five-relation lifecycle
ownership remain checkpoint 4; until then a declared cover cannot produce V2
Ready receipts and therefore fails closed before candidate publication.

## Implemented

- `DistannGenerationDescriptor` remains byte-identical V2 with no cover. A
  covered descriptor is V3 and appends the exact canonical V1 cover descriptor
  plus its digest. Decode accepts V2/V3, validates the embedded digest and exact
  frozen row schema, and re-encodes either version canonically. T2 moves the
  already-resolved, registration-bound descriptor forward; it does not re-read
  reloptions after registration replay.
- `DistannReadyReceipt` remains V1/303 bytes with no cover. Covered V2 receipts
  are exactly 359 bytes and add sidecar row count, explicitly named
  `initial_content_digest`, heap bytes, and index bytes. Row count must equal
  the owner's initial record count. Generation-catalog storage is bounded
  variable length and decodes before use or persistence; bootstrap SQL accepts
  only the exact V1/303 or V2/359 version/length pairs.
- `DistannEpochManifestV2` remains byte-identical V2 with no cover. Covered
  manifests use V3 and add the cover-descriptor digest plus a domain-separated,
  roster-ordered global initial-content digest over each owner's node id, row
  count, and owner initial-content digest. Validation rejects mixed legacy and
  covered receipts, unpaired fields, or any recomputed global-digest mismatch.
- Epoch fingerprints accept V2 and V3. Manifest construction emits the matching
  fingerprint version, build-candidate validation requires the fingerprint and
  manifest versions to agree, and either version is accepted as a parent.
- Ready-receipt-set framing remains under its unchanged V1 domain, uses bounded
  variable-length entries, and decodes both 303-byte V1 and 359-byte V2
  receipts. Every former fixed-303 Rust, SQL, export, and lifecycle-fixture
  consumer is updated. This research-stage bootstrap catalog change requires
  re-bootstrap; future control and candidate A/B arms will both use the
  post-change bootstrap and the same extension binary.
- The descriptor, receipt, manifest, and receipt-set digest domain strings are
  unchanged. New sidecar-only descriptor/global-content domains are separate.
  Frozen V2 descriptor, V1 receipt, V2 manifest, V2 fingerprint, and build
  candidate/receipt-set fixtures all decode and re-encode byte-for-byte.
- Seq-02's persisted-byte rejection matrix is fully exercised, including count,
  attnum order, identity, UTF-8, width, descriptor truncation, invalid requested
  TID, and genuine bitmap truncation. The fixed-width helper no longer allocates
  temporary schema attributes, and hot payload encode/decode no longer repeat
  immutable descriptor validation per row.

## Validation

- `cargo fmt --all -- --check` — pass.
- `cargo check --lib --no-default-features --features pg18` — pass.
- Focused payload-sidecar tests — 6 passed, including every seq-02 decode
  rejection carry-in.
- Legacy identity tests — the V2 descriptor and V2 manifest preserve frozen
  bytes and digests; covered V3 forms round-trip; cross-version parents pass.
- Ready receipt V1/V2 test, receipt-set V1/V2 framing test, and covered
  descriptor/receipt/manifest/fingerprint build-candidate test — all pass.
- Frozen on-disk DistANN fixture suite — 21 passed, including byte-identical
  V2 descriptor, V1 receipt, V2 manifest/fingerprint, and V1 build candidate
  with its Ready-receipt-set framing.
- Full strict clippy reports only the four pre-existing main failures in
  `ambuild.rs`, `generation_descriptor.rs`, `head_sample.rs`, and
  `remote_endpoint.rs`. Three files are byte-identical to main; blame proves the
  `generation_descriptor.rs` lint remains the untouched 2026-08-01 line despite
  Task 229 additions elsewhere in that file. Re-running with only those four
  lint names allowed passes all targets under `-D warnings`.
- No PostgreSQL, `cargo pgrx test`, live fixture, corpus, or benchmark command
  was run. The frozen byte fixtures above are ordinary Rust tests.

Durable output and command provenance are in `artifacts/manifest.md`.

## Review questions

1. Are legacy no-cover V2/V1/V2 bytes, digests, V2 fingerprint, and receipt-set
   framing genuinely preserved under unchanged existing domains?
2. Does the covered V3/V2/V3 chain bind the exact cover descriptor, per-owner
   initial content, roster-ordered global content, and matching V3 fingerprint
   without confusing immutable initial-build identity with later DML state?
3. Is receipt storage now safely bounded variable length across every former
   fixed-303 Rust/SQL consumer, with dual decode and exact version/length gates?
4. Does `artifacts/seq02-disposition.md` fully close the required corruption
   tests and per-row allocation carry-in while threading T2's resolved
   descriptor rather than reopening a reloption drift window?
5. May checkpoint 4 proceed to the cataloged sidecar heap/index pair and all
   five-relation lifecycle ownership surfaces?
