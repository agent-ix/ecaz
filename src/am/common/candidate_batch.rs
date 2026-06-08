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

    for (payload, out_score) in batch.payloads().iter().zip(out_scores.iter_mut()) {
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
}
