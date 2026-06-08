use crate::quant::prod::{PreparedLutNoQjl4BitQuery, ProdQuantizer};

#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(dead_code)]
pub(crate) enum CandidateMeta<'a> {
    None,
    Gamma(f32),
    GammaAndResidualSigns { gamma: f32, signs: &'a [u8] },
    Binary,
    RaBitQ,
    GroupedPq { group_count: usize },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct CandidatePayload<'a, Meta = CandidateMeta<'a>> {
    pub(crate) code: &'a [u8],
    pub(crate) meta: Meta,
}

impl<'a, Meta> CandidatePayload<'a, Meta> {
    pub(crate) fn new(code: &'a [u8], meta: Meta) -> Self {
        Self { code, meta }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct CandidateBatch<'a, Id, Meta = CandidateMeta<'a>> {
    ids: Vec<Id>,
    payloads: Vec<CandidatePayload<'a, Meta>>,
    capacity: usize,
}

impl<'a, Id, Meta> CandidateBatch<'a, Id, Meta> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self {
            ids: Vec::with_capacity(capacity),
            payloads: Vec::with_capacity(capacity),
            capacity,
        }
    }

    pub(crate) fn push(
        &mut self,
        id: Id,
        payload: CandidatePayload<'a, Meta>,
    ) -> Result<(), String> {
        if self.ids.len() >= self.capacity {
            return Err(format!(
                "candidate batch capacity {} exceeded",
                self.capacity
            ));
        }
        self.ids.push(id);
        self.payloads.push(payload);
        Ok(())
    }

    pub(crate) fn len(&self) -> usize {
        self.ids.len()
    }

    #[allow(dead_code)]
    pub(crate) fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    #[allow(dead_code)]
    pub(crate) fn ids(&self) -> &[Id] {
        &self.ids
    }

    pub(crate) fn payloads(&self) -> &[CandidatePayload<'a, Meta>] {
        &self.payloads
    }
}

pub(crate) fn score_turboquant_no_qjl_4bit_batch<Id>(
    quantizer: &ProdQuantizer,
    prepared: &PreparedLutNoQjl4BitQuery,
    batch: &CandidateBatch<'_, Id>,
    out_scores: &mut [f32],
) -> Result<(), String> {
    if batch.len() != out_scores.len() {
        return Err(format!(
            "candidate batch score output count {} does not match candidate count {}",
            out_scores.len(),
            batch.len()
        ));
    }

    for payload in batch.payloads() {
        match payload.meta {
            CandidateMeta::None | CandidateMeta::Gamma(_) => {}
            CandidateMeta::GammaAndResidualSigns { .. }
            | CandidateMeta::Binary
            | CandidateMeta::RaBitQ
            | CandidateMeta::GroupedPq { .. } => {
                return Err(
                    "TurboQuant no-QJL 4-bit batch received incompatible candidate metadata"
                        .to_owned(),
                );
            }
        }
    }

    if batch.len() >= crate::quant::lut32::BLOCK_WIDTH {
        let mse_codes: Vec<&[u8]> = batch
            .payloads()
            .iter()
            .map(|payload| quantizer.mse_code_bytes_no_qjl_4bit(payload.code))
            .collect();
        return crate::quant::lut32::score_lut_no_qjl_4bit_batch(
            &prepared.lut,
            quantizer.original_dim,
            &mse_codes,
            out_scores,
        );
    }

    for (payload, out_score) in batch.payloads().iter().zip(out_scores.iter_mut()) {
        *out_score = quantizer.score_ip_from_parts_lut_no_qjl_4bit(prepared, payload.code);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{CandidateBatch, CandidateMeta, CandidatePayload};

    #[test]
    fn candidate_batch_preserves_ids_and_payloads() {
        let first = [1_u8, 2, 3];
        let second = [4_u8, 5, 6];
        let mut batch = CandidateBatch::with_capacity(2);

        batch
            .push(10_u32, CandidatePayload::new(&first, CandidateMeta::None))
            .unwrap();
        batch
            .push(
                11_u32,
                CandidatePayload::new(&second, CandidateMeta::Gamma(1.25)),
            )
            .unwrap();

        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
        assert_eq!(batch.ids(), &[10, 11]);
        assert_eq!(batch.payloads()[0].code, &first);
        assert_eq!(batch.payloads()[1].meta, CandidateMeta::Gamma(1.25));
    }

    #[test]
    fn candidate_batch_rejects_capacity_overflow() {
        let code = [1_u8];
        let mut batch = CandidateBatch::with_capacity(1);

        batch
            .push(0_u32, CandidatePayload::new(&code, CandidateMeta::None))
            .unwrap();

        assert!(batch
            .push(1_u32, CandidatePayload::new(&code, CandidateMeta::None))
            .is_err());
    }

    #[test]
    fn turboquant_lut_batch_matches_scalar_tail() {
        let quantizer = crate::quant::prod::ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 31);
        let prepared = quantizer.prepare_ip_query_lut_no_qjl_4bit(&query);
        let encoded: Vec<_> = (0..39)
            .map(|seed| quantizer.encode(&random_unit_vector(1536, seed)).mse_packed)
            .collect();
        let mut batch = CandidateBatch::with_capacity(encoded.len());
        for (index, payload) in encoded.iter().enumerate() {
            batch
                .push(index, CandidatePayload::new(payload, CandidateMeta::None))
                .unwrap();
        }
        let mut batch_scores = vec![0.0; batch.len()];

        super::score_turboquant_no_qjl_4bit_batch(&quantizer, &prepared, &batch, &mut batch_scores)
            .unwrap();

        for (payload, score) in encoded.iter().zip(batch_scores.iter()) {
            let scalar = quantizer.score_ip_from_parts_lut_no_qjl_4bit(&prepared, payload);
            assert_eq!(score.to_bits(), scalar.to_bits());
        }
    }

    fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
        let mut state = seed ^ 0xA076_1D64_78BD_642F;
        let mut values = Vec::with_capacity(dim);
        let mut norm_sq = 0.0_f32;
        for _ in 0..dim {
            state = state
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(0xBF58_476D_1CE4_E5B9);
            let raw = ((state >> 32) as u32) as f32 / (u32::MAX as f32);
            let value = raw * 2.0 - 1.0;
            values.push(value);
            norm_sq += value * value;
        }
        let norm = norm_sq.sqrt().max(f32::MIN_POSITIVE);
        for value in &mut values {
            *value /= norm;
        }
        values
    }
}
