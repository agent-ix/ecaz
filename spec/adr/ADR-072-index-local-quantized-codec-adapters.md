---
id: ADR-072
title: "Index-Local Quantized Codec Adapters"
status: PROPOSED
impact: Extends ADR-071. Affects future HNSW RaBitQ work, DiskANN storage formats, IVF quantizer-profile maintenance, FR-034, FR-035, FR-038, and ADR-041/ADR-033 adapter boundaries.
date: 2026-05-26
---
# ADR-072: Index-Local Quantized Codec Adapters

## Context

ADR-071 sets the target direction for a unified quantizer interface across
HNSW, DiskANN, and IVF. Task 60's DiskANN RaBitQ integration provides more
evidence about where the shared boundary should sit.

The reusable part is the quantizer family:

- deterministic parameters such as dimension, seed, and bit width;
- optional training or model preparation;
- vector-to-code encoding;
- query-side prepared scorer construction;
- approximate scoring over code bytes;
- family metadata such as code length and wire-format identity.

The AM-specific part is the codec adapter that binds those code bytes into an
index's physical and traversal surface:

- DiskANN stores direct or grouped search bytes in `VamanaNodeTuple`, tags
  `VamanaMetadataPage`, controls sidecar flags, and must preserve on-demand
  `GraphReader` traversal.
- HNSW stores graph payloads behind graph storage descriptors, hot/cold/rerank
  tuple layouts, insert/vacuum scoring hooks, and compatibility rules inherited
  from older TurboQuant and PqFastScan formats.
- IVF stores posting-list payloads and list-local scan state, with less graph
  lifecycle coupling but its own list metadata and profile semantics.

Task 60 made DiskANN look "codec-shaped" through local structures like
`DiskannBuildCodec` and `DiskannPreparedPrefilter`, but that is not the same as
a ready shared trait. HNSW RaBitQ is expected to expose a different and more
entangled storage surface.

## Decision

Adopt **index-local quantized codec adapters** as the near-term pattern.

The architecture has two layers:

```text
shared quantizer family
  owns quantization math and score semantics

index-local codec adapter
  owns metadata, tuple/list layout, sidecars, traversal binding, and AM
  compatibility rules
```

The shared quantizer layer must not know about `VamanaMetadataPage`, HNSW graph
tuples, IVF posting-list pages, PostgreSQL relation handles, WAL posture, or
AM scan state. It may expose code bytes, code length, prepared scorer state,
and family metadata.

The index-local codec adapter must not fork quantizer math. It may own only
AM binding logic:

- reloption/profile mapping into the selected quantizer family;
- build-time payload preparation;
- insert/update payload preparation;
- metadata discriminator fields and compatibility checks;
- tuple/list payload sizing and encoding;
- scan-time prepared scorer selection;
- score polarity normalization for that AM's traversal;
- sidecar, codebook, rerank, or direct-code layout decisions.

In concrete terms, the desired shape is:

```text
RaBitQQuantizer / PqFastScanQuantizer / TurboQuantQuantizer
        |
        v
DiskANN codec adapter / HNSW codec adapter / IVF codec adapter
        |
        v
AM build, insert, scan, vacuum, metadata, and benchmark reporting
```

The adapter may start as concrete enums and helper structs inside each AM. It
does not need to be a trait until the repeated shape is proven by at least two
AMs.

## Current Guidance

### DiskANN

DiskANN may continue with a local codec-shaped adapter. Task 60's
`DiskannBuildCodec` and `DiskannPreparedPrefilter` are acceptable near-term
forms because they keep the critical DiskANN invariants explicit:

- RaBitQ has an explicit metadata discriminator;
- `pq_fastscan` stays backward compatible;
- RaBitQ stores direct search-code bytes and no binary sidecar;
- scan-time reads stay on-demand through `GraphReader`.

### HNSW

HNSW RaBitQ should not begin with a cross-AM abstraction. Instead, it should
extract a narrow HNSW-local codec adapter while integrating RaBitQ.

That adapter should isolate the already-entangled HNSW concerns:

- storage-format reloption mapping;
- graph storage descriptor selection;
- hot/cold/rerank tuple layout;
- build and insert payload encoding;
- insert/vacuum/scan approximate scorer construction;
- metadata compatibility and legacy format validation.

The existing HNSW graph lifecycle adapter direction in ADR-033 remains useful:
shared graph topology lifecycle should stay separate from format-specific
payload, storage, and scoring hooks.

### IVF

IVF is already format-pluggable enough to support RaBitQ, but that does not
mean it has a reusable codec abstraction. IVF should keep its list-local adapter
surface and only adopt shared traits when the DiskANN/HNSW/IVF shapes converge.

## Extraction Trigger

Promote index-local codec adapters into a shared trait only after the HNSW
RaBitQ work proves the repeated shape.

Good triggers:

1. DiskANN and HNSW have equivalent local methods for build encode, insert
   encode, query scorer preparation, metadata family identity, and code length.
2. IVF needs to consume the same quantizer-family metadata object to avoid
   reloption/reporting drift.
3. A new quantizer family would otherwise require duplicating the same adapter
   boilerplate across at least two AMs.
4. Benchmark rows need a shared quantizer/storage identity object beyond the
   current `storage_format`, `cache_state`, and host-parity fields.

The first extraction should be small: move the stable shared quantizer-family
contract first, then optionally lift a narrow codec trait. Do not rewrite all
AMs in one PR.

## Consequences

- **Clearer near-term HNSW work.** HNSW can be disentangled enough to add
  RaBitQ without freezing an unproven cross-AM API.
- **DiskANN remains a useful prototype.** Its Task 60 codec-shaped structures
  become evidence for the eventual shared shape, not a one-off workaround.
- **IVF is not overfit to graph assumptions.** Posting-list concerns stay local
  until there is a real common contract.
- **On-disk compatibility stays AM-owned.** The common quantizer layer does not
  decide tuple/page layout or migration behavior.
- **Some duplication remains intentional.** Until HNSW and DiskANN prove a
  repeated shape, local adapters are cheaper than premature generality.

## Alternatives Considered

### Extract a shared codec trait before HNSW RaBitQ

Rejected. DiskANN and IVF alone do not expose enough of HNSW's graph storage,
insert/vacuum, and rerank-layout constraints. A trait extracted now would
likely encode DiskANN assumptions and make HNSW harder, not cleaner.

### Keep only ADR-071 and rely on local judgment

Rejected. ADR-071 correctly names the shared quantizer direction, but Task 60
clarified that each AM needs an explicit codec adapter boundary between shared
math and local storage/traversal. Without this ADR, future HNSW work could
either over-extract too early or continue entangling payload and graph logic.

### Put tuple/list layout into the quantizer family

Rejected, consistent with ADR-071. RaBitQ, PqFastScan, and TurboQuant should
not know how DiskANN, HNSW, or IVF lay out pages. The quantizer family owns
code and scoring semantics; the AM codec adapter owns persistence.

## Related Decisions

- ADR-033: Shared graph lifecycle with format-specific insert/vacuum adapters.
- ADR-041: Module structure for multi-AM, multi-quantizer growth.
- ADR-045: Page-layout discipline for graph access methods.
- ADR-048: IVF access method.
- ADR-070: On-disk forward-compat encoding convention.
- ADR-071: Unified quantizer interface across access methods.
