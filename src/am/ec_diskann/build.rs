//! Pure-Rust build orchestrator for `ec_diskann` (task 17 Phase 5C-2).
//!
//! Glue layer that ties Phase 5A's algorithm core ([`build_vamana_graph`])
//! and Phase 5C-1's persistence sequencer ([`persist_vamana_graph`])
//! together with a populated [`VamanaMetadataPage`].
//!
//! Inputs are codec-opaque: the orchestrator sees [`NodePayload`]s
//! whose `binary_words` and `search_code` byte-runs were produced by
//! the caller (Phase 5C-3 will be the pgrx-side caller that runs the
//! SRHT transform and grouped-PQ encoder before invoking this
//! function). The build distance is also caller-supplied, typically
//! bound to `score_ip_codes_lite` over the same encoded codes.
//!
//! Outputs:
//!   - [`PersistedGraph`] — the encoded `DataPageChain` + the
//!     node-id ↔ TID map.
//!   - [`VamanaMetadataPage`] — populated with `entry_point` (the
//!     medoid TID), `dimensions`, `search_subvector_count` /
//!     `_dim`, and the right `payload_flags`. The pgrx caller writes
//!     this to block 0 under the same GenericXLog as the chain.
//!
//! What this module does NOT do (lives in Phase 5C-3):
//!   - Heap scan / pgrx callbacks.
//!   - Quantizer training (codebook fit, SRHT seed, grouped-PQ k-means).
//!   - GenericXLog wrapping and block-zero metadata write.
//!   - `grouped_codebook_head` chain (codebook persistence).

use crate::am::ec_diskann::page::{
    VamanaMetadataPage, INDEX_FORMAT_V3_DISKANN, PAYLOAD_FLAG_BINARY_SIDECAR,
    PAYLOAD_FLAG_GROUPED_SEARCH_CODE, VAMANA_SEARCH_CODEC_GROUPED_PQ,
    VAMANA_TRANSFORM_KIND_SRHT,
};
use crate::am::ec_diskann::persist::{persist_vamana_graph, NodePayload, PersistedGraph};
use crate::am::ec_diskann::vamana::{approximate_medoid, build_vamana_graph};
use crate::storage::page::ItemPointer;

/// Capped sample size for the medoid approximation. Mirrors the
/// pgvectorscale heuristic; large enough to be representative for
/// 10k–10M point graphs, small enough to keep medoid cost O(S²)
/// negligible relative to graph build cost.
pub const MEDOID_SAMPLE_CAP: usize = 1000;

/// Caller-supplied parameters for one Vamana index build. These are
/// the per-index constants from the relation's reloptions plus the
/// quantizer shape (set after training).
#[derive(Debug, Clone, Copy)]
pub struct BuildParams {
    /// Vamana max degree `R`. Reloption `graph_degree_r`.
    pub graph_degree_r: u16,
    /// Vamana build search-list size `L`. Reloption `build_list_size_l`.
    pub build_list_size_l: u16,
    /// Robust-prune α for the second pass. Reloption `alpha`.
    pub alpha: f32,
    /// Vector dimensionality. Used for `W = dimensions.div_ceil(64)`
    /// when the binary sidecar flag is on.
    pub dimensions: u16,
    /// Number of grouped-PQ subvectors (`M`). `C = M.div_ceil(2)` for
    /// PQ4 nibble packing.
    pub search_subvector_count: u16,
    /// Width (in source dims) of each grouped-PQ subvector.
    pub search_subvector_dim: u16,
    /// Deterministic seed for medoid sample + insertion-order shuffle.
    pub seed: u64,
    /// PostgreSQL page size (`BLCKSZ`). Threaded so non-pgrx tests can
    /// pick a small size to force chain spill.
    pub page_size: usize,
    /// True iff the index keeps a per-node binary sidecar (W>0).
    pub has_binary_sidecar: bool,
}

impl BuildParams {
    /// `W = dimensions.div_ceil(64)` when the binary sidecar is on, 0
    /// otherwise. Derivation rule from ADR-045 reference layout.
    pub fn binary_word_count(&self) -> usize {
        if self.has_binary_sidecar {
            (self.dimensions as usize).div_ceil(64)
        } else {
            0
        }
    }

    /// `C = M.div_ceil(2)` for PQ4 nibble packing. Derivation rule
    /// from ADR-045 reference layout.
    pub fn search_code_len(&self) -> usize {
        (self.search_subvector_count as usize).div_ceil(2)
    }

    /// Initial `payload_flags` byte for the metadata page.
    ///
    /// V0 reranks from the heap `ecvector` row (ADR-044 default) and
    /// writes no index-side cold payload chain. Per ADR-046 frozen
    /// rule 1 and ADR-047 frozen rule 4, `PAYLOAD_FLAG_COLD_RERANK_PAYLOAD`
    /// must stay clear — setting it would advertise a cold chain that
    /// does not exist and trip the scan path's rerank wiring. A future
    /// ADR-044 C1 reopen is the only path that sets it (packet 11018).
    pub fn payload_flags(&self) -> u8 {
        let mut flags = PAYLOAD_FLAG_GROUPED_SEARCH_CODE;
        if self.has_binary_sidecar {
            flags |= PAYLOAD_FLAG_BINARY_SIDECAR;
        }
        flags
    }
}

/// Result of one end-to-end build: the page chain plus the populated
/// metadata page. The pgrx caller (Phase 5C-3) is responsible for
/// writing both into the relation under a single GenericXLog
/// transaction.
#[derive(Debug, Clone)]
pub struct BuildOutput {
    pub metadata: VamanaMetadataPage,
    pub persisted: PersistedGraph,
}

/// Drive medoid → build → persist → metadata-page assembly.
///
/// Caller contract:
/// - `payloads.len()` is the live node count `N`.
/// - Each `payload.binary_words.len() == params.binary_word_count()`
///   and `payload.search_code.len() == params.search_code_len()`.
/// - `build_dist(a, b)` returns nonnegative f32; symmetric and
///   metric-like in expectation. Phase 5C-3 binds this to
///   `score_ip_codes_lite` over the encoded grouped-PQ codes.
pub fn build_and_persist_vamana<D>(
    params: BuildParams,
    payloads: &[NodePayload],
    build_dist: D,
) -> Result<BuildOutput, String>
where
    D: Fn(u32, u32) -> f32 + Copy,
{
    let n = payloads.len();
    if n == 0 {
        return Err("cannot build an empty Vamana index".into());
    }
    if params.graph_degree_r == 0 {
        return Err("graph_degree_r must be > 0".into());
    }
    if params.build_list_size_l == 0 {
        return Err("build_list_size_l must be > 0".into());
    }
    if !(params.alpha.is_finite() && params.alpha >= 1.0) {
        return Err(format!("alpha must be finite and >= 1.0, got {}", params.alpha));
    }
    if params.dimensions == 0 {
        return Err("dimensions must be > 0".into());
    }

    let medoid = approximate_medoid(n, MEDOID_SAMPLE_CAP, params.seed, build_dist);

    let graph = build_vamana_graph(
        n,
        medoid,
        params.graph_degree_r as usize,
        params.build_list_size_l as usize,
        params.alpha,
        params.seed,
        build_dist,
    );

    let w = params.binary_word_count();
    let c = params.search_code_len();

    let persisted = persist_vamana_graph(
        &graph,
        medoid,
        params.page_size,
        payloads,
        params.graph_degree_r,
        w,
        c,
    )?;

    let metadata = VamanaMetadataPage {
        format_version: INDEX_FORMAT_V3_DISKANN,
        entry_point: persisted.entry_point_tid,
        graph_degree_r: params.graph_degree_r,
        build_list_size_l: params.build_list_size_l,
        alpha: params.alpha,
        dimensions: params.dimensions,
        seed: params.seed,
        inserted_since_rebuild: 0,
        needs_medoid_refresh: false,
        transform_kind: VAMANA_TRANSFORM_KIND_SRHT,
        search_codec_kind: VAMANA_SEARCH_CODEC_GROUPED_PQ,
        payload_flags: params.payload_flags(),
        search_subvector_count: params.search_subvector_count,
        search_subvector_dim: params.search_subvector_dim,
        // Codebook chain head is owned by Phase 5C-3 (codebook
        // persistence). Build orchestrator leaves it INVALID; the
        // pgrx caller patches it after writing the codebook chain.
        grouped_codebook_head: ItemPointer::INVALID,
    };

    Ok(BuildOutput {
        metadata,
        persisted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_diskann::tuple::VamanaNodeTuple;
    use crate::storage::page::DEFAULT_PAGE_SIZE;

    /// 2-D synthetic vectors on a unit grid; L2² as build distance.
    /// Shared across tests so determinism is easy to assert.
    fn synth_vectors(n: usize) -> Vec<[f32; 2]> {
        (0..n)
            .map(|i| {
                let r = (i / 16) as f32;
                let c = (i % 16) as f32;
                [r, c]
            })
            .collect()
    }

    fn synth_payloads(n: usize, w: usize, c: usize) -> Vec<NodePayload> {
        (0..n)
            .map(|i| NodePayload {
                primary_heaptid: ItemPointer {
                    block_number: 5000 + i as u32,
                    offset_number: 1,
                },
                binary_words: vec![i as u64; w],
                search_code: vec![(i & 0xff) as u8; c],
            })
            .collect()
    }

    fn default_params(n_dims: u16) -> BuildParams {
        BuildParams {
            graph_degree_r: 16,
            build_list_size_l: 64,
            alpha: 1.2,
            dimensions: n_dims,
            search_subvector_count: 16,
            search_subvector_dim: n_dims / 16,
            seed: 17,
            page_size: DEFAULT_PAGE_SIZE,
            has_binary_sidecar: true,
        }
    }

    // BO-001: empty payloads errors.
    #[test]
    fn bo_001_empty_payloads_errors() {
        let params = default_params(64);
        let err = build_and_persist_vamana(params, &[], |_, _| 0.0).expect_err("empty");
        assert!(err.contains("empty"), "got: {err}");
    }

    // BO-002: zero R errors.
    #[test]
    fn bo_002_zero_r_errors() {
        let mut params = default_params(64);
        params.graph_degree_r = 0;
        let payloads = synth_payloads(2, params.binary_word_count(), params.search_code_len());
        let err = build_and_persist_vamana(params, &payloads, |_, _| 0.0).expect_err("zero R");
        assert!(err.contains("graph_degree_r"), "got: {err}");
    }

    // BO-003: alpha < 1.0 errors.
    #[test]
    fn bo_003_alpha_below_one_errors() {
        let mut params = default_params(64);
        params.alpha = 0.5;
        let payloads = synth_payloads(2, params.binary_word_count(), params.search_code_len());
        let err = build_and_persist_vamana(params, &payloads, |_, _| 0.0).expect_err("alpha");
        assert!(err.contains("alpha"), "got: {err}");
    }

    // BO-003b: payload_flags clears PAYLOAD_FLAG_COLD_RERANK_PAYLOAD
    // on V0 builds (ADR-046 frozen rule 1, ADR-047 frozen rule 4,
    // packet 11018). Sets GROUPED + BINARY_SIDECAR (when on) and
    // nothing else.
    #[test]
    fn bo_003b_payload_flags_clears_cold_rerank_in_v0() {
        use crate::am::ec_diskann::page::{
            PAYLOAD_FLAG_BINARY_SIDECAR, PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            PAYLOAD_FLAG_GROUPED_SEARCH_CODE,
        };

        let params = default_params(64);
        assert!(params.has_binary_sidecar);
        let flags = params.payload_flags();
        assert_eq!(
            flags & PAYLOAD_FLAG_COLD_RERANK_PAYLOAD,
            0,
            "V0 must not set PAYLOAD_FLAG_COLD_RERANK_PAYLOAD",
        );
        assert_ne!(flags & PAYLOAD_FLAG_GROUPED_SEARCH_CODE, 0);
        assert_ne!(flags & PAYLOAD_FLAG_BINARY_SIDECAR, 0);

        let mut params_no_sidecar = params;
        params_no_sidecar.has_binary_sidecar = false;
        let flags_ns = params_no_sidecar.payload_flags();
        assert_eq!(flags_ns & PAYLOAD_FLAG_COLD_RERANK_PAYLOAD, 0);
        assert_eq!(flags_ns & PAYLOAD_FLAG_BINARY_SIDECAR, 0);
        assert_ne!(flags_ns & PAYLOAD_FLAG_GROUPED_SEARCH_CODE, 0);
    }

    // BO-004: derivation rules — W=dimensions/64 when sidecar on,
    // 0 when off; C=M.div_ceil(2).
    #[test]
    fn bo_004_w_c_derivation() {
        let mut params = default_params(128);
        assert_eq!(params.binary_word_count(), 2); // 128 / 64
        params.has_binary_sidecar = false;
        assert_eq!(params.binary_word_count(), 0);

        params.search_subvector_count = 17;
        assert_eq!(params.search_code_len(), 9); // 17.div_ceil(2)
        params.search_subvector_count = 16;
        assert_eq!(params.search_code_len(), 8);
    }

    // BO-005: end-to-end build with synthetic L2 vectors. Asserts
    // metadata fields are populated, every node has a valid TID, and
    // the entry_point matches `node_to_tid[medoid]`.
    #[test]
    fn bo_005_end_to_end_metadata_and_persisted() {
        let n = 64;
        let vectors = synth_vectors(n);
        let dist = |a: u32, b: u32| -> f32 {
            let av = vectors[a as usize];
            let bv = vectors[b as usize];
            let dx = av[0] - bv[0];
            let dy = av[1] - bv[1];
            dx * dx + dy * dy
        };

        let params = default_params(64);
        let payloads = synth_payloads(n, params.binary_word_count(), params.search_code_len());

        let out = build_and_persist_vamana(params, &payloads, dist).expect("build");

        assert_eq!(out.metadata.format_version, INDEX_FORMAT_V3_DISKANN);
        assert_eq!(out.metadata.graph_degree_r, params.graph_degree_r);
        assert_eq!(out.metadata.build_list_size_l, params.build_list_size_l);
        assert_eq!(out.metadata.alpha.to_bits(), params.alpha.to_bits());
        assert_eq!(out.metadata.dimensions, params.dimensions);
        assert_eq!(out.metadata.seed, params.seed);
        assert_eq!(out.metadata.transform_kind, VAMANA_TRANSFORM_KIND_SRHT);
        assert_eq!(out.metadata.search_codec_kind, VAMANA_SEARCH_CODEC_GROUPED_PQ);
        assert_eq!(out.metadata.payload_flags, params.payload_flags());
        assert_eq!(out.metadata.search_subvector_count, params.search_subvector_count);
        assert_eq!(out.metadata.grouped_codebook_head, ItemPointer::INVALID);

        // Entry point = medoid TID; never INVALID for a non-empty build.
        assert_ne!(out.metadata.entry_point, ItemPointer::INVALID);
        assert_eq!(out.persisted.node_to_tid.len(), n);
        for (i, tid) in out.persisted.node_to_tid.iter().enumerate() {
            assert_ne!(*tid, ItemPointer::INVALID, "node {i} has no TID");
        }
    }

    // BO-006: payload-shape mismatch surfaces from persist (W/C
    // pre-validation in the persist layer fires).
    #[test]
    fn bo_006_payload_shape_mismatch_errors() {
        let params = default_params(64);
        let mut payloads = synth_payloads(4, params.binary_word_count(), params.search_code_len());
        payloads[0].search_code.pop();
        let err = build_and_persist_vamana(params, &payloads, |_, _| 0.0)
            .expect_err("shape mismatch");
        assert!(err.contains("search_code"), "got: {err}");
    }

    // BO-007: deterministic — same seed + same dist + same payloads ⇒
    // bit-equal entry_point and node_to_tid layout.
    #[test]
    fn bo_007_deterministic_for_fixed_seed() {
        let n = 32;
        let vectors = synth_vectors(n);
        let dist = |a: u32, b: u32| -> f32 {
            let av = vectors[a as usize];
            let bv = vectors[b as usize];
            let dx = av[0] - bv[0];
            let dy = av[1] - bv[1];
            dx * dx + dy * dy
        };

        let params = default_params(64);
        let payloads = synth_payloads(n, params.binary_word_count(), params.search_code_len());

        let a = build_and_persist_vamana(params, &payloads, dist).expect("build a");
        let b = build_and_persist_vamana(params, &payloads, dist).expect("build b");

        assert_eq!(a.metadata.entry_point, b.metadata.entry_point);
        assert_eq!(a.persisted.node_to_tid, b.persisted.node_to_tid);
        assert_eq!(a.persisted.persistence_order, b.persisted.persistence_order);
    }

    // BO-008: round-trip — every persisted tuple decodes with the
    // metadata-derived (R, W, C) and the entry-point TID maps back to
    // the medoid's node id via node_to_tid.
    #[test]
    fn bo_008_persisted_tuples_decode_with_metadata_constants() {
        let n = 16;
        let vectors = synth_vectors(n);
        let dist = |a: u32, b: u32| -> f32 {
            let av = vectors[a as usize];
            let bv = vectors[b as usize];
            let dx = av[0] - bv[0];
            let dy = av[1] - bv[1];
            dx * dx + dy * dy
        };

        let params = default_params(64);
        let payloads = synth_payloads(n, params.binary_word_count(), params.search_code_len());
        let out = build_and_persist_vamana(params, &payloads, dist).expect("build");

        let r = out.metadata.graph_degree_r;
        let w = (out.metadata.dimensions as usize).div_ceil(64); // sidecar on
        let c = (out.metadata.search_subvector_count as usize).div_ceil(2);

        for (node, tid) in out.persisted.node_to_tid.iter().enumerate() {
            let page = out.persisted.chain.get_page(tid.block_number).expect("page");
            let bytes = page.raw_tuple(*tid).expect("tuple");
            let tuple = VamanaNodeTuple::decode(bytes, r, w, c).expect("decode");
            assert_eq!(tuple.primary_heaptid, payloads[node].primary_heaptid);
            assert_eq!(tuple.binary_words, payloads[node].binary_words);
            assert_eq!(tuple.search_code, payloads[node].search_code);
        }

        // Entry point round-trips to a node id.
        let medoid_node = out
            .persisted
            .node_to_tid
            .iter()
            .position(|t| *t == out.metadata.entry_point)
            .expect("entry_point in node_to_tid");
        assert!(medoid_node < n);
    }
}
