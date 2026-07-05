# Review Request: Phase 5C-2 — Pure-Rust Build Orchestrator

Branch: `adr034-diskann-access-method`
Author: coder-2
Companion to: 11014 (ADR-045), 11015 (Phase 5A), 11016 (Phase 5B),
11017 (Phase 5C-1)

## What this slice is

Second sub-slice of Phase 5C: pure-Rust glue that ties Phase 5A's
algorithm core (`build_vamana_graph`, `approximate_medoid`) to Phase
5C-1's persistence sequencer (`persist_vamana_graph`) and assembles
the populated `VamanaMetadataPage`. No pgrx, no quantizer training —
the orchestrator is the API surface that Phase 5C-3's pgrx ambuild
callback will call once it has heap-scanned + encoded the rows.

## Scope

- `src/am/diskann/build.rs` — new file, 374 lines incl. 8 tests.
- `src/am/diskann/mod.rs` — `pub mod build;` declaration.

No other source files touched.

## What changed

### Public API

```rust
pub const MEDOID_SAMPLE_CAP: usize = 1000;

pub struct BuildParams {
    pub graph_degree_r: u16,
    pub build_list_size_l: u16,
    pub alpha: f32,
    pub dimensions: u16,
    pub search_subvector_count: u16,
    pub search_subvector_dim: u16,
    pub seed: u64,
    pub page_size: usize,
    pub has_binary_sidecar: bool,
}

impl BuildParams {
    pub fn binary_word_count(&self) -> usize;  // W = dim.div_ceil(64) or 0
    pub fn search_code_len(&self) -> usize;    // C = M.div_ceil(2)
    pub fn payload_flags(&self) -> u8;
}

pub struct BuildOutput {
    pub metadata: VamanaMetadataPage,
    pub persisted: PersistedGraph,
}

pub fn build_and_persist_vamana<D: Fn(u32, u32) -> f32 + Copy>(
    params: BuildParams,
    payloads: &[NodePayload],
    build_dist: D,
) -> Result<BuildOutput, String>;
```

### What it does

1. Validates `BuildParams` (R/L > 0, finite alpha ≥ 1, dimensions > 0,
   non-empty payloads).
2. `approximate_medoid(N, MEDOID_SAMPLE_CAP=1000, seed, build_dist)`.
3. `build_vamana_graph(N, medoid, R, L, alpha, seed, build_dist)`.
4. Derives `(W, C)` from `BuildParams` per the ADR-045 reference
   layout rules and calls `persist_vamana_graph(...)` from 5C-1.
5. Assembles a `VamanaMetadataPage` with:
   - `format_version = INDEX_FORMAT_V3_DISKANN`
   - `entry_point = persisted.entry_point_tid` (medoid TID)
   - `transform_kind = SRHT`, `search_codec_kind = GROUPED_PQ`
   - `payload_flags` derived from `has_binary_sidecar` (always
     includes grouped + cold-rerank per ADR-044's current default)
   - `grouped_codebook_head = INVALID` — Phase 5C-3 patches this
     after writing the codebook chain
6. Returns `BuildOutput { metadata, persisted }`. The pgrx caller
   writes both into the relation under one GenericXLog transaction.

### Tests (8, all green)

- **BO-001** empty payloads errors
- **BO-002** zero `graph_degree_r` errors
- **BO-003** alpha < 1.0 errors
- **BO-004** `(W, C)` derivation: W=dim/64 with sidecar on, 0 off;
  C=M.div_ceil(2). Locks the ADR-045 derivation rules at the API
  boundary.
- **BO-005** end-to-end on 64 synthetic 2D L2 points: every metadata
  field populated correctly, every node has a valid TID,
  entry_point ≠ INVALID
- **BO-006** payload shape mismatch surfaces from the persist layer
  (proves the W/C contract is enforced through the orchestrator)
- **BO-007** deterministic — same seed + same dist + same payloads ⇒
  bit-equal `entry_point`, `node_to_tid`, and `persistence_order`
- **BO-008** every persisted tuple decodes with the metadata-derived
  `(R, W, C)` triple and round-trips its primary_heaptid /
  binary_words / search_code; entry_point maps back to a valid node id

```
running 8 tests
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured;
             555 filtered out; finished in 0.01s
```

`cargo check --lib` clean (5 pre-existing dead-code warnings).
Full diskann module: 44 tests pass (6 page + 13 tuple + 6 vamana +
11 persist + 8 build).

## Review focus

1. **`(W, C)` derivation lives in `BuildParams`, not in the metadata
   page.** Phase 5B request packet (11016) §Review focus 2 asked
   reviewer to confirm these rules; they are now codified in
   `BuildParams::binary_word_count()` and `search_code_len()` and
   tested via BO-004. Reviewer confirm the rules:
   - `W = if has_binary_sidecar { dimensions.div_ceil(64) } else { 0 }`
   - `C = search_subvector_count.div_ceil(2)` (PQ4 nibbles)
2. **`grouped_codebook_head = INVALID` at orchestrator output.** The
   codebook is sizable (`GROUPED_PQ_CENTROIDS × subvector_dim` per
   subvector) and lives in its own page chain. The pgrx caller
   (Phase 5C-3) writes the codebook chain and patches the metadata
   page in the same GenericXLog transaction. Reviewer confirm this
   split (orchestrator owns metadata-from-graph; codebook ownership
   belongs to the pgrx wiring).
3. **`MEDOID_SAMPLE_CAP = 1000`.** Matches pgvectorscale; makes
   medoid cost O(S²)=10⁶ distance calls regardless of N. For N <
   1000 the medoid is exact. Reviewer call: keep at 1000, expose as
   reloption, or scale with N?
4. **`payload_flags()` always sets `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD`.**
   This matches ADR-044's current default ("rerank from heap via
   ecvector EXTERNAL"). When ADR-044's C1 reopen flips the default,
   this helper is the single edit-site. Reviewer flag if a cleaner
   home is preferred (e.g., a separate ADR-044 module).
5. **Build distance is a `Fn(u32, u32) -> f32 + Copy` closure.** Same
   shape as Phase 5A's `build_vamana_graph` parameter so no
   adaptation is needed at the orchestrator boundary. Reviewer
   confirm carrying the closure through unchanged is the right call.

## Questions to answer

- **Should `BuildParams` carry `medoid_sample_cap` as a field?**
  Currently a module const. Argument for: makes determinism testing
  easier (vary cap, prove medoid changes). Argument against: not a
  reloption, no shipping reason to vary it. Held: const for now.
- **Should the orchestrator log unreachable-node count when
  `persisted.unreached` is non-empty?** It's a soft-warning condition
  per Phase 5C-1's design. The pgrx caller has the right `pgrx::warning!`
  context; the pure-Rust orchestrator just surfaces it via
  `BuildOutput.persisted.unreached`. Held: report, don't log.
- **Should the orchestrator do a final connectivity check
  (BFS-from-entry-point reaches a configurable fraction)?** The
  fraction is policy and belongs upstream; the orchestrator's
  `unreached` count is the raw signal. Held: defer to caller.

## Not doing in this packet

- **pgrx ambuild callback.** Phase 5C-3 (next): `ambuild` /
  `ambuildempty`, heap-scan plumbing, per-row SRHT/PQ encode,
  driving `build_and_persist_vamana`, codebook chain, GenericXLog
  block-zero metadata write.
- **Quantizer training.** Lives in Phase 5C-3 alongside the heap scan.
- **Live insert path.** Phase 7.

## Dependencies

- **ADR-045 ACCEPTED** — derivation rules for `(W, C)` and the
  metadata-page-as-single-source-of-truth invariant are encoded here.
- **Phase 5A (11015)** — uses `approximate_medoid` and
  `build_vamana_graph`.
- **Phase 5C-1 (11017)** — uses `persist_vamana_graph`, `NodePayload`,
  `PersistedGraph`.
- **`am::diskann::page`** — uses `VamanaMetadataPage` constructor
  fields, format/transform/codec/payload-flag constants.

## Companion packets

- **11014** — ADR-045 page-layout discipline.
- **11015** — Phase 5A vamana algorithm core.
- **11016** — Phase 5B slim tuple.
- **11017** — Phase 5C-1 persistence sequencer.
- **11019** — Phase 5C-3 pgrx ambuild + quantizer wiring (future).

## Definition of ready

- ADR-045 ACCEPTED.
- 8 BO tests green (verified locally).
- Reviewer confirms the `(W, C)` derivation rules and the
  metadata-page split between orchestrator and pgrx caller.
- Phase 5C-3 does not start before this lands.

## Handoff notes

The orchestrator is intentionally trivial — about 50 lines of
non-test logic. Its purpose is to:

1. Lock the `(W, C)` derivation rules at one site (`BuildParams`).
2. Define the API the pgrx-side caller will use, so Phase 5C-3 can
   focus entirely on the pgrx mechanics (heap scan, training,
   GenericXLog) without re-deciding "what is a build?"
3. Make the build pipeline testable end-to-end with a synthetic L2
   distance — no quantizer required.

If reviewer pushes back on `MEDOID_SAMPLE_CAP` or `payload_flags()`
defaults, the change is one line each. If the `(W, C)` derivation
rules need to flex for future codecs, the right move is to make
`BuildParams` an enum over codec families rather than to weaken the
derivation site.
