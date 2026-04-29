use super::options::StorageFormat;
use super::page;
use crate::quant::grouped_pq::{
    build_grouped_pq_lut_f32, grouped_pq_score_f32, GROUPED_PQ_CENTROIDS,
};
use crate::quant::prod::{ExactScoreMode, PreparedLutNoQjl4BitQuery, PreparedQuery, ProdQuantizer};
use crate::quant::rabitq::{code_len_for, PreparedEstimator, RaBitQQuantizer};
use crate::quant::rotation;
use crate::quant::Quantizer;
use crate::storage::page::ItemPointer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IvfQuantizerProfile {
    TurboQuant,
    PqFastScan {
        group_count: usize,
        group_size: usize,
    },
    RaBitQ,
}

pub(super) enum IvfPreparedQuery {
    TurboQuant(PreparedQuery),
    TurboQuantNoQjl4BitLut(PreparedLutNoQjl4BitQuery),
    PqFastScan {
        lut: Vec<f32>,
        group_count: usize,
        suffix_max: Vec<f32>,
    },
    RaBitQ(PreparedEstimator),
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct IvfPqFastScanModel {
    pub(super) group_count: usize,
    pub(super) group_size: usize,
    pub(super) flat_codebooks: Vec<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct IvfQuantizer {
    profile: IvfQuantizerProfile,
    dimensions: usize,
}

impl IvfQuantizer {
    pub(super) fn resolve(
        storage_format: StorageFormat,
        dimensions: usize,
    ) -> Result<Self, String> {
        Self::resolve_with_pq_group_size(storage_format, dimensions, None)
    }

    pub(super) fn resolve_with_pq_group_size(
        storage_format: StorageFormat,
        dimensions: usize,
        pq_group_size: Option<usize>,
    ) -> Result<Self, String> {
        storage_format.validate_v1_supported()?;
        let profile = match storage_format {
            StorageFormat::Auto | StorageFormat::TurboQuant => IvfQuantizerProfile::TurboQuant,
            StorageFormat::PqFastScan => {
                let transform_dim = rotation::effective_transform_dim(dimensions);
                let group_size = resolve_pq_fastscan_group_size(dimensions, pq_group_size)?;
                IvfQuantizerProfile::PqFastScan {
                    group_count: transform_dim / group_size,
                    group_size,
                }
            }
            StorageFormat::RaBitQ => IvfQuantizerProfile::RaBitQ,
        };
        Ok(Self {
            profile,
            dimensions,
        })
    }

    pub(super) fn encode_source(self, source: &[f32]) -> Result<(u16, f32, Vec<u8>), String> {
        if source.is_empty() {
            return Err("embedding must not be empty".to_owned());
        }
        if source.len() != self.dimensions {
            return Err(format!(
                "embedding dimension mismatch: got {}, expected {}",
                source.len(),
                self.dimensions
            ));
        }
        let dimensions = u16::try_from(source.len())
            .map_err(|_| format!("embedding dimension {} exceeds maximum 65535", source.len()))?;

        match self.profile {
            IvfQuantizerProfile::TurboQuant => {
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                let encoded = quantizer.encode(source);
                let mut payload = encoded.mse_packed;
                payload.extend_from_slice(&encoded.qjl_packed);
                Ok((dimensions, encoded.gamma, payload))
            }
            IvfQuantizerProfile::RaBitQ => {
                let quantizer = self.rabitq_quantizer()?;
                Ok((dimensions, 0.0, quantizer.encode_code(source).into_vec()))
            }
            IvfQuantizerProfile::PqFastScan { .. } => {
                Err("ec_ivf pq_fastscan encoding requires a trained grouped codebook".to_owned())
            }
        }
    }

    pub(super) fn encode_source_with_pq_model(
        self,
        source: &[f32],
        model: &IvfPqFastScanModel,
    ) -> Result<(u16, f32, Vec<u8>), String> {
        if source.is_empty() {
            return Err("embedding must not be empty".to_owned());
        }
        if source.len() != self.dimensions {
            return Err(format!(
                "embedding dimension mismatch: got {}, expected {}",
                source.len(),
                self.dimensions
            ));
        }
        self.validate_pq_model(model)?;
        let dimensions = u16::try_from(source.len())
            .map_err(|_| format!("embedding dimension {} exceeds maximum 65535", source.len()))?;
        let prod = ProdQuantizer::cached(
            self.dimensions,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        let rotated = rotation::srht_padded(source, &prod.signs);
        let codebook_iter = model
            .flat_codebooks
            .chunks_exact(model.group_size * GROUPED_PQ_CENTROIDS);
        let payload =
            crate::quant::grouped_pq::encode_grouped_pq(&rotated, codebook_iter, model.group_size);
        Ok((dimensions, 0.0, payload))
    }

    pub(super) fn prepare_ip_query(self, query: &[f32]) -> Result<IvfPreparedQuery, String> {
        if query.len() != self.dimensions {
            return Err(format!(
                "query dimension mismatch: got {}, expected {}",
                query.len(),
                self.dimensions
            ));
        }
        match self.profile {
            IvfQuantizerProfile::TurboQuant => {
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                match quantizer.exact_score_mode() {
                    ExactScoreMode::MseNoQjl4Bit => {
                        return Ok(IvfPreparedQuery::TurboQuantNoQjl4BitLut(
                            quantizer.prepare_ip_query_lut_no_qjl_4bit(query),
                        ));
                    }
                    ExactScoreMode::MseLutQjl
                    | ExactScoreMode::MseLutOnly
                    | ExactScoreMode::MseQjlOnly
                    | ExactScoreMode::MseScalarOnly => {}
                }
                Ok(IvfPreparedQuery::TurboQuant(
                    quantizer.prepare_ip_query(query),
                ))
            }
            IvfQuantizerProfile::RaBitQ => {
                let quantizer = self.rabitq_quantizer()?;
                Ok(IvfPreparedQuery::RaBitQ(quantizer.prepare_estimator(query)))
            }
            IvfQuantizerProfile::PqFastScan { .. } => {
                Err("ec_ivf pq_fastscan query prep requires persisted grouped codebooks".to_owned())
            }
        }
    }

    pub(super) fn prepare_ip_query_with_pq_model(
        self,
        query: &[f32],
        model: &IvfPqFastScanModel,
    ) -> Result<IvfPreparedQuery, String> {
        if query.len() != self.dimensions {
            return Err(format!(
                "query dimension mismatch: got {}, expected {}",
                query.len(),
                self.dimensions
            ));
        }
        self.validate_pq_model(model)?;
        let prod = ProdQuantizer::cached(
            self.dimensions,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        let rotated = rotation::srht_padded(query, &prod.signs);
        let transform_dim = model.group_count * model.group_size;
        let lut = build_grouped_pq_lut_f32(
            &rotated[..transform_dim],
            &model.flat_codebooks,
            model.group_size,
        );
        let suffix_max = grouped_pq_suffix_max(&lut, model.group_count);
        Ok(IvfPreparedQuery::PqFastScan {
            lut,
            group_count: model.group_count,
            suffix_max,
        })
    }

    pub(super) fn score_ip_from_parts(
        self,
        prepared_query: &IvfPreparedQuery,
        gamma: f32,
        payload: &[u8],
    ) -> Result<f32, String> {
        match (self.profile, prepared_query) {
            (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::TurboQuant(prepared_query)) => {
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                Ok(quantizer.score_ip_from_parts(prepared_query, gamma, payload))
            }
            (
                IvfQuantizerProfile::TurboQuant,
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(prepared_query),
            ) => {
                let quantizer = ProdQuantizer::cached(
                    self.dimensions,
                    crate::DEFAULT_QUANT_BITS,
                    crate::DEFAULT_QUANT_SEED,
                );
                Ok(quantizer.score_ip_from_parts_lut_no_qjl_4bit(prepared_query, payload))
            }
            (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::RaBitQ(prepared_query)) => {
                let _ = gamma;
                Ok(prepared_query.estimate_ip(payload).estimate)
            }
            (
                IvfQuantizerProfile::PqFastScan { group_count, .. },
                IvfPreparedQuery::PqFastScan {
                    lut,
                    group_count: prepared_group_count,
                    ..
                },
            ) => {
                let _ = gamma;
                if group_count != *prepared_group_count {
                    return Err("ec_ivf pq_fastscan prepared query group count mismatch".to_owned());
                }
                Ok(grouped_pq_score_f32(lut, group_count, payload))
            }
            (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::RaBitQ(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuant(_))
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::TurboQuantNoQjl4BitLut(_))
            | (IvfQuantizerProfile::TurboQuant, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::RaBitQ, IvfPreparedQuery::PqFastScan { .. })
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::TurboQuant(_))
            | (
                IvfQuantizerProfile::PqFastScan { .. },
                IvfPreparedQuery::TurboQuantNoQjl4BitLut(_),
            )
            | (IvfQuantizerProfile::PqFastScan { .. }, IvfPreparedQuery::RaBitQ(_)) => {
                Err("ec_ivf prepared query does not match quantizer profile".to_owned())
            }
        }
    }

    pub(super) fn score_ip_from_parts_with_min_bound(
        self,
        prepared_query: &IvfPreparedQuery,
        gamma: f32,
        payload: &[u8],
        min_ip_to_keep: Option<f32>,
    ) -> Result<Option<f32>, String> {
        match (self.profile, prepared_query, min_ip_to_keep) {
            (
                IvfQuantizerProfile::PqFastScan { group_count, .. },
                IvfPreparedQuery::PqFastScan {
                    lut,
                    group_count: prepared_group_count,
                    suffix_max,
                },
                Some(min_ip_to_keep),
            ) => {
                let _ = gamma;
                if group_count != *prepared_group_count {
                    return Err("ec_ivf pq_fastscan prepared query group count mismatch".to_owned());
                }
                Ok(grouped_pq_score_f32_with_min_bound(
                    lut,
                    suffix_max,
                    group_count,
                    payload,
                    min_ip_to_keep,
                ))
            }
            _ => self
                .score_ip_from_parts(prepared_query, gamma, payload)
                .map(Some),
        }
    }

    pub(super) fn payload_len(self) -> usize {
        match self.profile {
            IvfQuantizerProfile::TurboQuant => {
                crate::code_len(self.dimensions, crate::DEFAULT_QUANT_BITS)
            }
            IvfQuantizerProfile::PqFastScan { group_count, .. } => group_count.div_ceil(2),
            IvfQuantizerProfile::RaBitQ => code_len_for(self.dimensions, crate::DEFAULT_QUANT_BITS)
                .expect("default RaBitQ configuration should be valid"),
        }
    }

    pub(super) fn uses_score_bound_pruning(self) -> bool {
        matches!(self.profile, IvfQuantizerProfile::PqFastScan { .. })
    }

    fn rabitq_quantizer(self) -> Result<RaBitQQuantizer, String> {
        RaBitQQuantizer::with_seeded_srht_bits(
            self.dimensions,
            crate::DEFAULT_QUANT_SEED,
            crate::DEFAULT_QUANT_BITS,
        )
    }

    fn validate_pq_model(self, model: &IvfPqFastScanModel) -> Result<(), String> {
        match self.profile {
            IvfQuantizerProfile::PqFastScan {
                group_count,
                group_size,
            } => {
                if model.group_count != group_count || model.group_size != group_size {
                    return Err(format!(
                        "ec_ivf pq_fastscan model shape mismatch: model {}x{}, expected {}x{}",
                        model.group_count, model.group_size, group_count, group_size
                    ));
                }
                let expected = group_count * GROUPED_PQ_CENTROIDS * group_size;
                if model.flat_codebooks.len() != expected {
                    return Err(format!(
                        "ec_ivf pq_fastscan codebook length mismatch: got {}, expected {expected}",
                        model.flat_codebooks.len()
                    ));
                }
                Ok(())
            }
            _ => Err("ec_ivf pq_fastscan model used with non-pq quantizer".to_owned()),
        }
    }
}

pub(super) fn default_pq_fastscan_group_size(dimensions: usize) -> usize {
    rotation::effective_transform_dim(dimensions).min(16)
}

pub(super) fn resolve_pq_fastscan_group_size(
    dimensions: usize,
    requested_group_size: Option<usize>,
) -> Result<usize, String> {
    let transform_dim = rotation::effective_transform_dim(dimensions);
    let group_size =
        requested_group_size.unwrap_or_else(|| default_pq_fastscan_group_size(dimensions));
    if group_size == 0 {
        return Err("ec_ivf pq_fastscan pq_group_size must be greater than zero".to_owned());
    }
    if !matches!(group_size, 8 | 16 | 32) && group_size != transform_dim {
        return Err(format!(
            "ec_ivf pq_fastscan pq_group_size must be 8, 16, 32, or the full transformed dimension {transform_dim}; got {group_size}"
        ));
    }
    if group_size > transform_dim || transform_dim % group_size != 0 {
        return Err(format!(
            "ec_ivf pq_fastscan pq_group_size {group_size} must divide transformed dimension {transform_dim}"
        ));
    }
    Ok(group_size)
}

pub(super) unsafe fn load_pq_fastscan_model(
    index_relation: pgrx::pg_sys::Relation,
    metadata: &page::MetadataPage,
) -> Result<IvfPqFastScanModel, String> {
    if metadata.storage_format != StorageFormat::PqFastScan {
        return Err("ec_ivf pq_fastscan model load requires a pq_fastscan index".to_owned());
    }
    if metadata.pq_codebook_head == ItemPointer::INVALID {
        return Err("ec_ivf pq_fastscan metadata is missing a codebook head".to_owned());
    }
    if metadata.pq_group_size == 0 {
        return Err("ec_ivf pq_fastscan metadata has zero group size".to_owned());
    }
    let group_size = usize::from(metadata.pq_group_size);
    let transform_dim = rotation::effective_transform_dim(metadata.dimensions as usize);
    if transform_dim % group_size != 0 {
        return Err(format!(
            "ec_ivf pq_fastscan transform dim {transform_dim} is not divisible by group size {group_size}"
        ));
    }
    let group_count = transform_dim / group_size;
    let centroid_count = group_size * GROUPED_PQ_CENTROIDS;
    let mut flat_codebooks = Vec::with_capacity(group_count * centroid_count);
    let mut next_tid = metadata.pq_codebook_head;

    for expected_group_index in 0..group_count {
        if next_tid == ItemPointer::INVALID {
            return Err(format!(
                "ec_ivf pq_fastscan codebook chain ended at group {expected_group_index}"
            ));
        }
        let tuple =
            unsafe { page::read_ivf_pq_codebook(index_relation, next_tid, centroid_count)? };
        if usize::from(tuple.group_index) != expected_group_index {
            return Err(format!(
                "ec_ivf pq_fastscan codebook order mismatch: got {}, expected {expected_group_index}",
                tuple.group_index
            ));
        }
        flat_codebooks.extend(tuple.centroids);
        next_tid = tuple.next_tid;
    }

    if next_tid != ItemPointer::INVALID {
        return Err("ec_ivf pq_fastscan codebook chain has trailing tuples".to_owned());
    }

    Ok(IvfPqFastScanModel {
        group_count,
        group_size,
        flat_codebooks,
    })
}

impl IvfPreparedQuery {
    #[cfg(any(test, feature = "pg_test"))]
    pub(super) fn lut_len(&self) -> usize {
        match self {
            Self::TurboQuant(prepared) => prepared.lut.len(),
            Self::TurboQuantNoQjl4BitLut(prepared) => prepared.lut.len(),
            Self::PqFastScan { lut, .. } => lut.len(),
            Self::RaBitQ(_) => 0,
        }
    }

    #[cfg(any(test, feature = "pg_test"))]
    pub(super) fn sq_len(&self) -> usize {
        match self {
            Self::TurboQuant(prepared) => prepared.sq.len(),
            Self::TurboQuantNoQjl4BitLut(_) => 0,
            Self::PqFastScan { .. } => 0,
            Self::RaBitQ(_) => 0,
        }
    }
}

fn grouped_pq_suffix_max(lut: &[f32], group_count: usize) -> Vec<f32> {
    let mut suffix_max = vec![0.0_f32; group_count + 1];
    for group_index in (0..group_count).rev() {
        let row_start = group_index * GROUPED_PQ_CENTROIDS;
        let row_max = lut[row_start..row_start + GROUPED_PQ_CENTROIDS]
            .iter()
            .copied()
            .fold(f32::NEG_INFINITY, f32::max);
        suffix_max[group_index] = suffix_max[group_index + 1] + row_max;
    }
    suffix_max
}

fn grouped_pq_score_f32_with_min_bound(
    lut: &[f32],
    suffix_max: &[f32],
    group_count: usize,
    packed_nibbles: &[u8],
    min_ip_to_keep: f32,
) -> Option<f32> {
    debug_assert_eq!(suffix_max.len(), group_count + 1);
    let mut score = 0.0_f32;
    for group_index in 0..group_count {
        let centroid_index =
            crate::quant::grouped_pq::grouped_pq_nibble(packed_nibbles, group_index);
        score += lut[group_index * GROUPED_PQ_CENTROIDS + centroid_index];
        if score + suffix_max[group_index + 1] < min_ip_to_keep {
            return None;
        }
    }
    Some(score)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_vector(dimensions: usize) -> Vec<f32> {
        let mut values = (0..dimensions)
            .map(|index| (index as f32 + 1.0) / dimensions as f32)
            .collect::<Vec<_>>();
        let norm = values.iter().map(|value| value * value).sum::<f32>().sqrt();
        values.iter_mut().for_each(|value| *value /= norm);
        values
    }

    #[test]
    fn supported_v1_formats_resolve_to_turboquant() {
        let auto = IvfQuantizer::resolve(StorageFormat::Auto, 16).unwrap();
        let explicit = IvfQuantizer::resolve(StorageFormat::TurboQuant, 16).unwrap();

        assert_eq!(auto.profile, IvfQuantizerProfile::TurboQuant);
        assert_eq!(explicit.profile, IvfQuantizerProfile::TurboQuant);
    }

    #[test]
    fn rabitq_v1_format_resolves_to_rabitq() {
        let explicit = IvfQuantizer::resolve(StorageFormat::RaBitQ, 16).unwrap();

        assert_eq!(explicit.profile, IvfQuantizerProfile::RaBitQ);
    }

    #[test]
    fn pq_fastscan_v1_format_resolves_to_grouped_profile() {
        let explicit = IvfQuantizer::resolve(StorageFormat::PqFastScan, 16).unwrap();

        assert_eq!(
            explicit.profile,
            IvfQuantizerProfile::PqFastScan {
                group_count: 1,
                group_size: 16
            }
        );
    }

    #[test]
    fn pq_fastscan_accepts_metadata_group_size_override() {
        let explicit =
            IvfQuantizer::resolve_with_pq_group_size(StorageFormat::PqFastScan, 64, Some(8))
                .unwrap();

        assert_eq!(
            explicit.profile,
            IvfQuantizerProfile::PqFastScan {
                group_count: 8,
                group_size: 8
            }
        );
        assert_eq!(explicit.payload_len(), 4);
    }

    #[test]
    fn pq_fastscan_rejects_group_size_that_does_not_divide_transform() {
        let err = IvfQuantizer::resolve_with_pq_group_size(StorageFormat::PqFastScan, 64, Some(12))
            .unwrap_err();

        assert!(err.contains("pq_group_size"));
        assert!(err.contains("must be 8, 16, 32"));
    }

    #[test]
    fn turboquant_dispatch_matches_direct_prod_score() {
        let dimensions = 32;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let (_, gamma, payload) = dispatch.encode_source(&source).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();

        let direct = ProdQuantizer::cached(
            dimensions,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        let direct_prepared = direct.prepare_ip_query(&query);

        assert_eq!(
            dispatch
                .score_ip_from_parts(&prepared, gamma, &payload)
                .unwrap(),
            direct.score_ip_from_parts(&direct_prepared, gamma, &payload)
        );
    }

    #[test]
    fn turboquant_dispatch_uses_lut_for_no_qjl_4bit_lane() {
        let dimensions = 1536;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::TurboQuant, dimensions).unwrap();
        let (_, gamma, payload) = dispatch.encode_source(&source).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();

        assert_eq!(prepared.lut_len(), dimensions * 16);
        assert_eq!(prepared.sq_len(), 0);

        let direct = ProdQuantizer::cached(
            dimensions,
            crate::DEFAULT_QUANT_BITS,
            crate::DEFAULT_QUANT_SEED,
        );
        let direct_prepared = direct.prepare_ip_query_lut_no_qjl_4bit(&query);

        assert_eq!(
            dispatch
                .score_ip_from_parts(&prepared, gamma, &payload)
                .unwrap(),
            direct.score_ip_from_parts_lut_no_qjl_4bit(&direct_prepared, &payload)
        );
    }

    #[test]
    fn rabitq_dispatch_matches_direct_quantizer_score() {
        let dimensions = 32;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::RaBitQ, dimensions).unwrap();
        let (_, gamma, payload) = dispatch.encode_source(&source).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();

        let direct = RaBitQQuantizer::with_seeded_srht_bits(
            dimensions,
            crate::DEFAULT_QUANT_SEED,
            crate::DEFAULT_QUANT_BITS,
        )
        .unwrap();
        let direct_prepared = direct.prepare_estimator(&query);

        assert_eq!(gamma, 0.0);
        assert_eq!(payload.len(), direct.code_len());
        assert_eq!(
            dispatch
                .score_ip_from_parts(&prepared, gamma, &payload)
                .unwrap(),
            direct.estimate_ip(&direct_prepared, &payload).estimate
        );
    }

    #[test]
    fn rabitq_dispatch_does_not_rebuild_quantizer_while_scoring() {
        let dimensions = 40;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let dispatch = IvfQuantizer::resolve(StorageFormat::RaBitQ, dimensions).unwrap();

        crate::quant::rabitq::reset_seeded_srht_construction_count_for_test(dimensions);
        let (_, gamma, payload) = dispatch.encode_source(&source).unwrap();
        let prepared = dispatch.prepare_ip_query(&query).unwrap();
        let after_prepare = crate::quant::rabitq::seeded_srht_construction_count_for_test();

        assert_eq!(after_prepare, 2);
        for _ in 0..8 {
            let _ = dispatch
                .score_ip_from_parts(&prepared, gamma, &payload)
                .unwrap();
        }
        assert_eq!(
            crate::quant::rabitq::seeded_srht_construction_count_for_test(),
            after_prepare
        );
    }

    #[test]
    fn pq_fastscan_dispatch_scores_grouped_code_with_persisted_model() {
        let dimensions = 16;
        let source = unit_vector(dimensions);
        let query = unit_vector(dimensions);
        let training_rows = [
            unit_vector(dimensions),
            unit_vector(dimensions),
            (0..dimensions)
                .map(|index| if index % 2 == 0 { 0.25 } else { -0.25 })
                .collect::<Vec<_>>(),
            (0..dimensions)
                .map(|index| if index % 2 == 0 { -0.25 } else { 0.25 })
                .collect::<Vec<_>>(),
        ];
        let training_refs = training_rows.iter().map(Vec::as_slice).collect::<Vec<_>>();
        let trained = crate::am::common::training::train_grouped_pq4_model(
            &training_refs,
            dimensions,
            crate::DEFAULT_QUANT_SEED,
            default_pq_fastscan_group_size(dimensions),
            training_refs.len(),
            3,
        )
        .unwrap();
        let model = IvfPqFastScanModel {
            group_count: trained.group_count,
            group_size: trained.group_size,
            flat_codebooks: trained.codebooks.into_iter().flatten().collect(),
        };
        let dispatch = IvfQuantizer::resolve(StorageFormat::PqFastScan, dimensions).unwrap();
        let (_, gamma, payload) = dispatch
            .encode_source_with_pq_model(&source, &model)
            .unwrap();
        let prepared = dispatch
            .prepare_ip_query_with_pq_model(&query, &model)
            .unwrap();
        let score = dispatch
            .score_ip_from_parts(&prepared, gamma, &payload)
            .unwrap();
        let low_bound_score = dispatch
            .score_ip_from_parts_with_min_bound(&prepared, gamma, &payload, Some(score - 1.0))
            .unwrap();
        let high_bound_score = dispatch
            .score_ip_from_parts_with_min_bound(&prepared, gamma, &payload, Some(score + 1.0))
            .unwrap();

        let IvfPreparedQuery::PqFastScan {
            lut,
            group_count,
            suffix_max,
        } = prepared
        else {
            panic!("expected pq_fastscan prepared query");
        };
        assert_eq!(gamma, 0.0);
        assert_eq!(payload.len(), model.group_count.div_ceil(2));
        assert_eq!(suffix_max.len(), model.group_count + 1);
        assert_eq!(score, grouped_pq_score_f32(&lut, group_count, &payload));
        assert_eq!(low_bound_score, Some(score));
        assert_eq!(high_bound_score, None);
    }

    #[test]
    fn grouped_pq_score_bound_prunes_when_suffix_cannot_reach_minimum() {
        let group_count = 2;
        let mut lut = vec![0.0_f32; group_count * GROUPED_PQ_CENTROIDS];
        lut[1] = 0.25;
        lut[GROUPED_PQ_CENTROIDS + 2] = 0.5;
        lut[GROUPED_PQ_CENTROIDS + 3] = 2.0;
        let suffix_max = grouped_pq_suffix_max(&lut, group_count);
        let payload = crate::quant::grouped_pq::pack_grouped_pq_nibbles(&[1, 2]);

        assert_eq!(
            grouped_pq_score_f32_with_min_bound(&lut, &suffix_max, group_count, &payload, 0.7),
            Some(0.75)
        );
        assert_eq!(
            grouped_pq_score_f32_with_min_bound(&lut, &suffix_max, group_count, &payload, 0.8),
            None
        );
    }
}
