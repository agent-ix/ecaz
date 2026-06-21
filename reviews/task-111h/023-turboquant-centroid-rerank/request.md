# Task 111h / Packet 023 Review Request: Centroid-Relative TurboQuant Rerank Sidecars

## Summary

This packet requests review for a corrective Task 111h code checkpoint:

- `728cc2ed9ee2b14e14b667c521390d04f3880526` `task111h: make turboquant rerank sidecars centroid-relative`

Packet 022 made RaBitQ4/RaBitQ8 index-side rerank payloads residual against the
assigned IVF centroid. This checkpoint applies the same index-side sidecar
semantics to TurboQuant through the shared rerank payload codec:

- source-diagnostic TurboQuant remains whole-vector query-time conversion;
- index-side TurboQuant now rejects centroid-less sidecar encoding;
- build/insert encode `source - assigned_centroid`;
- scan scores the persisted payload estimate and adds back the selected-list
  centroid inner product through the existing centroid-IP correction path.

In distance form, quantized index-side rerank now uses:

```text
final distance = -(payload_estimate + centroid_ip)
```

## Format Version

The payload byte length and packed group layout are unchanged, but the persisted
TurboQuant payload semantics changed. The checkpoint bumps IVF from v6 to v7,
adds `fixtures/on-disk/ivf_metadata_v7.hex`, keeps v6 as rejected legacy, and
updates the upgrade matrix/docs so old whole-vector TurboQuant sidecar bytes are
not silently interpreted as centroid-relative payloads.

## Validation

Packet-local logs:

- `artifacts/cargo-test-centroid-relative-sidecar.log`
- `artifacts/cargo-test-payload-codecs.log`
- `artifacts/cargo-test-ivf-metadata.log`
- `artifacts/cargo-test-upgrade-matrix.log`
- `artifacts/git-diff-check.log`

Result summary:

```text
index_side_quantized_payloads_require_centroid_and_apply_correction: 1 passed
payload_codecs: 2 passed
ivf_metadata_: 6 passed
upgrade_matrix: 2 passed
git diff --check: exit 0
```

See `artifacts/manifest.md` for commands and exact result lines.

## Review Ask

Please review whether TurboQuant should use this centroid-relative sidecar
semantics for index placement, and whether the implementation correctly reuses
the shared build/insert/scan codec path rather than adding a format-specific
one-off.

Please also review whether the v7 format bump, fixture update, docs, and
upgrade matrix handling are sufficient for the semantic incompatibility with
v6 TurboQuant sidecar payloads.

## Non-Claims

This packet does not claim new recall, latency, or storage results. The older
Task 111h TurboQuant benchmark numbers were gathered before this fix and should
not be treated as final evidence for the compressed rerank design.

Together with packet 022, this makes the next benchmark rerun a post-fix
RaBitQ4/RaBitQ8/TurboQuant sweep, not a reinterpretation of the old data.
