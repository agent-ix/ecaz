use std::sync::Arc;

use crate::am::common::{
    candidate_batch::{CandidateMeta, CandidatePayload},
    quant_codec::{EncodedQuantPayload, QuantCodec, QuantCodecKind, QuantSearchCodecTag},
    training::{self, GroupedPq4Model},
};
use crate::quant::{
    grouped_pq::{
        build_grouped_pq_lut_f32, encode_grouped_pq, grouped_pq_score_f32, GROUPED_PQ_CENTROIDS,
    },
    prod::ProdQuantizer,
    rabitq::{code_len_for, PreparedEstimator, RaBitQQuantizer},
    Quantizer,
};
use crate::DEFAULT_QUANT_BITS;

use super::{
    options::StorageFormat,
    page::{VamanaMetadataPage, VAMANA_SEARCH_CODEC_GROUPED_PQ, VAMANA_SEARCH_CODEC_RABITQ},
    scan_query::{encode_query_srht, read_grouped_codebook_chain},
};
use crate::storage::page::{DataPageChain, ItemPointer};

pub(super) const DISKANN_RABITQ_BITS: u8 = 1;

#[derive(Debug, Clone)]
pub(super) struct DiskannEncodedPayload {
    pub(super) binary_words: Vec<u64>,
    pub(super) search_code: Vec<u8>,
}

pub(super) enum DiskannBuildCodec {
    PqFastScan {
        model: GroupedPq4Model,
        binary_quantizer: Option<Arc<ProdQuantizer>>,
    },
    RaBitQ {
        quantizer: Arc<RaBitQQuantizer>,
    },
}

impl DiskannBuildCodec {
    pub(super) fn prepare(
        storage_format: StorageFormat,
        source_refs: &[&[f32]],
        dimensions: usize,
        seed: u64,
        pq_group_size: usize,
        train_size: usize,
        kmeans_iters: usize,
    ) -> Result<Self, String> {
        match storage_format {
            StorageFormat::PqFastScan => {
                let model = training::train_grouped_pq4_model(
                    source_refs,
                    dimensions,
                    seed,
                    pq_group_size,
                    train_size,
                    kmeans_iters,
                )?;
                let sidecar_word_count = training::persisted_binary_sidecar_word_count(
                    u16::try_from(dimensions)
                        .map_err(|_| format!("ec_diskann dimensions {dimensions} exceed u16"))?,
                    DEFAULT_QUANT_BITS,
                    seed,
                );
                let binary_quantizer = (sidecar_word_count > 0)
                    .then(|| ProdQuantizer::cached(dimensions, DEFAULT_QUANT_BITS, seed));
                Ok(Self::PqFastScan {
                    model,
                    binary_quantizer,
                })
            }
            StorageFormat::RaBitQ => {
                let quantizer = RaBitQQuantizer::cached_seeded_srht_bits(
                    dimensions,
                    seed,
                    DISKANN_RABITQ_BITS,
                )?;
                Ok(Self::RaBitQ { quantizer })
            }
        }
    }

    pub(super) fn encode(&self, source_vector: &[f32]) -> DiskannEncodedPayload {
        match self {
            Self::PqFastScan {
                model,
                binary_quantizer,
            } => {
                let search_code = training::derive_grouped_pq4_code(source_vector, &model);
                let binary_words = match binary_quantizer {
                    Some(q) => {
                        let encoded = q.encode(source_vector);
                        let mut code = encoded.mse_packed;
                        code.extend_from_slice(&encoded.qjl_packed);
                        training::derive_persisted_binary_words(q, &code)
                    }
                    None => Vec::new(),
                };
                DiskannEncodedPayload {
                    binary_words,
                    search_code,
                }
            }
            Self::RaBitQ { quantizer } => DiskannEncodedPayload {
                binary_words: Vec::new(),
                search_code: quantizer.encode_code(source_vector).into_vec(),
            },
        }
    }

    pub(super) fn search_codec_kind(&self) -> u8 {
        match self {
            Self::PqFastScan { .. } => VAMANA_SEARCH_CODEC_GROUPED_PQ,
            Self::RaBitQ { .. } => VAMANA_SEARCH_CODEC_RABITQ,
        }
    }

    pub(super) fn search_subvector_count(&self) -> u16 {
        match self {
            Self::PqFastScan { model, .. } => u16::try_from(model.group_count)
                .expect("ec_diskann grouped-PQ group count should fit in u16"),
            Self::RaBitQ { .. } => 0,
        }
    }

    pub(super) fn search_subvector_dim(&self) -> u16 {
        match self {
            Self::PqFastScan { model, .. } => u16::try_from(model.group_size)
                .expect("ec_diskann grouped-PQ group size should fit in u16"),
            Self::RaBitQ { .. } => u16::from(DISKANN_RABITQ_BITS),
        }
    }

    pub(super) fn has_binary_sidecar(&self) -> bool {
        matches!(
            self,
            Self::PqFastScan {
                binary_quantizer: Some(_),
                ..
            }
        )
    }

    pub(super) fn pq_model(&self) -> Option<GroupedPq4Model> {
        match self {
            Self::PqFastScan { model, .. } => Some(model.clone()),
            Self::RaBitQ { .. } => None,
        }
    }
}

pub(super) enum DiskannPreparedPrefilter {
    BinarySidecar {
        rotated_query: Vec<f32>,
        query_words: Vec<u64>,
    },
    GroupedPq {
        rotated_query: Vec<f32>,
        flat_codebooks: Vec<f32>,
        query_lut: Vec<f32>,
        group_count: usize,
        group_size: usize,
    },
    RaBitQ {
        prepared: PreparedEstimator,
    },
}

impl DiskannPreparedPrefilter {
    pub(super) fn score(&self, tuple: &super::tuple::VamanaNodeTuple) -> f32 {
        match self {
            Self::BinarySidecar { query_words, .. } => {
                let codec = DiskannBinarySidecarPrefilterCodec::new(query_words.len());
                let code = u64_words_to_le_bytes(&tuple.binary_words);
                QuantCodec::score_ip_candidate(
                    &codec,
                    query_words,
                    CandidatePayload::new(&code, CandidateMeta::Binary),
                )
                .unwrap_or_else(|error| pgrx::error!("{error}"))
            }
            Self::GroupedPq {
                query_lut,
                group_count,
                group_size,
                ..
            } => {
                let codec = DiskannGroupedPqPrefilterCodec::new(*group_count, *group_size);
                -QuantCodec::score_ip_candidate(
                    &codec,
                    query_lut,
                    CandidatePayload::new(
                        &tuple.search_code,
                        CandidateMeta::GroupedPq {
                            group_count: *group_count,
                        },
                    ),
                )
                .unwrap_or_else(|error| pgrx::error!("{error}"))
            }
            Self::RaBitQ { prepared } => {
                let codec =
                    DiskannRaBitQPrefilterCodec::new(tuple.search_code.len(), DISKANN_RABITQ_BITS);
                -QuantCodec::score_ip_candidate(
                    &codec,
                    prepared,
                    CandidatePayload::new(&tuple.search_code, CandidateMeta::RaBitQ),
                )
                .unwrap_or_else(|error| pgrx::error!("{error}"))
            }
        }
    }

    pub(super) fn load_into_scan_opaque(self, opaque: &mut super::scan_state::DiskannScanOpaque) {
        match self {
            Self::BinarySidecar {
                rotated_query,
                query_words,
            } => {
                opaque.query_rotated = rotated_query;
                opaque.query_binary_words = query_words;
            }
            Self::GroupedPq {
                rotated_query,
                flat_codebooks,
                query_lut,
                ..
            } => {
                opaque.query_rotated = rotated_query;
                opaque.flat_codebooks = flat_codebooks;
                opaque.query_lut = query_lut;
            }
            Self::RaBitQ { .. } => {}
        }
    }
}

fn u64_words_to_le_bytes(words: &[u64]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(words.len() * std::mem::size_of::<u64>());
    for word in words {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes
}

fn le_bytes_to_u64_words(bytes: &[u8]) -> Result<Vec<u64>, String> {
    if bytes.len() % std::mem::size_of::<u64>() != 0 {
        return Err(format!(
            "ec_diskann binary sidecar payload byte length {} is not u64-aligned",
            bytes.len()
        ));
    }
    Ok(bytes
        .chunks_exact(std::mem::size_of::<u64>())
        .map(|chunk| u64::from_le_bytes(chunk.try_into().expect("chunk size is fixed at 8")))
        .collect())
}

struct DiskannBinarySidecarPrefilterCodec {
    word_count: usize,
}

impl DiskannBinarySidecarPrefilterCodec {
    fn new(word_count: usize) -> Self {
        Self { word_count }
    }
}

impl QuantCodec for DiskannBinarySidecarPrefilterCodec {
    type PreparedQuery = Vec<u64>;

    fn codec_kind(&self) -> QuantCodecKind {
        QuantCodecKind::Binary
    }

    fn search_codec_tag(&self) -> QuantSearchCodecTag {
        QuantSearchCodecTag::Binary
    }

    fn payload_len(&self) -> usize {
        self.word_count * std::mem::size_of::<u64>()
    }

    fn encode_source(&self, source: &[f32]) -> Result<EncodedQuantPayload, String> {
        Err(format!(
            "ec_diskann binary-sidecar prefilter codec uses persisted sidecar words; source length {}",
            source.len()
        ))
    }

    fn prepare_ip_query(&self, query: &[f32]) -> Result<Self::PreparedQuery, String> {
        Err(format!(
            "ec_diskann binary-sidecar prefilter codec uses scan-owned query words; query length {}",
            query.len()
        ))
    }

    fn score_ip_candidate(
        &self,
        prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>,
    ) -> Result<f32, String> {
        if prepared_query.len() != self.word_count {
            return Err(format!(
                "ec_diskann binary-sidecar prepared word count {} does not match codec word count {}",
                prepared_query.len(),
                self.word_count
            ));
        }
        let candidate_words = le_bytes_to_u64_words(payload.code)?;
        if candidate_words.len() != self.word_count {
            return Err(format!(
                "ec_diskann binary-sidecar candidate word count {} does not match codec word count {}",
                candidate_words.len(),
                self.word_count
            ));
        }
        Ok(super::scan_query::hamming_xor_popcount(prepared_query, &candidate_words) as f32)
    }
}

struct DiskannGroupedPqPrefilterCodec {
    group_count: usize,
    group_size: usize,
}

impl DiskannGroupedPqPrefilterCodec {
    fn new(group_count: usize, group_size: usize) -> Self {
        Self {
            group_count,
            group_size,
        }
    }
}

impl QuantCodec for DiskannGroupedPqPrefilterCodec {
    type PreparedQuery = Vec<f32>;

    fn codec_kind(&self) -> QuantCodecKind {
        QuantCodecKind::GroupedPq
    }

    fn search_codec_tag(&self) -> QuantSearchCodecTag {
        QuantSearchCodecTag::GroupedPq {
            group_count: self.group_count,
            group_size: self.group_size,
        }
    }

    fn payload_len(&self) -> usize {
        self.group_count.div_ceil(2)
    }

    fn encode_source(&self, source: &[f32]) -> Result<EncodedQuantPayload, String> {
        Err(format!(
            "ec_diskann grouped-PQ prefilter codec uses persisted grouped search codes; source length {}",
            source.len()
        ))
    }

    fn prepare_ip_query(&self, query: &[f32]) -> Result<Self::PreparedQuery, String> {
        Err(format!(
            "ec_diskann grouped-PQ prefilter codec uses scan-owned query LUTs; query length {}",
            query.len()
        ))
    }

    fn score_ip_candidate(
        &self,
        prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>,
    ) -> Result<f32, String> {
        if payload.code.len() != self.payload_len() {
            return Err(format!(
                "ec_diskann grouped-PQ search code len {} does not match codec width {}",
                payload.code.len(),
                self.payload_len()
            ));
        }
        if let CandidateMeta::GroupedPq { group_count } = payload.meta {
            if group_count != self.group_count {
                return Err(format!(
                    "ec_diskann grouped-PQ candidate group count {} does not match codec group count {}",
                    group_count,
                    self.group_count
                ));
            }
        }
        Ok(grouped_pq_score_f32(
            prepared_query,
            self.group_count,
            payload.code,
        ))
    }
}

struct DiskannRaBitQPrefilterCodec {
    search_code_len: usize,
    bits: u8,
}

impl DiskannRaBitQPrefilterCodec {
    fn new(search_code_len: usize, bits: u8) -> Self {
        Self {
            search_code_len,
            bits,
        }
    }
}

impl QuantCodec for DiskannRaBitQPrefilterCodec {
    type PreparedQuery = PreparedEstimator;

    fn codec_kind(&self) -> QuantCodecKind {
        QuantCodecKind::RaBitQ
    }

    fn search_codec_tag(&self) -> QuantSearchCodecTag {
        QuantSearchCodecTag::RaBitQ { bits: self.bits }
    }

    fn payload_len(&self) -> usize {
        self.search_code_len
    }

    fn encode_source(&self, source: &[f32]) -> Result<EncodedQuantPayload, String> {
        Err(format!(
            "ec_diskann RaBitQ prefilter codec uses DiskANN build/insert encoding; source length {}",
            source.len()
        ))
    }

    fn prepare_ip_query(&self, query: &[f32]) -> Result<Self::PreparedQuery, String> {
        Err(format!(
            "ec_diskann RaBitQ prefilter codec uses scan-owned prepared query state; query length {}",
            query.len()
        ))
    }

    fn score_ip_candidate(
        &self,
        prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>,
    ) -> Result<f32, String> {
        if payload.code.len() != self.search_code_len {
            return Err(format!(
                "ec_diskann RaBitQ search code len {} does not match codec width {}",
                payload.code.len(),
                self.search_code_len
            ));
        }
        Ok(prepared_query.estimate_ip_scalar_only(payload.code))
    }
}

pub(super) fn metadata_search_code_len(metadata: &VamanaMetadataPage) -> Result<usize, String> {
    match metadata.search_codec_kind {
        VAMANA_SEARCH_CODEC_GROUPED_PQ => {
            Ok(usize::from(metadata.search_subvector_count).div_ceil(2))
        }
        VAMANA_SEARCH_CODEC_RABITQ => {
            let bits = rabitq_bits_from_metadata(metadata)?;
            code_len_for(usize::from(metadata.dimensions), bits)
        }
        other => Err(format!("ec_diskann unsupported search codec kind {other}")),
    }
}

pub(super) fn rabitq_bits_from_metadata(metadata: &VamanaMetadataPage) -> Result<u8, String> {
    if metadata.search_codec_kind != VAMANA_SEARCH_CODEC_RABITQ {
        return Err("ec_diskann metadata is not RaBitQ".to_owned());
    }
    u8::try_from(metadata.search_subvector_dim).map_err(|_| {
        format!(
            "ec_diskann RaBitQ bit width {} exceeds u8",
            metadata.search_subvector_dim
        )
    })
}

pub(super) fn encode_insert_payload(
    metadata: &VamanaMetadataPage,
    chain: &DataPageChain,
    source_vector: &[f32],
) -> Result<DiskannEncodedPayload, String> {
    match metadata.search_codec_kind {
        VAMANA_SEARCH_CODEC_GROUPED_PQ => {
            encode_grouped_insert_payload(metadata, chain, source_vector)
        }
        VAMANA_SEARCH_CODEC_RABITQ => {
            let bits = rabitq_bits_from_metadata(metadata)?;
            let quantizer = RaBitQQuantizer::cached_seeded_srht_bits(
                usize::from(metadata.dimensions),
                metadata.seed,
                bits,
            )?;
            Ok(DiskannEncodedPayload {
                binary_words: Vec::new(),
                search_code: quantizer.encode_code(source_vector).into_vec(),
            })
        }
        other => Err(format!(
            "ec_diskann insert payload derivation does not support search codec kind {other}"
        )),
    }
}

fn encode_grouped_insert_payload(
    metadata: &VamanaMetadataPage,
    chain: &DataPageChain,
    source_vector: &[f32],
) -> Result<DiskannEncodedPayload, String> {
    let group_count = usize::from(metadata.search_subvector_count);
    let group_size = usize::from(metadata.search_subvector_dim);
    if group_count == 0 || group_size == 0 {
        return Err(
            "ec_diskann insert payload derivation requires non-zero grouped search shape".into(),
        );
    }
    if metadata.grouped_codebook_head == ItemPointer::INVALID {
        return Err(
            "ec_diskann insert payload derivation requires persisted grouped codebooks".into(),
        );
    }
    let flat_codebooks = read_grouped_codebook_chain(
        chain,
        metadata.grouped_codebook_head,
        group_count,
        group_size * GROUPED_PQ_CENTROIDS,
    )?;
    let rotated = encode_query_srht(
        source_vector,
        usize::from(metadata.dimensions),
        metadata.seed,
    );
    let expected_rotated_len = group_count
        .checked_mul(group_size)
        .ok_or_else(|| "ec_diskann grouped search shape overflows usize".to_owned())?;
    if rotated.len() != expected_rotated_len {
        return Err(format!(
            "ec_diskann insert payload rotated query length mismatch: got {}, expected {} from metadata",
            rotated.len(),
            expected_rotated_len
        ));
    }
    let search_code = encode_grouped_pq(
        &rotated,
        flat_codebooks.chunks_exact(GROUPED_PQ_CENTROIDS * group_size),
        group_size,
    );
    let binary_words = if metadata.payload_flags & super::page::PAYLOAD_FLAG_BINARY_SIDECAR != 0 {
        let quantizer = ProdQuantizer::cached(
            usize::from(metadata.dimensions),
            DEFAULT_QUANT_BITS,
            metadata.seed,
        );
        let encoded = quantizer.encode(source_vector);
        let mut code = encoded.mse_packed;
        code.extend_from_slice(&encoded.qjl_packed);
        training::derive_persisted_binary_words(&quantizer, &code)
    } else {
        Vec::new()
    };
    Ok(DiskannEncodedPayload {
        binary_words,
        search_code,
    })
}

pub(super) fn prepare_prefilter(
    chain: Option<&DataPageChain>,
    metadata: &VamanaMetadataPage,
    raw_query: &[f32],
    prefilter_kind: super::options::PrefilterKind,
    context: &str,
) -> Result<DiskannPreparedPrefilter, String> {
    match metadata.search_codec_kind {
        VAMANA_SEARCH_CODEC_RABITQ => {
            match prefilter_kind {
                super::options::PrefilterKind::Auto => {}
                super::options::PrefilterKind::BinarySidecar => {
                    return Err(format!(
                        "ec_diskann.prefilter_kind=binary_sidecar requested but {context} is a RaBitQ index with no binary sidecar"
                    ));
                }
                super::options::PrefilterKind::GroupedPq => {
                    return Err(format!(
                        "ec_diskann.prefilter_kind=grouped_pq requested but {context} is a RaBitQ index"
                    ));
                }
            }
            let bits = rabitq_bits_from_metadata(metadata)?;
            let quantizer = RaBitQQuantizer::cached_seeded_srht_bits(
                usize::from(metadata.dimensions),
                metadata.seed,
                bits,
            )?;
            Ok(DiskannPreparedPrefilter::RaBitQ {
                prepared: quantizer.prepare_estimator(raw_query),
            })
        }
        VAMANA_SEARCH_CODEC_GROUPED_PQ => prepare_grouped_or_sidecar_prefilter(
            chain,
            metadata,
            raw_query,
            prefilter_kind,
            context,
        ),
        other => Err(format!(
            "ec_diskann {context} unsupported search codec kind {other}"
        )),
    }
}

fn prepare_grouped_or_sidecar_prefilter(
    chain: Option<&DataPageChain>,
    metadata: &VamanaMetadataPage,
    raw_query: &[f32],
    prefilter_kind: super::options::PrefilterKind,
    context: &str,
) -> Result<DiskannPreparedPrefilter, String> {
    let has_binary_sidecar = metadata.payload_flags & super::page::PAYLOAD_FLAG_BINARY_SIDECAR != 0;
    let use_binary_sidecar = match prefilter_kind {
        super::options::PrefilterKind::Auto => has_binary_sidecar,
        super::options::PrefilterKind::BinarySidecar => {
            if !has_binary_sidecar {
                return Err(format!(
                    "ec_diskann.prefilter_kind=binary_sidecar requested but {context} has no binary sidecar"
                ));
            }
            true
        }
        super::options::PrefilterKind::GroupedPq => false,
    };
    let dimensions = metadata.dimensions as usize;
    let rotated_query = encode_query_srht(raw_query, dimensions, metadata.seed);
    if use_binary_sidecar {
        return Ok(DiskannPreparedPrefilter::BinarySidecar {
            query_words: super::scan_query::pack_query_sign_bits(&rotated_query, dimensions),
            rotated_query,
        });
    }
    let group_count = usize::from(metadata.search_subvector_count);
    let group_size = usize::from(metadata.search_subvector_dim);
    if group_count == 0 || group_size == 0 {
        return Err(format!(
            "ec_diskann {context} requires grouped-PQ metadata: group_count={}, group_size={}",
            group_count, group_size
        ));
    }
    if rotated_query.len() != group_count * group_size {
        return Err(format!(
            "ec_diskann {context} rotated query length {} does not match group_count {group_count} * group_size {group_size}",
            rotated_query.len()
        ));
    }
    let chain = chain.ok_or_else(|| {
        format!("ec_diskann {context} grouped-PQ prefilter requires materialized index chain")
    })?;
    let flat_codebooks = read_grouped_codebook_chain(
        chain,
        metadata.grouped_codebook_head,
        group_count,
        GROUPED_PQ_CENTROIDS * group_size,
    )?;
    let query_lut = build_grouped_pq_lut_f32(&rotated_query, &flat_codebooks, group_size);
    Ok(DiskannPreparedPrefilter::GroupedPq {
        rotated_query,
        flat_codebooks,
        query_lut,
        group_count,
        group_size,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::am::ec_diskann::page::VamanaMetadataPage;
    use crate::am::ec_diskann::tuple::VamanaNodeTuple;

    fn tuple_with_codes(binary_words: Vec<u64>, search_code: Vec<u8>) -> VamanaNodeTuple {
        VamanaNodeTuple {
            deleted: false,
            has_overflow_heaptids: false,
            primary_heaptid: ItemPointer::INVALID,
            rerank_tid: ItemPointer::INVALID,
            binary_words,
            search_code,
            neighbors: Vec::new(),
            neighbor_count: 0,
        }
    }

    #[test]
    fn rabitq_metadata_len_uses_one_bit_payload() {
        let mut metadata = VamanaMetadataPage::empty(32, 100, 1.2, 1536, 42);
        metadata.search_codec_kind = VAMANA_SEARCH_CODEC_RABITQ;
        metadata.search_subvector_count = 0;
        metadata.search_subvector_dim = u16::from(DISKANN_RABITQ_BITS);

        assert_eq!(metadata_search_code_len(&metadata).unwrap(), 204);
    }

    #[test]
    fn rabitq_insert_payload_has_no_sidecar() {
        let metadata = VamanaMetadataPage {
            search_codec_kind: VAMANA_SEARCH_CODEC_RABITQ,
            search_subvector_count: 0,
            search_subvector_dim: u16::from(DISKANN_RABITQ_BITS),
            payload_flags: 0,
            dimensions: 8,
            seed: 42,
            ..VamanaMetadataPage::empty(4, 16, 1.2, 8, 42)
        };
        let chain = DataPageChain::new(8192);
        let source = vec![1.0_f32 / (8.0_f32).sqrt(); 8];
        let payload = encode_insert_payload(&metadata, &chain, &source).unwrap();

        assert!(payload.binary_words.is_empty());
        assert_eq!(
            payload.search_code.len(),
            code_len_for(8, DISKANN_RABITQ_BITS).unwrap()
        );
    }

    #[test]
    fn rabitq_prefilter_rejects_binary_sidecar_override() {
        let metadata = VamanaMetadataPage {
            search_codec_kind: VAMANA_SEARCH_CODEC_RABITQ,
            search_subvector_count: 0,
            search_subvector_dim: u16::from(DISKANN_RABITQ_BITS),
            payload_flags: 0,
            dimensions: 8,
            seed: 42,
            ..VamanaMetadataPage::empty(4, 16, 1.2, 8, 42)
        };
        let query = vec![1.0_f32; 8];

        let err = match prepare_prefilter(
            None,
            &metadata,
            &query,
            super::super::options::PrefilterKind::BinarySidecar,
            "test",
        ) {
            Ok(_) => panic!("RaBitQ indexes should not satisfy a binary_sidecar override"),
            Err(err) => err,
        };

        assert!(
            err.contains("no binary sidecar"),
            "error should explain the forced sidecar mismatch: {err}"
        );
    }

    #[test]
    fn diskann_binary_sidecar_prefilter_codec_matches_direct_hamming_score() {
        let query_words = vec![0b1010_u64, 0b1111_0000_u64];
        let tuple = tuple_with_codes(vec![0b0011_u64, 0b1010_0000_u64], Vec::new());
        let prefilter = DiskannPreparedPrefilter::BinarySidecar {
            rotated_query: Vec::new(),
            query_words: query_words.clone(),
        };
        let codec = DiskannBinarySidecarPrefilterCodec::new(query_words.len());
        let payload_bytes = u64_words_to_le_bytes(&tuple.binary_words);

        assert_eq!(QuantCodec::codec_kind(&codec), QuantCodecKind::Binary);
        assert_eq!(
            QuantCodec::search_codec_tag(&codec),
            QuantSearchCodecTag::Binary
        );
        assert_eq!(QuantCodec::payload_len(&codec), payload_bytes.len());

        let observed = QuantCodec::score_ip_candidate(
            &codec,
            &query_words,
            CandidatePayload::new(&payload_bytes, CandidateMeta::Binary),
        )
        .unwrap();
        let expected =
            super::super::scan_query::hamming_xor_popcount(&query_words, &tuple.binary_words)
                as f32;

        assert_eq!(observed, expected);
        assert_eq!(prefilter.score(&tuple), expected);
    }

    #[test]
    fn diskann_grouped_pq_prefilter_codec_matches_direct_search_code_score() {
        let group_count = 3;
        let group_size = 2;
        let mut query_lut = vec![0.0_f32; group_count * GROUPED_PQ_CENTROIDS];
        query_lut[1] = 1.25;
        query_lut[GROUPED_PQ_CENTROIDS + 3] = -0.5;
        query_lut[2 * GROUPED_PQ_CENTROIDS + 2] = 2.0;
        let tuple = tuple_with_codes(Vec::new(), vec![0x31, 0x02]);
        let prefilter = DiskannPreparedPrefilter::GroupedPq {
            rotated_query: Vec::new(),
            flat_codebooks: Vec::new(),
            query_lut: query_lut.clone(),
            group_count,
            group_size,
        };
        let codec = DiskannGroupedPqPrefilterCodec::new(group_count, group_size);

        assert_eq!(QuantCodec::codec_kind(&codec), QuantCodecKind::GroupedPq);
        assert_eq!(
            QuantCodec::search_codec_tag(&codec),
            QuantSearchCodecTag::GroupedPq {
                group_count,
                group_size,
            }
        );
        assert_eq!(QuantCodec::payload_len(&codec), tuple.search_code.len());

        let observed = QuantCodec::score_ip_candidate(
            &codec,
            &query_lut,
            CandidatePayload::new(&tuple.search_code, CandidateMeta::GroupedPq { group_count }),
        )
        .unwrap();
        let expected = grouped_pq_score_f32(&query_lut, group_count, &tuple.search_code);

        assert_eq!(observed.to_bits(), expected.to_bits());
        assert_eq!(prefilter.score(&tuple).to_bits(), (-expected).to_bits());
    }

    #[test]
    fn diskann_rabitq_prefilter_codec_matches_direct_search_code_score() {
        let dimensions = 16;
        let vector = vec![1.0_f32 / (dimensions as f32).sqrt(); dimensions];
        let query = vector.clone();
        let quantizer =
            RaBitQQuantizer::cached_seeded_srht_bits(dimensions, 42, DISKANN_RABITQ_BITS).unwrap();
        let search_code = quantizer.encode_code(&vector).into_vec();
        let prepared = quantizer.prepare_estimator(&query);
        let tuple = tuple_with_codes(Vec::new(), search_code.clone());
        let prefilter = DiskannPreparedPrefilter::RaBitQ { prepared };
        let codec = DiskannRaBitQPrefilterCodec::new(search_code.len(), DISKANN_RABITQ_BITS);

        assert_eq!(QuantCodec::codec_kind(&codec), QuantCodecKind::RaBitQ);
        assert_eq!(
            QuantCodec::search_codec_tag(&codec),
            QuantSearchCodecTag::RaBitQ {
                bits: DISKANN_RABITQ_BITS,
            }
        );
        assert_eq!(QuantCodec::payload_len(&codec), search_code.len());

        let direct_prepared = quantizer.prepare_estimator(&query);
        let observed = QuantCodec::score_ip_candidate(
            &codec,
            &direct_prepared,
            CandidatePayload::new(&search_code, CandidateMeta::RaBitQ),
        )
        .unwrap();
        let expected = direct_prepared.estimate_ip_scalar_only(&search_code);

        assert_eq!(observed.to_bits(), expected.to_bits());
        assert_eq!(prefilter.score(&tuple).to_bits(), (-expected).to_bits());
    }
}
