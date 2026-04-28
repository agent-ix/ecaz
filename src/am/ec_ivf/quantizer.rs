use super::options::StorageFormat;
use crate::quant::prod::{PreparedLutNoQjl4BitQuery, PreparedQuery, ProdQuantizer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum IvfQuantizerProfile {
    TurboQuant,
}

pub(super) enum IvfPreparedQuery {
    TurboQuant(PreparedQuery),
    TurboQuantNoQjl4BitLut(PreparedLutNoQjl4BitQuery),
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
        storage_format.validate_v1_supported()?;
        let profile = match storage_format {
            StorageFormat::Auto | StorageFormat::TurboQuant => IvfQuantizerProfile::TurboQuant,
            StorageFormat::PqFastScan | StorageFormat::RaBitQ => {
                unreachable!("validate_v1_supported rejects unsupported IVF storage formats")
            }
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
        }
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
                if quantizer.exact_score_mode_name() == "mse_no_qjl_4bit" {
                    return Ok(IvfPreparedQuery::TurboQuantNoQjl4BitLut(
                        quantizer.prepare_ip_query_lut_no_qjl_4bit(query),
                    ));
                }
                Ok(IvfPreparedQuery::TurboQuant(
                    quantizer.prepare_ip_query(query),
                ))
            }
        }
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
        }
    }

    pub(super) fn payload_len(self) -> usize {
        match self.profile {
            IvfQuantizerProfile::TurboQuant => {
                crate::code_len(self.dimensions, crate::DEFAULT_QUANT_BITS)
            }
        }
    }
}

impl IvfPreparedQuery {
    #[cfg(any(test, feature = "pg_test"))]
    pub(super) fn lut_len(&self) -> usize {
        match self {
            Self::TurboQuant(prepared) => prepared.lut.len(),
            Self::TurboQuantNoQjl4BitLut(prepared) => prepared.lut.len(),
        }
    }

    #[cfg(any(test, feature = "pg_test"))]
    pub(super) fn sq_len(&self) -> usize {
        match self {
            Self::TurboQuant(prepared) => prepared.sq.len(),
            Self::TurboQuantNoQjl4BitLut(_) => 0,
        }
    }
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
    fn unsupported_v1_formats_are_rejected_at_dispatch() {
        assert!(IvfQuantizer::resolve(StorageFormat::PqFastScan, 16).is_err());
        assert!(IvfQuantizer::resolve(StorageFormat::RaBitQ, 16).is_err());
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
}
