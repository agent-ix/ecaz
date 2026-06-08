---
id: ADR-076
title: "TurboQuant TQ+ Format and Validation"
status: PROPOSED
impact: Governs Task 89 TQ+ re-landing across IVF, SPIRE, HNSW, and DiskANN. Affects ADR-006, ADR-032, ADR-048, ADR-070, ADR-071, ADR-072, FR-013, FR-034, FR-035, FR-038, and all AM storage-format reloption surfaces.
date: 2026-06-08
---
# ADR-076: TurboQuant TQ+ Format and Validation

## Context

Task 86 proved a narrow TurboQuant+ calibration profile on IVF and DBPedia,
but that work was intentionally slimmed out before merge. The reverted Task 86
implementation used a new operator-visible IVF storage format:

```text
storage_format = 'turboquant_tqplus'
StorageFormat::TurboQuantTqPlus = 4
```

The measurement profile kept hot posting bytes identical to TurboQuant no-QJL
4-bit codes, reused posting gamma as the per-vector TQ+ renormalization scalar,
and stored two float-vector calibration arrays (`shift[dim]`, `scale[dim]`) in
the existing IVF PQ codebook tuple chain.

That was acceptable as an isolated measurement tag, but Task 89 asks whether
TQ+ should become a production feature across IVF, SPIRE, HNSW, and DiskANN.
The production decision must avoid turning a narrow experiment into four
unrelated on-disk formats.

## Decision

**TQ+ is a TurboQuant calibration profile, not a new top-level quantizer family.**

Production Task 89 ports SHALL expose TQ+ as a profile layered under the
existing `turboquant` storage family:

```text
storage_format = 'turboquant'
turboquant_profile = 'standard' | 'tqplus'
```

`turboquant_profile = 'standard'` preserves current behavior and remains the
default. `turboquant_profile = 'tqplus'` is valid only for the no-QJL 4-bit
TurboQuant lane until a later task validates other bit/QJL configurations.

The old Task 86 measurement spelling, `storage_format = 'turboquant_tqplus'`,
SHALL NOT become the preferred production API. If the Task 86 packet-011
fixtures need to stay readable, tag `4` may be reintroduced as a read-only
legacy decoder that maps to `turboquant_profile = 'tqplus'` for that IVF
surface only. New writes SHALL use the profile form.

This decision intentionally keeps the stable family identity (`turboquant`)
separate from the calibration variant (`standard` vs. `tqplus`).

## Format Model

Every AM owns its tuple/page layout, consistent with ADR-071 and ADR-072, but
all AMs SHALL model TQ+ with the same logical fields:

- `profile`: `standard` or `tqplus`;
- `calibration_dim`: dimension of the calibration arrays;
- `shift[calibration_dim]`: finite f32 per coordinate;
- `scale[calibration_dim]`: finite non-zero f32 per coordinate;
- deterministic build seed/sample identity sufficient to reproduce the model;
- a metadata flag or version field proving the calibration is present before
  a TQ+ scan or insert attempts to score/encode.

Hot per-row payloads remain the TurboQuant no-QJL 4-bit code bytes. Any
per-vector renormalization scalar may reuse an existing hot scalar field only
where the AM already has one with identical lifetime and scan availability; the
AM-specific port must document that binding.

## Compatibility

Existing `turboquant` indexes read as `turboquant_profile = 'standard'`.

Existing `pq_fastscan` and `rabitq` indexes are unaffected.

Pre-Task-89 binaries presented with a new TQ+ page/metadata version may reject
the index as an unknown version per ADR-070 Option A. They must not reinterpret
TQ+ bytes as standard TurboQuant.

Task-89 binaries presented with Task 86 IVF tag `4`, if that legacy decoder is
implemented, SHALL validate the same constraints as a production TQ+ index:
calibration head present, exactly one shift and one scale array, matching
dimensions, finite values, and non-zero scales. If any condition fails, reads
SHALL ERROR before scoring.

## Operator Surface

New DDL should use:

```sql
WITH (
  storage_format = 'turboquant',
  turboquant_profile = 'tqplus'
)
```

The profile reloption is AM-scoped and must appear consistently on IVF, SPIRE,
HNSW, and DiskANN before Task 89 can claim all-index support.

If accepted for compatibility, `storage_format = 'turboquant_tqplus'` is a
deprecated IVF-only alias for legacy fixture reads and explicit migration tests.
It must not appear in user-facing examples as the preferred syntax.

## Calibration Storage

Task 89 SHALL NOT reuse the name `pq_codebook` for new TQ+ production metadata.
The Task 86 IVF codebook-chain reuse was measurement-local and is not the
long-term abstraction.

Each AM may reuse an existing WAL-covered float-array tuple/page primitive only
if the public metadata and diagnostic labels call it TurboQuant calibration,
not PQ codebook storage. If no suitable primitive exists, the AM must add a
narrow TQ+ calibration record owned by that AM.

For graph AMs, calibration loading should be relation/index scoped and cached
for the scan or insert operation. It should not be fetched per candidate during
greedy traversal.

## Measurement Methodology

Task 89 evidence SHALL use `ecaz bench suite` per FR-038. For each AM port,
commit paired suite configs or a single config with baseline/change columns:

- baseline: current `storage_format = 'turboquant'`, profile `standard`;
- change: `storage_format = 'turboquant'`, profile `tqplus`;
- corpus: DBPedia real 10k, 50k, and 100k for per-AM validation;
- metrics: recall@10, p50/p95/p99 latency, storage, and metadata overhead;
- isolation: one-index-per-table unless the packet explicitly documents a
  shared-table reason;
- determinism: fixed seed and a golden rebuild check for bit-identical index
  metadata/calibration pages.

Cross-corpus validation SHALL run IVF, SPIRE, HNSW, and DiskANN on at least one
non-DBPedia embedding distribution at 10k or 50k.

Streaming-insert drift validation SHALL compare post-insert TQ+ indexes against
full-rebuild TQ+ baselines at 10%, 25%, and 50% inserted rows. The acceptance
threshold is:

- recall@10 delta <= 0.5 percentage points at 25% inserted rows;
- recall@10 delta <= 1.0 percentage point at 50% inserted rows.

The 10% row is diagnostic and should pass the 0.5 pp threshold unless the
packet explicitly shows why the distribution is intentionally adversarial.

## Consequences

- TQ+ does not consume a new top-level storage-family identity on every AM.
- Existing `storage_format = 'turboquant'` semantics stay stable.
- The Task 86 tag-4 implementation remains valuable measurement history, but
  it is not the production API shape.
- Each AM still owns its physical layout, which avoids a premature cross-AM
  codec trait while forcing consistent reloptions and validation gates.
- Promotion remains evidence-gated. This ADR chooses the shape to validate; it
  does not by itself promote TQ+.

## Alternatives Considered

### Separate `turboquant_tqplus` Storage Format Everywhere

Rejected for production. It matched the Task 86 measurement slice and is easy
to gate, but it would make a calibration variant look like a separate quantizer
family across four AMs. That increases operator-visible surface area and makes
future TurboQuant profile growth expensive.

### Hidden Flag on `storage_format = 'turboquant'`

Rejected as underspecified. A hidden metadata bit would minimize DDL churn, but
operators and benchmark packets need a stable way to request and report the
variant. TQ+ changes recall/latency behavior enough that it must be explicit.

### Dedicated Shared Calibration Page Primitive First

Deferred. A shared primitive may be warranted after two AM ports prove the same
layout pressure, but Task 89 should first keep AM-owned layout decisions local.
The shared math and validation rules are common; persistence remains local.

## Verification

Task 89 reviewers should reject a port unless it proves:

1. `turboquant_profile = 'standard'` preserves existing TurboQuant behavior.
2. `turboquant_profile = 'tqplus'` rejects unsupported bit/QJL lanes.
3. Missing, wrong-dimension, non-finite, or zero-scale calibration metadata
   fails before scan scoring or insert encoding.
4. The AM's persisted metadata cannot be mistaken for standard TurboQuant by a
   Task-89 reader.
5. Per-AM suite evidence covers DBPedia 10k/50k/100k.
6. Cross-corpus and streaming-insert drift packets exist before closeout.

## Related Decisions

- ADR-006: Own TurboQuant implementation.
- ADR-032: Coexisting index formats.
- ADR-048: IVF access method.
- ADR-070: On-disk forward-compat encoding convention.
- ADR-071: Unified quantizer interface across access methods.
- ADR-072: Index-local quantized codec adapters.
- Task 89: TurboQuant TQ+ cross-AM and cross-corpus validation.
