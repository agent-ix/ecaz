use super::candidate_batch::{CandidateBatch, CandidatePayload};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QuantCodecKind {
    TurboQuant,
    RaBitQ,
    GroupedPq,
    Binary,
}

impl QuantCodecKind {
    pub(crate) const ALL: [Self; 4] = [
        Self::TurboQuant,
        Self::RaBitQ,
        Self::GroupedPq,
        Self::Binary,
    ];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::TurboQuant => "turboquant",
            Self::RaBitQ => "rabitq",
            Self::GroupedPq => "grouped_pq",
            Self::Binary => "binary",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QuantSearchCodecTag {
    TurboQuant,
    RaBitQ {
        bits: u8,
    },
    GroupedPq {
        group_count: usize,
        group_size: usize,
    },
    Binary,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct EncodedQuantPayload {
    pub(crate) dimensions: u16,
    pub(crate) gamma: f32,
    pub(crate) code: Vec<u8>,
}

pub(crate) trait QuantCodec {
    type PreparedQuery;

    fn codec_kind(&self) -> QuantCodecKind;

    fn search_codec_tag(&self) -> QuantSearchCodecTag;

    fn payload_len(&self) -> usize;

    fn encode_source(&self, source: &[f32]) -> Result<EncodedQuantPayload, String>;

    fn prepare_ip_query(&self, query: &[f32]) -> Result<Self::PreparedQuery, String>;

    fn score_ip_candidate(
        &self,
        prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>,
    ) -> Result<f32, String>;

    fn try_score_ip_candidate(
        &self,
        prepared_query: &Self::PreparedQuery,
        payload: CandidatePayload<'_>,
        min_ip_to_keep: Option<f32>,
    ) -> Result<Option<f32>, String> {
        let _ = min_ip_to_keep;
        self.score_ip_candidate(prepared_query, payload).map(Some)
    }

    fn score_ip_batch<Id>(
        &self,
        prepared_query: &Self::PreparedQuery,
        batch: &CandidateBatch<'_, Id>,
        out_scores: &mut [f32],
    ) -> Result<(), String> {
        if batch.len() != out_scores.len() {
            return Err(format!(
                "quant codec batch output count {} does not match candidate count {}",
                out_scores.len(),
                batch.len()
            ));
        }

        for (payload, out_score) in batch.payloads().iter().zip(out_scores.iter_mut()) {
            *out_score = self.score_ip_candidate(prepared_query, *payload)?;
        }
        Ok(())
    }
}
