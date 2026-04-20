---
id: ADR-032
title: "Coexisting Index Formats: TurboQuant and PqFastScan as First-Class Peers"
status: PROPOSED
impact: Affects FR-014, NFR-001, ADR-006, ADR-026, ADR-027, ADR-030, ADR-031
date: 2026-04-16
---
# ADR-032: Coexisting Index Formats — TurboQuant and PqFastScan

## Context

tqvector started with a single index format built on the TurboQuant quantizer
(ADR-006): SRHT rotation + per-dimension Lloyd-Max MSE + optional 1-bit QJL
residual. That format — internally `GraphStorageDescriptor::ScalarV1`, wire
tag `INDEX_FORMAT_V1_SCALAR` — is still the default and the only format that
supports online insert and vacuum today.

Packets 278–333 on the `adr030-v2-*` branch line explored an alternative
format: SRHT rotation + grouped PQ4 search codes + FastScan-style SIMD
scorer, with an optional RaBitQ-style binary sidecar (ADR-031 lineage) and
a cold rerank payload kept separate from the hot traversal tuple (ADR-030
2026-04-13 design checkpoint). That format — internally
`GraphStorageDescriptor::GroupedV2`, wire tag `INDEX_FORMAT_V2_GROUPED` —
has narrowed query latency meaningfully relative to TurboQuant on the 50k
warm real seam and is now close enough to NFR-001 to be worth shipping.

The open question is *how* to land it. The feasibility branch gates the
new format behind a `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD` env var
(`src/am/build.rs:1102`), rejects inserts
(`src/am/insert.rs:186`, `ADR030_GROUPED_V2_INSERT_UNSUPPORTED`), rejects
vacuum (`src/am/vacuum.rs:107`, `ADR030_GROUPED_V2_VACUUM_UNSUPPORTED`), and
hard-codes `group_size = 16` and `bits = 4`. That posture is fine for
benchmarks and wrong for `main`.

## Decision

### Both formats are first-class, peer-selectable index layouts

tqvector will carry **two** first-class index formats in `main`:

- **TurboQuant** — renamed from `ScalarV1`. SRHT + per-dimension MSE ± QJL.
  Remains the default on this merge for backwards compatibility.
- **PqFastScan** — renamed from `GroupedV2`. SRHT + grouped PQ4 + FastScan
  scoring, with hot/cold payload split and optional binary sidecar.

The on-disk wire tags (`INDEX_FORMAT_V1_SCALAR` / `INDEX_FORMAT_V2_GROUPED`)
are not renamed. Versioning bytes serve wire compatibility, not human
identification. The Rust discriminator enum is renamed to match:

```rust
pub(crate) enum GraphStorageDescriptor {
    TurboQuant { code_len: usize },                // was ScalarV1
    PqFastScan(PqFastScanLayout),                  // was GroupedV2(GroupedGraphLayout)
}
```

Format selection moves from a process-wide env var to a per-index reloption:

```sql
CREATE INDEX ... USING ec_hnsw (embedding vector_ip_ops)
    WITH (storage_format = 'pq_fastscan');
```

`storage_format` accepts `'turboquant'` (default) and `'pq_fastscan'`.

### No experimental gate in main

PqFastScan ships with parity on all AM entrypoints:

- Build — already implemented on the feasibility branch.
- Scan — already implemented.
- **Insert** — currently hard-errors. Must implement before merge. Re-encoding
  a new tuple into the existing learned subvector codebooks is straightforward;
  the lock-ordering rules from ADR-026 apply unchanged.
- **Vacuum** — currently hard-errors. Must implement before merge. Applies
  ADR-027 lock ordering; must handle both hot and cold payload pages.

Hard-coded build parameters (`ADR030_EXPERIMENTAL_GROUP_SIZE = 16`,
`bits = 4`) become metadata-driven values on the `PqFastScanLayout`,
plumbed via the reloption or derived from dimension.

### Why not land PqFastScan as experimental

An experimental gate is tempting because it protects users from partial
support. But carrying two formats where only one is insertable and
vacuumable creates a subtle footgun: any user who tries PqFastScan for a
real workload discovers the gap at runtime rather than at CREATE INDEX.
The cost of finishing insert/vacuum before merge is bounded and
well-understood; the cost of shipping a silently-crippled second format
is unbounded. tqvector has no experimental-format precedent in `main` and
shouldn't set one now.

### Sequencing relative to TurboQuant speedups

Landing PqFastScan cleanly is a prerequisite to the follow-on work of
porting its architectural wins back onto TurboQuant. Three building
blocks transfer cleanly to the TurboQuant path:

1. **Binary prefilter → TurboQuant rerank** (ADR-031 sidecar in front of
   the TurboQuant LUT scorer).
2. **Heap-f32 rerank mode** lets traversal run on the smaller 4+0
   TurboQuant_mse payload (no QJL) with exact rerank from the heap —
   ADR-025 §Mitigation 3 sketched this but never shipped it.
3. **Hot/cold payload split** — keep QJL bits and gamma out of the
   scoring-hot bytes read per graph edge.

That iteration is tracked as task 16 and is intentionally not bundled
with the PqFastScan landing in task 15, so the TurboQuant speedup work
does not block or ride on PqFastScan polish.

## Consequences

### Structural work required before merge

Enumerated in task 15. Summary:

- Rust enum and function rename across the am/quant modules.
- New `storage_format` reloption in `src/am/options.rs`.
- Remove the `TQVECTOR_EXPERIMENTAL_ADR030_V2_BUILD` env var.
- `ec_hnsw_aminsert` parity for PqFastScan.
- `ec_hnsw_ambulkdelete` / `ec_hnsw_amvacuumcleanup` parity.
- Parameterize `group_size` / `bits` through the layout rather than
  module-level constants.

### Scan dispatch shape going forward

`src/am/scan.rs` is 6545 lines with 18 `GraphStorageDescriptor` match
sites today. Tolerable for two formats, brittle for three. Extracting a
`ScoringStrategy` trait is **advisory, not blocking** — it becomes
blocking if and when a third format is proposed.

### Defaults

The default format on this merge is **TurboQuant**. Flipping the default
to PqFastScan is a separate decision deferred until:

- PqFastScan insert throughput is measured on the 1k-insert harness.
- PqFastScan vacuum is validated on a deleted-tuple churn run.
- Real-corpus recall across the 1536/1024/2048 tiers is confirmed to
  match or beat TurboQuant at equivalent payload.

### Version-skew posture

Existing `INDEX_FORMAT_V1_SCALAR` indexes keep working unchanged.
PqFastScan requires REINDEX to adopt; there is no v1→v2 auto-upgrade and
none is planned. The ADR-030 v2 checkpoint already committed to
rebuild-only.

## References

- ADR-006: Own quantizer implementation based on TurboQuantDB
- ADR-025: Bit allocation — MSE vs QJL (4+0 fast path, heap-rerank sketch)
- ADR-026: Live insert backlink lock ordering
- ADR-027: Vacuum graph repair lock ordering
- ADR-030: FastScan Grouped Subvector Scoring — v2 design checkpoint
- ADR-031: RaBitQ Binary Pre-Filter for Beam Search Candidate Scoring
- Task 14: ADR-030 v2 grouped-index implementation track (feasibility to this ADR)
- Task 15: Land PqFastScan as first-class (this ADR's execution vehicle)
- Task 16: TurboQuant iteration with PqFastScan learnings
