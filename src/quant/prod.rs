//! Product quantizer orchestrator.

use std::collections::HashMap;
use std::f32::consts::PI;
use std::sync::{Arc, Mutex, OnceLock};

use crate::quant::codebook;
use crate::quant::mse;
use crate::quant::qjl;
use crate::quant::rotation;
use crate::quant::simd::{backend, SimdBackend};
use crate::quant::CodeIndex;

#[derive(Debug, Clone, PartialEq)]
pub struct EncodedTq {
    pub gamma: f32,
    pub mse_packed: Vec<u8>,
    pub qjl_packed: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedQuery {
    pub lut: Vec<f32>,
    pub rotated: Vec<f32>,
    pub sq: Vec<f32>,
    pub qjl_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Int8ApproxNoQjl4BitQuery {
    pub codebook: [i8; 16],
    pub rotated: Vec<i8>,
    pub score_scale: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedLutNoQjl4BitQuery {
    pub lut: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedTiledLutNoQjl4BitQuery {
    pub lut: Vec<f32>,
    pub tile_size: usize,
}

pub use crate::quant::rabitq::BinarySignNoQjl4BitQuery;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExactScoreMode {
    MseNoQjl4Bit,
    MseLutQjl,
    MseLutOnly,
    MseQjlOnly,
    MseScalarOnly,
}

#[derive(Debug)]
pub struct ProdQuantizer {
    pub transform_dim: usize,
    pub original_dim: usize,
    pub bits: u8,
    #[allow(dead_code)]
    pub seed: u64,
    pub codebook: Vec<f32>,
    pub signs: Vec<f32>,
    qjl_signs: Vec<f32>,
}

type QuantizerKey = (usize, u8, u64);

fn cache() -> &'static Mutex<HashMap<QuantizerKey, Arc<ProdQuantizer>>> {
    static CACHE: OnceLock<Mutex<HashMap<QuantizerKey, Arc<ProdQuantizer>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

impl ProdQuantizer {
    pub fn new(dim: usize, bits: u8, seed: u64) -> Self {
        assert!(dim > 0, "dimension must be positive");
        assert!((2..=8).contains(&bits), "bits must be within 2..=8");

        let transform_dim = rotation::effective_transform_dim(dim);
        let qjl_active = qjl_enabled(dim, bits);
        let codebook = codebook::lloyd_max(mse_bits(dim, bits) as usize, dim, 20_000)
            .into_iter()
            .map(|value| value as f32)
            .collect();
        let signs = rotation::sign_vector(transform_dim, seed);
        let qjl_signs = if qjl_active {
            rotation::sign_vector(transform_dim, seed ^ 0x9E37_79B9_7F4A_7C15)
        } else {
            Vec::new()
        };

        Self {
            transform_dim,
            original_dim: dim,
            bits,
            seed,
            codebook,
            signs,
            qjl_signs,
        }
    }

    pub fn cached(dim: usize, bits: u8, seed: u64) -> Arc<Self> {
        let key = (dim, bits, seed);
        let mut guard = cache().lock().expect("quantizer cache poisoned");
        guard
            .entry(key)
            .or_insert_with(|| Arc::new(Self::new(dim, bits, seed)))
            .clone()
    }

    pub fn cached_with_presence(dim: usize, bits: u8, seed: u64) -> (Arc<Self>, bool) {
        let key = (dim, bits, seed);
        let mut guard = cache().lock().expect("quantizer cache poisoned");
        let was_present = guard.contains_key(&key);
        let quantizer = guard
            .entry(key)
            .or_insert_with(|| Arc::new(Self::new(dim, bits, seed)))
            .clone();
        (quantizer, was_present)
    }

    pub fn encode(&self, vector: &[f32]) -> EncodedTq {
        assert_eq!(
            vector.len(),
            self.original_dim,
            "vector length mismatch: got {}, expected {}",
            vector.len(),
            self.original_dim
        );

        let rotated = rotation::srht_padded(vector, &self.signs);
        let mse_indices = mse::quantize_to_indices(&self.codebook, &rotated, self.original_dim);
        let mse_values = mse::decode_indices(&self.codebook, &mse_indices);

        let mut rotated_domain = vec![0.0_f32; self.transform_dim];
        rotated_domain[..self.original_dim].copy_from_slice(&mse_values);
        let decoded_mse = qjl::decode_mse_only(&rotated_domain, &self.signs, self.original_dim);

        let residual: Vec<f32> = vector
            .iter()
            .zip(decoded_mse.iter())
            .map(|(input, approx)| input - approx)
            .collect();
        let gamma = residual
            .iter()
            .map(|value| value * value)
            .sum::<f32>()
            .sqrt();
        let qjl_packed = if qjl_enabled(self.original_dim, self.bits) {
            let qjl_projection = qjl::qjl_project(&residual, &self.qjl_signs);
            let qjl_signs = qjl_projection[..self.original_dim]
                .iter()
                .map(|value| *value >= 0.0)
                .collect::<Vec<_>>();
            pack_qjl_signs(&qjl_signs)
        } else {
            Vec::new()
        };

        EncodedTq {
            gamma,
            mse_packed: pack_mse_indices(&mse_indices, mse_bits(self.original_dim, self.bits)),
            qjl_packed,
        }
    }

    #[allow(dead_code)]
    pub fn decode_approximate(&self, payload: &[u8]) -> Vec<f32> {
        let (_, mse_packed, _) = self.split_payload(payload);
        let mse_indices = unpack_mse_indices(
            mse_packed,
            self.original_dim,
            mse_bits(self.original_dim, self.bits),
        );
        let mse_values = mse::decode_indices(&self.codebook, &mse_indices);

        let mut rotated_domain = vec![0.0_f32; self.transform_dim];
        rotated_domain[..self.original_dim].copy_from_slice(&mse_values);
        qjl::decode_mse_only(&rotated_domain, &self.signs, self.original_dim)
    }

    pub fn prepare_ip_query(&self, query: &[f32]) -> PreparedQuery {
        assert_eq!(
            query.len(),
            self.original_dim,
            "query length mismatch: got {}, expected {}",
            query.len(),
            self.original_dim
        );

        let qjl_active = qjl_enabled(self.original_dim, self.bits);
        let mut rotated = rotation::srht_padded(query, &self.signs);
        let mut qjl_projection = qjl_active.then(|| qjl::qjl_project(query, &self.qjl_signs));
        let num_centroids = 1usize << mse_bits(self.original_dim, self.bits);

        let lut = if prepared_query_uses_lut(self.original_dim, self.bits) {
            build_prepared_query_lut(&rotated[..self.original_dim], &self.codebook, num_centroids)
        } else {
            Vec::new()
        };
        rotated.truncate(self.original_dim);
        if let Some(projection) = qjl_projection.as_mut() {
            projection.truncate(self.original_dim);
        }

        PreparedQuery {
            lut,
            rotated,
            sq: qjl_projection.unwrap_or_default(),
            qjl_scale: if qjl_active {
                (PI / 2.0).sqrt() / self.original_dim as f32
            } else {
                0.0
            },
        }
    }

    pub fn score_ip_encoded(&self, prepared: &PreparedQuery, payload: &[u8]) -> f32 {
        let (gamma, mse_packed, qjl_packed) = self.split_payload(payload);
        self.score_ip_from_split_parts(prepared, gamma, mse_packed, qjl_packed)
    }

    pub fn score_ip_from_parts(
        &self,
        prepared: &PreparedQuery,
        gamma: f32,
        code_bytes: &[u8],
    ) -> f32 {
        let (mse_packed, qjl_packed) = self.split_code_bytes(code_bytes);
        self.score_ip_from_split_parts(prepared, gamma, mse_packed, qjl_packed)
    }

    pub fn prepare_ip_query_int8_approx_no_qjl_4bit(
        &self,
        query: &[f32],
    ) -> Int8ApproxNoQjl4BitQuery {
        assert_eq!(
            query.len(),
            self.original_dim,
            "query length mismatch: got {}, expected {}",
            query.len(),
            self.original_dim
        );
        assert!(
            self.bits == 4 && !qjl_enabled(self.original_dim, self.bits),
            "int8 approximate query prep requires the no-QJL 4-bit lane"
        );

        let rotated = rotation::srht_padded(query, &self.signs);
        let rotated = &rotated[..self.original_dim];
        let (rotated_i8, rotated_scale) = quantize_i8(rotated);
        let (codebook_i8, codebook_scale) = quantize_codebook_i8_16(&self.codebook);

        Int8ApproxNoQjl4BitQuery {
            codebook: codebook_i8,
            rotated: rotated_i8,
            score_scale: rotated_scale * codebook_scale,
        }
    }

    pub fn prepare_ip_query_lut_no_qjl_4bit(&self, query: &[f32]) -> PreparedLutNoQjl4BitQuery {
        assert_eq!(
            query.len(),
            self.original_dim,
            "query length mismatch: got {}, expected {}",
            query.len(),
            self.original_dim
        );
        assert!(
            self.bits == 4 && !qjl_enabled(self.original_dim, self.bits),
            "explicit LUT query prep requires the no-QJL 4-bit lane"
        );

        let rotated = rotation::srht_padded(query, &self.signs);
        PreparedLutNoQjl4BitQuery {
            lut: build_prepared_query_lut(&rotated[..self.original_dim], &self.codebook, 16),
        }
    }

    pub fn prepare_ip_query_tiled_lut_no_qjl_4bit(
        &self,
        query: &[f32],
        tile_size: usize,
    ) -> PreparedTiledLutNoQjl4BitQuery {
        assert!(
            tile_size > 0,
            "tiled LUT query prep requires a positive tile size"
        );
        let prepared = self.prepare_ip_query_lut_no_qjl_4bit(query);
        PreparedTiledLutNoQjl4BitQuery {
            lut: prepared.lut,
            tile_size,
        }
    }

    pub fn binary_sign_no_qjl_4bit_supported(&self) -> bool {
        self.bits == 4 && !qjl_enabled(self.original_dim, self.bits)
    }

    pub fn int8_approx_no_qjl_4bit_supported(&self) -> bool {
        self.bits == 4 && !qjl_enabled(self.original_dim, self.bits)
    }

    pub fn exact_score_uses_lut(&self) -> bool {
        prepared_query_uses_lut(self.original_dim, self.bits)
    }

    pub fn exact_score_uses_qjl(&self) -> bool {
        qjl_enabled(self.original_dim, self.bits)
    }

    pub fn exact_score_mode(&self) -> ExactScoreMode {
        if self.bits == 4 && !qjl_enabled(self.original_dim, self.bits) {
            ExactScoreMode::MseNoQjl4Bit
        } else if self.exact_score_uses_lut() && self.exact_score_uses_qjl() {
            ExactScoreMode::MseLutQjl
        } else if self.exact_score_uses_lut() {
            ExactScoreMode::MseLutOnly
        } else if self.exact_score_uses_qjl() {
            ExactScoreMode::MseQjlOnly
        } else {
            ExactScoreMode::MseScalarOnly
        }
    }

    pub fn exact_score_mode_name(&self) -> &'static str {
        match self.exact_score_mode() {
            ExactScoreMode::MseNoQjl4Bit => "mse_no_qjl_4bit",
            ExactScoreMode::MseLutQjl => "mse_lut_qjl",
            ExactScoreMode::MseLutOnly => "mse_lut_only",
            ExactScoreMode::MseQjlOnly => "mse_qjl_only",
            ExactScoreMode::MseScalarOnly => "mse_scalar_only",
        }
    }

    pub fn prepare_ip_query_binary_sign_no_qjl_4bit(
        &self,
        query: &[f32],
    ) -> BinarySignNoQjl4BitQuery {
        assert_eq!(
            query.len(),
            self.original_dim,
            "query length mismatch: got {}, expected {}",
            query.len(),
            self.original_dim
        );
        assert!(
            self.binary_sign_no_qjl_4bit_supported(),
            "binary sign query prep requires the no-QJL 4-bit lane"
        );

        let rotated = rotation::srht_padded(query, &self.signs);
        BinarySignNoQjl4BitQuery {
            words: crate::quant::rabitq::sign_words_from_rotated(&rotated[..self.original_dim]),
        }
    }

    pub fn binary_sign_words_from_packed_no_qjl_4bit(&self, code_bytes: &[u8]) -> Vec<u64> {
        assert!(
            self.binary_sign_no_qjl_4bit_supported(),
            "binary sign code derivation requires the no-QJL 4-bit lane"
        );

        crate::quant::rabitq::sign_words_from_packed_4bit(
            code_bytes,
            self.original_dim,
            &crate::quant::rabitq::binary_sign_lookup_4bit(&self.codebook),
        )
    }

    pub fn score_binary_sign_words_no_qjl_4bit(
        &self,
        prepared: &BinarySignNoQjl4BitQuery,
        candidate_words: &[u64],
    ) -> f32 {
        assert!(
            self.binary_sign_no_qjl_4bit_supported(),
            "binary sign scoring requires the no-QJL 4-bit lane"
        );
        crate::quant::rabitq::hamming_similarity(
            &prepared.words,
            candidate_words,
            self.original_dim,
        )
    }

    pub fn score_ip_from_parts_int8_approx_no_qjl_4bit(
        &self,
        prepared: &Int8ApproxNoQjl4BitQuery,
        code_bytes: &[u8],
    ) -> f32 {
        assert!(
            self.bits == 4 && !qjl_enabled(self.original_dim, self.bits),
            "int8 approximate scoring requires the no-QJL 4-bit lane"
        );
        let (mse_packed, qjl_packed) = self.split_code_bytes(code_bytes);
        debug_assert!(qjl_packed.is_empty());
        self.score_ip_from_split_parts_int8_approx_no_qjl_4bit(prepared, mse_packed)
    }

    pub fn score_ip_from_parts_lut_no_qjl_4bit(
        &self,
        prepared: &PreparedLutNoQjl4BitQuery,
        code_bytes: &[u8],
    ) -> f32 {
        assert!(
            self.bits == 4 && !qjl_enabled(self.original_dim, self.bits),
            "explicit LUT scoring requires the no-QJL 4-bit lane"
        );
        let (mse_packed, qjl_packed) = self.split_code_bytes(code_bytes);
        debug_assert!(qjl_packed.is_empty());
        self.score_ip_from_split_parts_lut_no_qjl_4bit(&prepared.lut, mse_packed)
    }

    pub fn score_ip_from_parts_tiled_lut_no_qjl_4bit(
        &self,
        prepared: &PreparedTiledLutNoQjl4BitQuery,
        code_bytes: &[u8],
    ) -> f32 {
        assert!(
            self.bits == 4 && !qjl_enabled(self.original_dim, self.bits),
            "tiled LUT scoring requires the no-QJL 4-bit lane"
        );
        let (mse_packed, qjl_packed) = self.split_code_bytes(code_bytes);
        debug_assert!(qjl_packed.is_empty());
        self.score_ip_from_split_parts_tiled_lut_no_qjl_4bit(
            &prepared.lut,
            prepared.tile_size,
            mse_packed,
        )
    }

    fn score_ip_from_split_parts(
        &self,
        prepared: &PreparedQuery,
        gamma: f32,
        mse_packed: &[u8],
        qjl_packed: &[u8],
    ) -> f32 {
        if !qjl_enabled(self.original_dim, self.bits) {
            if self.bits == 4 {
                return self.score_ip_from_split_parts_no_qjl_4bit(prepared, mse_packed);
            }

            return self.score_ip_from_split_parts_scalar(prepared, gamma, mse_packed, qjl_packed);
        }

        match backend() {
            #[cfg(target_arch = "x86_64")]
            SimdBackend::Avx2Fma => unsafe {
                self.score_ip_from_split_parts_avx2(prepared, gamma, mse_packed, qjl_packed)
            },
            #[cfg(target_arch = "aarch64")]
            SimdBackend::Neon => unsafe {
                self.score_ip_from_split_parts_neon(prepared, gamma, mse_packed, qjl_packed)
            },
            SimdBackend::Scalar => {
                self.score_ip_from_split_parts_scalar(prepared, gamma, mse_packed, qjl_packed)
            }
        }
    }

    fn score_ip_from_split_parts_no_qjl_4bit(
        &self,
        prepared: &PreparedQuery,
        mse_packed: &[u8],
    ) -> f32 {
        debug_assert_eq!(self.bits, 4);
        debug_assert!(!qjl_enabled(self.original_dim, self.bits));

        let mut sum = 0.0_f32;
        let mut dim_index = 0usize;

        for &packed in mse_packed {
            if dim_index >= self.original_dim {
                break;
            }

            let low_nibble = (packed & 0x0F) as usize;
            sum += self.codebook[low_nibble] * prepared.rotated[dim_index];
            dim_index += 1;

            if dim_index >= self.original_dim {
                break;
            }

            let high_nibble = (packed >> 4) as usize;
            sum += self.codebook[high_nibble] * prepared.rotated[dim_index];
            dim_index += 1;
        }

        sum
    }

    fn score_ip_from_split_parts_int8_approx_no_qjl_4bit(
        &self,
        prepared: &Int8ApproxNoQjl4BitQuery,
        mse_packed: &[u8],
    ) -> f32 {
        debug_assert_eq!(self.bits, 4);
        debug_assert!(!qjl_enabled(self.original_dim, self.bits));

        if prepared.score_scale == 0.0 {
            return 0.0;
        }

        let mut sum = 0_i32;
        let mut dim_index = 0usize;

        for &packed in mse_packed {
            if dim_index >= self.original_dim {
                break;
            }

            let low_nibble = (packed & 0x0F) as usize;
            sum += prepared.codebook[low_nibble] as i32 * prepared.rotated[dim_index] as i32;
            dim_index += 1;

            if dim_index >= self.original_dim {
                break;
            }

            let high_nibble = (packed >> 4) as usize;
            sum += prepared.codebook[high_nibble] as i32 * prepared.rotated[dim_index] as i32;
            dim_index += 1;
        }

        sum as f32 * prepared.score_scale
    }

    fn score_ip_from_split_parts_lut_no_qjl_4bit(&self, lut: &[f32], mse_packed: &[u8]) -> f32 {
        debug_assert_eq!(self.bits, 4);
        debug_assert!(!qjl_enabled(self.original_dim, self.bits));
        debug_assert_eq!(lut.len(), self.original_dim * 16);

        let mut sum = 0.0_f32;
        let mut dim_index = 0usize;

        for &packed in mse_packed {
            if dim_index >= self.original_dim {
                break;
            }

            let low_nibble = (packed & 0x0F) as usize;
            sum += lut[dim_index * 16 + low_nibble];
            dim_index += 1;

            if dim_index >= self.original_dim {
                break;
            }

            let high_nibble = (packed >> 4) as usize;
            sum += lut[dim_index * 16 + high_nibble];
            dim_index += 1;
        }

        sum
    }

    fn score_ip_from_split_parts_tiled_lut_no_qjl_4bit(
        &self,
        lut: &[f32],
        tile_size: usize,
        mse_packed: &[u8],
    ) -> f32 {
        debug_assert_eq!(self.bits, 4);
        debug_assert!(!qjl_enabled(self.original_dim, self.bits));
        debug_assert_eq!(lut.len(), self.original_dim * 16);
        assert!(
            tile_size > 0,
            "tiled LUT scoring requires a positive tile size"
        );

        let mut sum = 0.0_f32;
        let mut tile_start = 0usize;

        while tile_start < self.original_dim {
            let tile_end = (tile_start + tile_size).min(self.original_dim);
            let tile_lut = &lut[tile_start * 16..tile_end * 16];

            for dim_index in tile_start..tile_end {
                let centroid_index = mse_index_at(mse_packed, dim_index, 4) as usize;
                let tile_dim = dim_index - tile_start;
                sum += tile_lut[tile_dim * 16 + centroid_index];
            }

            tile_start = tile_end;
        }

        sum
    }

    fn score_ip_from_split_parts_scalar(
        &self,
        prepared: &PreparedQuery,
        gamma: f32,
        mse_packed: &[u8],
        qjl_packed: &[u8],
    ) -> f32 {
        let bits_per_index = mse_bits(self.original_dim, self.bits);
        let num_centroids = 1usize << bits_per_index;

        let mut mse_sum = 0.0_f32;
        let mut qjl_sum = 0.0_f32;
        let mut dim_index = 0usize;
        let qjl_active = qjl_enabled(self.original_dim, self.bits);

        if !qjl_active {
            while dim_index < self.original_dim {
                let centroid_index = mse_index_at(mse_packed, dim_index, bits_per_index) as usize;
                mse_sum += if bits_per_index == 3 || prepared.lut.is_empty() {
                    self.codebook[centroid_index] * prepared.rotated[dim_index]
                } else {
                    prepared.lut[dim_index * num_centroids + centroid_index]
                };
                dim_index += 1;
            }
            return mse_sum;
        }

        if bits_per_index == 3 {
            while dim_index + 8 <= self.original_dim {
                let indices = decode_eight_3bit_aligned(mse_packed, dim_index);
                let sign_lanes = qjl_sign_lanes(qjl_packed[dim_index / 8]);
                for lane in 0..8 {
                    let absolute = dim_index + lane;
                    mse_sum += self.codebook[indices[lane]] * prepared.rotated[absolute];
                    qjl_sum += prepared.sq[absolute] * sign_lanes[lane];
                }
                dim_index += 8;
            }

            while dim_index < self.original_dim {
                let centroid_index = mse_index_at(mse_packed, dim_index, bits_per_index) as usize;
                mse_sum += self.codebook[centroid_index] * prepared.rotated[dim_index];
                qjl_sum += if qjl_sign_at(qjl_packed, dim_index) {
                    prepared.sq[dim_index]
                } else {
                    -prepared.sq[dim_index]
                };
                dim_index += 1;
            }
        } else {
            while dim_index < self.original_dim {
                let centroid_index = mse_index_at(mse_packed, dim_index, bits_per_index) as usize;
                mse_sum += prepared.lut[dim_index * num_centroids + centroid_index];
                qjl_sum += if qjl_sign_at(qjl_packed, dim_index) {
                    prepared.sq[dim_index]
                } else {
                    -prepared.sq[dim_index]
                };
                dim_index += 1;
            }
        }

        mse_sum + gamma * prepared.qjl_scale * qjl_sum
    }

    #[allow(dead_code)]
    pub fn score_ip_encoded_lite(&self, payload_a: &[u8], payload_b: &[u8]) -> f32 {
        let (_, mse_a, _) = self.split_payload(payload_a);
        let (_, mse_b, _) = self.split_payload(payload_b);
        self.score_ip_mse_codes(mse_a, mse_b)
    }

    pub fn score_ip_codes_lite(&self, code_a: &[u8], code_b: &[u8]) -> f32 {
        let (mse_a, _) = self.split_code_bytes(code_a);
        let (mse_b, _) = self.split_code_bytes(code_b);
        self.score_ip_mse_codes(mse_a, mse_b)
    }

    fn score_ip_mse_codes(&self, mse_a: &[u8], mse_b: &[u8]) -> f32 {
        if mse_bits(self.original_dim, self.bits) == 3 {
            match backend() {
                #[cfg(target_arch = "x86_64")]
                SimdBackend::Avx2Fma => unsafe { self.score_ip_mse_codes_avx2(mse_a, mse_b) },
                #[cfg(target_arch = "aarch64")]
                SimdBackend::Neon => unsafe { self.score_ip_mse_codes_neon(mse_a, mse_b) },
                SimdBackend::Scalar => self.score_ip_mse_codes_scalar(mse_a, mse_b),
            }
        } else {
            self.score_ip_mse_codes_scalar(mse_a, mse_b)
        }
    }

    fn score_ip_mse_codes_scalar(&self, mse_a: &[u8], mse_b: &[u8]) -> f32 {
        let bits_per_index = mse_bits(self.original_dim, self.bits);
        let mut mse_sum = 0.0_f32;
        let mut dim_index = 0usize;

        if bits_per_index == 3 {
            while dim_index + 8 <= self.original_dim {
                let indices_a = decode_eight_3bit_aligned(mse_a, dim_index);
                let indices_b = decode_eight_3bit_aligned(mse_b, dim_index);
                for lane in 0..8 {
                    mse_sum += self.codebook[indices_a[lane]] * self.codebook[indices_b[lane]];
                }
                dim_index += 8;
            }
        }

        while dim_index < self.original_dim {
            let idx_a = mse_index_at(mse_a, dim_index, bits_per_index) as usize;
            let idx_b = mse_index_at(mse_b, dim_index, bits_per_index) as usize;
            mse_sum += self.codebook[idx_a] * self.codebook[idx_b];
            dim_index += 1;
        }
        mse_sum
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,fma")]
    unsafe fn score_ip_mse_codes_avx2(&self, mse_a: &[u8], mse_b: &[u8]) -> f32 {
        use std::arch::x86_64::{
            _mm256_add_ps, _mm256_loadu_ps, _mm256_mul_ps, _mm256_permutevar8x32_ps,
            _mm256_set1_epi32, _mm256_setr_epi32, _mm256_setzero_ps, _mm256_storeu_ps,
        };

        let bits_per_index = self.bits - 1;
        if bits_per_index != 3 {
            return self.score_ip_mse_codes_scalar(mse_a, mse_b);
        }

        debug_assert_eq!(self.codebook.len(), 8);
        let codebook = _mm256_loadu_ps(self.codebook.as_ptr());
        let shifts = _mm256_setr_epi32(0, 3, 6, 9, 12, 15, 18, 21);
        let mask = _mm256_set1_epi32(0x7);
        let mut mse_acc0 = _mm256_setzero_ps();
        let mut mse_acc1 = _mm256_setzero_ps();
        let mut mse_acc2 = _mm256_setzero_ps();
        let mut mse_acc3 = _mm256_setzero_ps();
        let mut dim_index = 0usize;

        while dim_index + 32 <= self.original_dim {
            let la0 = decode_eight_3bit_lanes_avx2(
                decode_eight_3bit_aligned_word(mse_a, dim_index),
                shifts,
                mask,
            );
            let lb0 = decode_eight_3bit_lanes_avx2(
                decode_eight_3bit_aligned_word(mse_b, dim_index),
                shifts,
                mask,
            );
            let la1 = decode_eight_3bit_lanes_avx2(
                decode_eight_3bit_aligned_word(mse_a, dim_index + 8),
                shifts,
                mask,
            );
            let lb1 = decode_eight_3bit_lanes_avx2(
                decode_eight_3bit_aligned_word(mse_b, dim_index + 8),
                shifts,
                mask,
            );
            let la2 = decode_eight_3bit_lanes_avx2(
                decode_eight_3bit_aligned_word(mse_a, dim_index + 16),
                shifts,
                mask,
            );
            let lb2 = decode_eight_3bit_lanes_avx2(
                decode_eight_3bit_aligned_word(mse_b, dim_index + 16),
                shifts,
                mask,
            );
            let la3 = decode_eight_3bit_lanes_avx2(
                decode_eight_3bit_aligned_word(mse_a, dim_index + 24),
                shifts,
                mask,
            );
            let lb3 = decode_eight_3bit_lanes_avx2(
                decode_eight_3bit_aligned_word(mse_b, dim_index + 24),
                shifts,
                mask,
            );
            mse_acc0 = _mm256_add_ps(
                mse_acc0,
                _mm256_mul_ps(
                    _mm256_permutevar8x32_ps(codebook, la0),
                    _mm256_permutevar8x32_ps(codebook, lb0),
                ),
            );
            mse_acc1 = _mm256_add_ps(
                mse_acc1,
                _mm256_mul_ps(
                    _mm256_permutevar8x32_ps(codebook, la1),
                    _mm256_permutevar8x32_ps(codebook, lb1),
                ),
            );
            mse_acc2 = _mm256_add_ps(
                mse_acc2,
                _mm256_mul_ps(
                    _mm256_permutevar8x32_ps(codebook, la2),
                    _mm256_permutevar8x32_ps(codebook, lb2),
                ),
            );
            mse_acc3 = _mm256_add_ps(
                mse_acc3,
                _mm256_mul_ps(
                    _mm256_permutevar8x32_ps(codebook, la3),
                    _mm256_permutevar8x32_ps(codebook, lb3),
                ),
            );
            dim_index += 32;
        }

        while dim_index + 8 <= self.original_dim {
            let lanes_a = decode_eight_3bit_lanes_avx2(
                decode_eight_3bit_aligned_word(mse_a, dim_index),
                shifts,
                mask,
            );
            let lanes_b = decode_eight_3bit_lanes_avx2(
                decode_eight_3bit_aligned_word(mse_b, dim_index),
                shifts,
                mask,
            );
            mse_acc0 = _mm256_add_ps(
                mse_acc0,
                _mm256_mul_ps(
                    _mm256_permutevar8x32_ps(codebook, lanes_a),
                    _mm256_permutevar8x32_ps(codebook, lanes_b),
                ),
            );
            dim_index += 8;
        }

        let mut mse_lanes = [0.0_f32; 8];
        _mm256_storeu_ps(
            mse_lanes.as_mut_ptr(),
            _mm256_add_ps(
                _mm256_add_ps(mse_acc0, mse_acc1),
                _mm256_add_ps(mse_acc2, mse_acc3),
            ),
        );
        let mut mse_sum = mse_lanes.into_iter().sum::<f32>();

        while dim_index < self.original_dim {
            let idx_a = mse_index_at(mse_a, dim_index, bits_per_index) as usize;
            let idx_b = mse_index_at(mse_b, dim_index, bits_per_index) as usize;
            mse_sum += self.codebook[idx_a] * self.codebook[idx_b];
            dim_index += 1;
        }

        mse_sum
    }

    #[allow(dead_code)]
    pub fn pack_payload(&self, encoded: &EncodedTq) -> Vec<u8> {
        let mut payload =
            Vec::with_capacity(4 + encoded.mse_packed.len() + encoded.qjl_packed.len());
        payload.extend_from_slice(&encoded.gamma.to_le_bytes());
        payload.extend_from_slice(&encoded.mse_packed);
        payload.extend_from_slice(&encoded.qjl_packed);
        payload
    }

    fn split_code_bytes<'a>(&self, code_bytes: &'a [u8]) -> (&'a [u8], &'a [u8]) {
        let mse_len = mse_code_len(self.original_dim, self.bits);
        let qjl_len = qjl_code_len_for_bits(self.original_dim, self.bits);
        assert_eq!(
            code_bytes.len(),
            mse_len + qjl_len,
            "code length mismatch: got {}, expected {}",
            code_bytes.len(),
            mse_len + qjl_len
        );
        let qjl_start = mse_len;
        (
            &code_bytes[..qjl_start],
            &code_bytes[qjl_start..qjl_start + qjl_len],
        )
    }

    fn split_payload<'a>(&self, payload: &'a [u8]) -> (f32, &'a [u8], &'a [u8]) {
        assert!(
            payload.len() >= 4,
            "payload too short: got {}, need at least 4 bytes",
            payload.len()
        );
        let gamma = f32::from_le_bytes(payload[..4].try_into().expect("gamma slice"));
        let mse_len = mse_code_len(self.original_dim, self.bits);
        let qjl_len = qjl_code_len_for_bits(self.original_dim, self.bits);
        assert_eq!(
            payload.len(),
            4 + mse_len + qjl_len,
            "payload length mismatch: got {}, expected {}",
            payload.len(),
            4 + mse_len + qjl_len
        );
        let mse_start = 4;
        let qjl_start = mse_start + mse_len;
        (
            gamma,
            &payload[mse_start..qjl_start],
            &payload[qjl_start..qjl_start + qjl_len],
        )
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,fma")]
    unsafe fn score_ip_from_split_parts_avx2(
        &self,
        prepared: &PreparedQuery,
        gamma: f32,
        mse_packed: &[u8],
        qjl_packed: &[u8],
    ) -> f32 {
        use std::arch::x86_64::{
            _mm256_add_ps, _mm256_fmadd_ps, _mm256_loadu_ps, _mm256_mul_ps,
            _mm256_permutevar8x32_ps, _mm256_set1_epi32, _mm256_setr_epi32, _mm256_setzero_ps,
            _mm256_storeu_ps,
        };

        let bits_per_index = mse_bits(self.original_dim, self.bits);
        let num_centroids = 1usize << bits_per_index;
        let mut mse_sum = 0.0_f32;
        let mut qjl_sum = 0.0_f32;
        let mut dim_index = 0usize;
        let mut mse_acc0 = _mm256_setzero_ps();
        let mut mse_acc1 = _mm256_setzero_ps();
        let mut mse_acc2 = _mm256_setzero_ps();
        let mut mse_acc3 = _mm256_setzero_ps();
        let mut qjl_acc0 = _mm256_setzero_ps();
        let mut qjl_acc1 = _mm256_setzero_ps();
        let mut qjl_acc2 = _mm256_setzero_ps();
        let mut qjl_acc3 = _mm256_setzero_ps();

        if bits_per_index == 3 {
            debug_assert_eq!(self.codebook.len(), 8);
            let codebook = _mm256_loadu_ps(self.codebook.as_ptr());
            let shifts = _mm256_setr_epi32(0, 3, 6, 9, 12, 15, 18, 21);
            let mask = _mm256_set1_epi32(0x7);

            while dim_index + 32 <= self.original_dim {
                let l0 = decode_eight_3bit_lanes_avx2(
                    decode_eight_3bit_aligned_word(mse_packed, dim_index),
                    shifts,
                    mask,
                );
                let l1 = decode_eight_3bit_lanes_avx2(
                    decode_eight_3bit_aligned_word(mse_packed, dim_index + 8),
                    shifts,
                    mask,
                );
                let l2 = decode_eight_3bit_lanes_avx2(
                    decode_eight_3bit_aligned_word(mse_packed, dim_index + 16),
                    shifts,
                    mask,
                );
                let l3 = decode_eight_3bit_lanes_avx2(
                    decode_eight_3bit_aligned_word(mse_packed, dim_index + 24),
                    shifts,
                    mask,
                );

                mse_acc0 = _mm256_fmadd_ps(
                    _mm256_permutevar8x32_ps(codebook, l0),
                    _mm256_loadu_ps(prepared.rotated.as_ptr().add(dim_index)),
                    mse_acc0,
                );
                qjl_acc0 = _mm256_fmadd_ps(
                    _mm256_loadu_ps(prepared.sq.as_ptr().add(dim_index)),
                    _mm256_loadu_ps(qjl_sign_lanes(qjl_packed[dim_index / 8]).as_ptr()),
                    qjl_acc0,
                );
                mse_acc1 = _mm256_fmadd_ps(
                    _mm256_permutevar8x32_ps(codebook, l1),
                    _mm256_loadu_ps(prepared.rotated.as_ptr().add(dim_index + 8)),
                    mse_acc1,
                );
                qjl_acc1 = _mm256_fmadd_ps(
                    _mm256_loadu_ps(prepared.sq.as_ptr().add(dim_index + 8)),
                    _mm256_loadu_ps(qjl_sign_lanes(qjl_packed[(dim_index + 8) / 8]).as_ptr()),
                    qjl_acc1,
                );
                mse_acc2 = _mm256_fmadd_ps(
                    _mm256_permutevar8x32_ps(codebook, l2),
                    _mm256_loadu_ps(prepared.rotated.as_ptr().add(dim_index + 16)),
                    mse_acc2,
                );
                qjl_acc2 = _mm256_fmadd_ps(
                    _mm256_loadu_ps(prepared.sq.as_ptr().add(dim_index + 16)),
                    _mm256_loadu_ps(qjl_sign_lanes(qjl_packed[(dim_index + 16) / 8]).as_ptr()),
                    qjl_acc2,
                );
                mse_acc3 = _mm256_fmadd_ps(
                    _mm256_permutevar8x32_ps(codebook, l3),
                    _mm256_loadu_ps(prepared.rotated.as_ptr().add(dim_index + 24)),
                    mse_acc3,
                );
                qjl_acc3 = _mm256_fmadd_ps(
                    _mm256_loadu_ps(prepared.sq.as_ptr().add(dim_index + 24)),
                    _mm256_loadu_ps(qjl_sign_lanes(qjl_packed[(dim_index + 24) / 8]).as_ptr()),
                    qjl_acc3,
                );
                dim_index += 32;
            }

            while dim_index + 8 <= self.original_dim {
                let lanes = decode_eight_3bit_lanes_avx2(
                    decode_eight_3bit_aligned_word(mse_packed, dim_index),
                    shifts,
                    mask,
                );

                mse_acc0 = _mm256_fmadd_ps(
                    _mm256_permutevar8x32_ps(codebook, lanes),
                    _mm256_loadu_ps(prepared.rotated.as_ptr().add(dim_index)),
                    mse_acc0,
                );
                qjl_acc0 = _mm256_fmadd_ps(
                    _mm256_loadu_ps(prepared.sq.as_ptr().add(dim_index)),
                    _mm256_loadu_ps(qjl_sign_lanes(qjl_packed[dim_index / 8]).as_ptr()),
                    qjl_acc0,
                );
                dim_index += 8;
            }
        } else {
            while dim_index + 8 <= self.original_dim {
                let mut mse_values = [0.0_f32; 8];
                for (lane, mse_value) in mse_values.iter_mut().enumerate() {
                    let absolute = dim_index + lane;
                    let centroid_index =
                        mse_index_at(mse_packed, absolute, bits_per_index) as usize;
                    *mse_value = prepared.lut[absolute * num_centroids + centroid_index];
                }

                mse_acc0 = _mm256_add_ps(mse_acc0, _mm256_loadu_ps(mse_values.as_ptr()));
                qjl_acc0 = _mm256_add_ps(
                    qjl_acc0,
                    _mm256_mul_ps(
                        _mm256_loadu_ps(prepared.sq.as_ptr().add(dim_index)),
                        _mm256_loadu_ps(qjl_sign_lanes(qjl_packed[dim_index / 8]).as_ptr()),
                    ),
                );
                dim_index += 8;
            }
        }

        let mut mse_lanes = [0.0_f32; 8];
        let mut qjl_lanes = [0.0_f32; 8];
        _mm256_storeu_ps(
            mse_lanes.as_mut_ptr(),
            _mm256_add_ps(
                _mm256_add_ps(mse_acc0, mse_acc1),
                _mm256_add_ps(mse_acc2, mse_acc3),
            ),
        );
        _mm256_storeu_ps(
            qjl_lanes.as_mut_ptr(),
            _mm256_add_ps(
                _mm256_add_ps(qjl_acc0, qjl_acc1),
                _mm256_add_ps(qjl_acc2, qjl_acc3),
            ),
        );
        mse_sum += mse_lanes.into_iter().sum::<f32>();
        qjl_sum += qjl_lanes.into_iter().sum::<f32>();

        while dim_index < self.original_dim {
            let centroid_index = mse_index_at(mse_packed, dim_index, bits_per_index) as usize;
            mse_sum += if bits_per_index == 3 {
                self.codebook[centroid_index] * prepared.rotated[dim_index]
            } else {
                prepared.lut[dim_index * num_centroids + centroid_index]
            };
            qjl_sum += if qjl_sign_at(qjl_packed, dim_index) {
                prepared.sq[dim_index]
            } else {
                -prepared.sq[dim_index]
            };
            dim_index += 1;
        }

        mse_sum + gamma * prepared.qjl_scale * qjl_sum
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn score_ip_from_split_parts_neon(
        &self,
        prepared: &PreparedQuery,
        gamma: f32,
        mse_packed: &[u8],
        qjl_packed: &[u8],
    ) -> f32 {
        use std::arch::aarch64::{
            vaddq_f32, vandq_u32, vdupq_n_f32, vdupq_n_u32, vfmaq_f32, vld1q_f32, vld1q_s32,
            vmulq_f32, vshlq_u32, vst1q_f32, vst1q_u32,
        };

        let bits_per_index = self.bits - 1;
        let num_centroids = 1usize << bits_per_index;
        let mut mse_sum = 0.0_f32;
        let mut qjl_sum = 0.0_f32;
        let mut dim_index = 0usize;

        if bits_per_index == 3 {
            let shifts_lo = vld1q_s32([0_i32, -3, -6, -9].as_ptr());
            let shifts_hi = vld1q_s32([-12_i32, -15, -18, -21].as_ptr());
            let mask = vdupq_n_u32(0x7);
            let mut mse_acc0 = vdupq_n_f32(0.0);
            let mut mse_acc1 = vdupq_n_f32(0.0);
            let mut qjl_acc0 = vdupq_n_f32(0.0);
            let mut qjl_acc1 = vdupq_n_f32(0.0);

            while dim_index + 8 <= self.original_dim {
                let word = decode_eight_3bit_aligned_word(mse_packed, dim_index);
                let broadcast = vdupq_n_u32(word);
                let idx_lo = vandq_u32(vshlq_u32(broadcast, shifts_lo), mask);
                let idx_hi = vandq_u32(vshlq_u32(broadcast, shifts_hi), mask);

                let mut idx_buf = [0_u32; 4];
                let mut cb_lo = [0.0_f32; 4];
                let mut cb_hi = [0.0_f32; 4];
                vst1q_u32(idx_buf.as_mut_ptr(), idx_lo);
                cb_lo[0] = self.codebook[idx_buf[0] as usize];
                cb_lo[1] = self.codebook[idx_buf[1] as usize];
                cb_lo[2] = self.codebook[idx_buf[2] as usize];
                cb_lo[3] = self.codebook[idx_buf[3] as usize];
                vst1q_u32(idx_buf.as_mut_ptr(), idx_hi);
                cb_hi[0] = self.codebook[idx_buf[0] as usize];
                cb_hi[1] = self.codebook[idx_buf[1] as usize];
                cb_hi[2] = self.codebook[idx_buf[2] as usize];
                cb_hi[3] = self.codebook[idx_buf[3] as usize];

                mse_acc0 = vfmaq_f32(
                    mse_acc0,
                    vld1q_f32(cb_lo.as_ptr()),
                    vld1q_f32(prepared.rotated.as_ptr().add(dim_index)),
                );
                mse_acc1 = vfmaq_f32(
                    mse_acc1,
                    vld1q_f32(cb_hi.as_ptr()),
                    vld1q_f32(prepared.rotated.as_ptr().add(dim_index + 4)),
                );

                let sign_lanes = qjl_sign_lanes(qjl_packed[dim_index / 8]);
                qjl_acc0 = vfmaq_f32(
                    qjl_acc0,
                    vld1q_f32(prepared.sq.as_ptr().add(dim_index)),
                    vld1q_f32(sign_lanes.as_ptr()),
                );
                qjl_acc1 = vfmaq_f32(
                    qjl_acc1,
                    vld1q_f32(prepared.sq.as_ptr().add(dim_index + 4)),
                    vld1q_f32(sign_lanes.as_ptr().add(4)),
                );

                dim_index += 8;
            }

            let mse_total = vaddq_f32(mse_acc0, mse_acc1);
            let qjl_total = vaddq_f32(qjl_acc0, qjl_acc1);
            let mut mse_lanes = [0.0_f32; 4];
            let mut qjl_lanes = [0.0_f32; 4];
            vst1q_f32(mse_lanes.as_mut_ptr(), mse_total);
            vst1q_f32(qjl_lanes.as_mut_ptr(), qjl_total);
            mse_sum += mse_lanes.iter().sum::<f32>();
            qjl_sum += qjl_lanes.iter().sum::<f32>();
        } else {
            while dim_index + 4 <= self.original_dim {
                let mut mse_values = [0.0_f32; 4];
                for (lane, mse_value) in mse_values.iter_mut().enumerate() {
                    let absolute = dim_index + lane;
                    let centroid_index =
                        mse_index_at(mse_packed, absolute, bits_per_index) as usize;
                    *mse_value = prepared.lut[absolute * num_centroids + centroid_index];
                }

                let mut qjl_terms = [0.0_f32; 4];
                let sign_lanes = qjl_sign_lanes(qjl_packed[dim_index / 8]);
                vst1q_f32(
                    qjl_terms.as_mut_ptr(),
                    vmulq_f32(
                        vld1q_f32(prepared.sq.as_ptr().add(dim_index)),
                        vld1q_f32(sign_lanes.as_ptr().add(dim_index % 8)),
                    ),
                );
                for lane in 0..4 {
                    mse_sum += mse_values[lane];
                    qjl_sum += qjl_terms[lane];
                }
                dim_index += 4;
            }
        }

        while dim_index < self.original_dim {
            let centroid_index = mse_index_at(mse_packed, dim_index, bits_per_index) as usize;
            mse_sum += if bits_per_index == 3 {
                self.codebook[centroid_index] * prepared.rotated[dim_index]
            } else {
                prepared.lut[dim_index * num_centroids + centroid_index]
            };
            qjl_sum += if qjl_sign_at(qjl_packed, dim_index) {
                prepared.sq[dim_index]
            } else {
                -prepared.sq[dim_index]
            };
            dim_index += 1;
        }

        mse_sum + gamma * prepared.qjl_scale * qjl_sum
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn score_ip_mse_codes_neon(&self, mse_a: &[u8], mse_b: &[u8]) -> f32 {
        use std::arch::aarch64::{
            vaddq_f32, vandq_u32, vdupq_n_f32, vdupq_n_u32, vfmaq_f32, vld1q_f32, vld1q_s32,
            vshlq_u32, vst1q_f32, vst1q_u32,
        };

        let bits_per_index = mse_bits(self.original_dim, self.bits);
        if bits_per_index != 3 {
            return self.score_ip_mse_codes_scalar(mse_a, mse_b);
        }

        let shifts_lo = vld1q_s32([0_i32, -3, -6, -9].as_ptr());
        let shifts_hi = vld1q_s32([-12_i32, -15, -18, -21].as_ptr());
        let mask = vdupq_n_u32(0x7);
        let mut mse_acc0 = vdupq_n_f32(0.0);
        let mut mse_acc1 = vdupq_n_f32(0.0);
        let mut dim_index = 0usize;

        while dim_index + 8 <= self.original_dim {
            let word_a = decode_eight_3bit_aligned_word(mse_a, dim_index);
            let word_b = decode_eight_3bit_aligned_word(mse_b, dim_index);
            let broadcast_a = vdupq_n_u32(word_a);
            let broadcast_b = vdupq_n_u32(word_b);
            let idx_a_lo = vandq_u32(vshlq_u32(broadcast_a, shifts_lo), mask);
            let idx_a_hi = vandq_u32(vshlq_u32(broadcast_a, shifts_hi), mask);
            let idx_b_lo = vandq_u32(vshlq_u32(broadcast_b, shifts_lo), mask);
            let idx_b_hi = vandq_u32(vshlq_u32(broadcast_b, shifts_hi), mask);

            let mut idx_buf = [0_u32; 4];
            let mut cb_a_lo = [0.0_f32; 4];
            let mut cb_a_hi = [0.0_f32; 4];
            let mut cb_b_lo = [0.0_f32; 4];
            let mut cb_b_hi = [0.0_f32; 4];
            vst1q_u32(idx_buf.as_mut_ptr(), idx_a_lo);
            cb_a_lo[0] = self.codebook[idx_buf[0] as usize];
            cb_a_lo[1] = self.codebook[idx_buf[1] as usize];
            cb_a_lo[2] = self.codebook[idx_buf[2] as usize];
            cb_a_lo[3] = self.codebook[idx_buf[3] as usize];
            vst1q_u32(idx_buf.as_mut_ptr(), idx_a_hi);
            cb_a_hi[0] = self.codebook[idx_buf[0] as usize];
            cb_a_hi[1] = self.codebook[idx_buf[1] as usize];
            cb_a_hi[2] = self.codebook[idx_buf[2] as usize];
            cb_a_hi[3] = self.codebook[idx_buf[3] as usize];
            vst1q_u32(idx_buf.as_mut_ptr(), idx_b_lo);
            cb_b_lo[0] = self.codebook[idx_buf[0] as usize];
            cb_b_lo[1] = self.codebook[idx_buf[1] as usize];
            cb_b_lo[2] = self.codebook[idx_buf[2] as usize];
            cb_b_lo[3] = self.codebook[idx_buf[3] as usize];
            vst1q_u32(idx_buf.as_mut_ptr(), idx_b_hi);
            cb_b_hi[0] = self.codebook[idx_buf[0] as usize];
            cb_b_hi[1] = self.codebook[idx_buf[1] as usize];
            cb_b_hi[2] = self.codebook[idx_buf[2] as usize];
            cb_b_hi[3] = self.codebook[idx_buf[3] as usize];

            mse_acc0 = vfmaq_f32(
                mse_acc0,
                vld1q_f32(cb_a_lo.as_ptr()),
                vld1q_f32(cb_b_lo.as_ptr()),
            );
            mse_acc1 = vfmaq_f32(
                mse_acc1,
                vld1q_f32(cb_a_hi.as_ptr()),
                vld1q_f32(cb_b_hi.as_ptr()),
            );

            dim_index += 8;
        }

        let mse_total = vaddq_f32(mse_acc0, mse_acc1);
        let mut mse_lanes = [0.0_f32; 4];
        vst1q_f32(mse_lanes.as_mut_ptr(), mse_total);
        let mut mse_sum = mse_lanes.iter().sum::<f32>();

        while dim_index < self.original_dim {
            let idx_a = mse_index_at(mse_a, dim_index, bits_per_index) as usize;
            let idx_b = mse_index_at(mse_b, dim_index, bits_per_index) as usize;
            mse_sum += self.codebook[idx_a] * self.codebook[idx_b];
            dim_index += 1;
        }

        mse_sum
    }
}

fn qjl_enabled(dim: usize, bits: u8) -> bool {
    !(bits == 4 && rotation::tile_dim(dim).is_some())
}

fn prepared_query_uses_lut(dim: usize, bits: u8) -> bool {
    qjl_enabled(dim, bits) || bits != 4
}

fn mse_bits(dim: usize, bits: u8) -> u8 {
    if qjl_enabled(dim, bits) {
        bits.saturating_sub(1)
    } else {
        bits
    }
}

fn qjl_code_len_for_bits(dim: usize, bits: u8) -> usize {
    if qjl_enabled(dim, bits) {
        qjl_code_len(dim)
    } else {
        0
    }
}

pub fn mse_code_len(dim: usize, bits: u8) -> usize {
    let bits_per_index = mse_bits(dim, bits) as usize;
    (dim * bits_per_index).div_ceil(8)
}

pub fn qjl_code_len(dim: usize) -> usize {
    dim.div_ceil(8)
}

fn build_prepared_query_lut(rotated: &[f32], codebook: &[f32], num_centroids: usize) -> Vec<f32> {
    debug_assert_eq!(codebook.len(), num_centroids);
    let mut lut = vec![0.0_f32; rotated.len() * num_centroids];

    if let [c0, c1, c2, c3, c4, c5, c6, c7] = codebook {
        for (row, &value) in lut.chunks_exact_mut(8).zip(rotated.iter()) {
            row[0] = c0 * value;
            row[1] = c1 * value;
            row[2] = c2 * value;
            row[3] = c3 * value;
            row[4] = c4 * value;
            row[5] = c5 * value;
            row[6] = c6 * value;
            row[7] = c7 * value;
        }
        return lut;
    }

    for (row, &value) in lut.chunks_exact_mut(num_centroids).zip(rotated.iter()) {
        for (slot, &centroid) in row.iter_mut().zip(codebook.iter()) {
            *slot = centroid * value;
        }
    }

    lut
}

fn quantize_i8(values: &[f32]) -> (Vec<i8>, f32) {
    let max_abs = values
        .iter()
        .fold(0.0_f32, |max_abs, value| max_abs.max(value.abs()));
    if max_abs <= f32::EPSILON {
        return (vec![0_i8; values.len()], 0.0);
    }

    let scale = max_abs / 127.0;
    let quantized = values
        .iter()
        .map(|value| ((*value / scale).round().clamp(-127.0, 127.0)) as i8)
        .collect();
    (quantized, scale)
}

fn quantize_codebook_i8_16(codebook: &[f32]) -> ([i8; 16], f32) {
    assert_eq!(
        codebook.len(),
        16,
        "int8 codebook quantization requires a 16-entry 4-bit codebook"
    );
    let max_abs = codebook
        .iter()
        .fold(0.0_f32, |max_abs, value| max_abs.max(value.abs()));
    if max_abs <= f32::EPSILON {
        return ([0_i8; 16], 0.0);
    }

    let scale = max_abs / 127.0;
    let mut quantized = [0_i8; 16];
    for (slot, value) in quantized.iter_mut().zip(codebook.iter()) {
        *slot = ((*value / scale).round().clamp(-127.0, 127.0)) as i8;
    }
    (quantized, scale)
}

pub fn payload_len(dim: usize, bits: u8) -> usize {
    4 + mse_code_len(dim, bits) + qjl_code_len_for_bits(dim, bits)
}

pub fn pack_mse_indices(indices: &[CodeIndex], bits_per_index: u8) -> Vec<u8> {
    // Bytewise fast paths for the production bit widths (2, 3, 4, 5).
    // Each fast path produces output that is byte-for-byte identical to
    // the generic per-bit loop — verified by
    // `pack_mse_indices_fast_paths_match_generic`.
    match bits_per_index {
        2 => pack_mse_indices_2bit(indices),
        3 => pack_mse_indices_3bit(indices),
        4 => pack_mse_indices_4bit(indices),
        5 => pack_mse_indices_5bit(indices),
        _ => pack_mse_indices_generic(indices, bits_per_index),
    }
}

fn pack_mse_indices_generic(indices: &[CodeIndex], bits_per_index: u8) -> Vec<u8> {
    let total_bits = indices.len() * bits_per_index as usize;
    let mut packed = vec![0_u8; total_bits.div_ceil(8)];
    for (index, value) in indices.iter().enumerate() {
        write_bits_le(
            &mut packed,
            index * bits_per_index as usize,
            bits_per_index as usize,
            *value,
        );
    }
    packed
}

fn pack_mse_indices_2bit(indices: &[CodeIndex]) -> Vec<u8> {
    // Four 2-bit indices per byte: byte = (d3<<6) | (d2<<4) | (d1<<2) | d0.
    let total_bits = indices.len() * 2;
    let mut packed = vec![0_u8; total_bits.div_ceil(8)];
    for (out_byte, chunk) in packed.iter_mut().zip(indices.chunks(4)) {
        let d0 = (chunk[0] & 0x03) as u8;
        let d1 = chunk.get(1).copied().unwrap_or(0) as u8 & 0x03;
        let d2 = chunk.get(2).copied().unwrap_or(0) as u8 & 0x03;
        let d3 = chunk.get(3).copied().unwrap_or(0) as u8 & 0x03;
        *out_byte = (d3 << 6) | (d2 << 4) | (d1 << 2) | d0;
    }
    packed
}

fn pack_mse_indices_3bit(indices: &[CodeIndex]) -> Vec<u8> {
    // Eight 3-bit indices pack into three bytes (24 bits). Build the
    // 24-bit accumulator low-bit-first to match write_bits_le's
    // little-endian convention, then write the low 3 bytes.
    let total_bits = indices.len() * 3;
    let mut packed = vec![0_u8; total_bits.div_ceil(8)];
    let mut out = packed.as_mut_slice();
    for chunk in indices.chunks(8) {
        let mut acc: u32 = 0;
        for (offset, value) in chunk.iter().enumerate() {
            acc |= ((*value as u32) & 0x07) << (offset * 3);
        }
        let total_bytes = (chunk.len() * 3).div_ceil(8);
        let bytes = acc.to_le_bytes();
        out[..total_bytes].copy_from_slice(&bytes[..total_bytes]);
        out = &mut out[total_bytes..];
    }
    packed
}

fn pack_mse_indices_4bit(indices: &[CodeIndex]) -> Vec<u8> {
    // Two 4-bit indices per byte: low nibble = first index, high nibble
    // = second index. Matches write_bits_le's little-endian layout.
    let total_bits = indices.len() * 4;
    let mut packed = vec![0_u8; total_bits.div_ceil(8)];
    for (out_byte, chunk) in packed.iter_mut().zip(indices.chunks(2)) {
        let lo = (chunk[0] & 0x0F) as u8;
        let hi = chunk.get(1).copied().unwrap_or(0) as u8 & 0x0F;
        *out_byte = (hi << 4) | lo;
    }
    packed
}

fn pack_mse_indices_5bit(indices: &[CodeIndex]) -> Vec<u8> {
    // Eight 5-bit indices pack into five bytes (40 bits). Build the
    // 40-bit accumulator low-bit-first to match write_bits_le's
    // little-endian convention, then write the low 5 bytes.
    let total_bits = indices.len() * 5;
    let mut packed = vec![0_u8; total_bits.div_ceil(8)];
    let mut out = packed.as_mut_slice();
    for chunk in indices.chunks(8) {
        let mut acc: u64 = 0;
        for (offset, value) in chunk.iter().enumerate() {
            acc |= ((*value as u64) & 0x1F) << (offset * 5);
        }
        let total_bytes = (chunk.len() * 5).div_ceil(8);
        let bytes = acc.to_le_bytes();
        out[..total_bytes].copy_from_slice(&bytes[..total_bytes]);
        out = &mut out[total_bytes..];
    }
    packed
}

#[allow(dead_code)]
pub fn unpack_mse_indices(packed: &[u8], dim: usize, bits_per_index: u8) -> Vec<CodeIndex> {
    (0..dim)
        .map(|index| mse_index_at(packed, index, bits_per_index))
        .collect()
}

pub fn pack_qjl_signs(signs: &[bool]) -> Vec<u8> {
    let mut packed = vec![0_u8; qjl_code_len(signs.len())];
    for (index, is_positive) in signs.iter().enumerate() {
        if *is_positive {
            packed[index / 8] |= 1 << (index % 8);
        }
    }
    packed
}

#[allow(dead_code)]
pub fn unpack_qjl_signs(packed: &[u8], dim: usize) -> Vec<bool> {
    (0..dim).map(|index| qjl_sign_at(packed, index)).collect()
}

fn mse_index_at(packed: &[u8], dim_index: usize, bits_per_index: u8) -> CodeIndex {
    read_bits_le(
        packed,
        dim_index * bits_per_index as usize,
        bits_per_index as usize,
    )
}

fn decode_eight_3bit_aligned_word(packed: &[u8], dim_index: usize) -> u32 {
    debug_assert_eq!(dim_index % 8, 0);
    let byte_index = (dim_index / 8) * 3;
    u32::from_le_bytes([
        packed[byte_index],
        packed[byte_index + 1],
        packed[byte_index + 2],
        0,
    ])
}

fn decode_eight_3bit_aligned(packed: &[u8], dim_index: usize) -> [usize; 8] {
    let word = decode_eight_3bit_aligned_word(packed, dim_index);
    [
        (word & 0x7) as usize,
        ((word >> 3) & 0x7) as usize,
        ((word >> 6) & 0x7) as usize,
        ((word >> 9) & 0x7) as usize,
        ((word >> 12) & 0x7) as usize,
        ((word >> 15) & 0x7) as usize,
        ((word >> 18) & 0x7) as usize,
        ((word >> 21) & 0x7) as usize,
    ]
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
unsafe fn decode_eight_3bit_lanes_avx2(
    word: u32,
    shifts: std::arch::x86_64::__m256i,
    mask: std::arch::x86_64::__m256i,
) -> std::arch::x86_64::__m256i {
    use std::arch::x86_64::{_mm256_and_si256, _mm256_set1_epi32, _mm256_srlv_epi32};

    _mm256_and_si256(
        _mm256_srlv_epi32(_mm256_set1_epi32(word as i32), shifts),
        mask,
    )
}

fn qjl_sign_at(packed: &[u8], dim_index: usize) -> bool {
    (packed[dim_index / 8] >> (dim_index % 8)) & 1 == 1
}

fn qjl_sign_lanes(byte: u8) -> &'static [f32; 8] {
    static LUT: [[f32; 8]; 256] = build_qjl_sign_lut();
    &LUT[byte as usize]
}

const fn build_qjl_sign_lut() -> [[f32; 8]; 256] {
    let mut lut = [[0.0; 8]; 256];
    let mut byte = 0usize;
    while byte < 256 {
        let mut lane = 0usize;
        while lane < 8 {
            lut[byte][lane] = if ((byte >> lane) & 1) == 1 { 1.0 } else { -1.0 };
            lane += 1;
        }
        byte += 1;
    }
    lut
}

fn write_bits_le(buffer: &mut [u8], start_bit: usize, width: usize, value: u16) {
    for offset in 0..width {
        let absolute_bit = start_bit + offset;
        let byte_index = absolute_bit / 8;
        let bit_index = absolute_bit % 8;
        let bit = ((value >> offset) & 1) as u8;
        buffer[byte_index] = (buffer[byte_index] & !(1 << bit_index)) | (bit << bit_index);
    }
}

fn read_bits_le(buffer: &[u8], start_bit: usize, width: usize) -> u16 {
    debug_assert!((1..=16).contains(&width));
    let byte_index = start_bit / 8;
    let bit_index = start_bit % 8;
    let bytes_to_read = ((bit_index + width).div_ceil(8)).min(3);
    let mut word = 0_u32;
    for offset in 0..bytes_to_read {
        word |= (buffer[byte_index + offset] as u32) << (offset * 8);
    }
    let mask = (1_u32 << width) - 1;
    ((word >> bit_index) & mask) as u16
}

#[allow(dead_code)]
struct ProdQueryScorer<'a> {
    quantizer: &'a ProdQuantizer,
    prepared: PreparedQuery,
}

impl crate::quant::QueryScorer for ProdQueryScorer<'_> {
    fn score(&self, code: &[u8]) -> f32 {
        self.quantizer.score_ip_encoded(&self.prepared, code)
    }
}

impl crate::quant::Quantizer for ProdQuantizer {
    fn encode_code(&self, v: &[f32]) -> Box<[u8]> {
        self.pack_payload(&self.encode(v)).into_boxed_slice()
    }

    fn prepare_scorer(
        &self,
        query: &[f32],
    ) -> Box<dyn crate::quant::QueryScorer + Send + Sync + '_> {
        Box::new(ProdQueryScorer {
            quantizer: self,
            prepared: self.prepare_ip_query(query),
        })
    }

    fn code_len(&self) -> usize {
        payload_len(self.original_dim, self.bits)
    }

    fn wire_format_version(&self) -> u32 {
        crate::am::page::INDEX_FORMAT_V1_SCALAR as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn random_unit_vector(dim: usize, seed: u64) -> Vec<f32> {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        let mut values = (0..dim)
            .map(|_| rng.gen_range(-1.0_f32..1.0_f32))
            .collect::<Vec<_>>();
        let norm = values.iter().map(|v| v * v).sum::<f32>().sqrt();
        for value in &mut values {
            *value /= norm.max(f32::EPSILON);
        }
        values
    }

    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let dot = a.iter().zip(b).map(|(x, y)| x * y).sum::<f32>();
        let norm_a = a.iter().map(|v| v * v).sum::<f32>().sqrt();
        let norm_b = b.iter().map(|v| v * v).sum::<f32>().sqrt();
        dot / (norm_a * norm_b).max(f32::EPSILON)
    }

    #[test]
    fn mse_pack_unpack_roundtrip_all_widths() {
        let mut rng = ChaCha8Rng::seed_from_u64(7);
        for bits in 1..=7 {
            let max_value = 1_u16 << bits;
            let indices = (0..257)
                .map(|_| rng.gen_range(0..max_value))
                .collect::<Vec<_>>();
            let packed = pack_mse_indices(&indices, bits);
            let unpacked = unpack_mse_indices(&packed, indices.len(), bits);
            assert_eq!(unpacked, indices, "failed at bits={bits}");
        }
    }

    #[test]
    fn decode_eight_3bit_aligned_matches_packer() {
        let indices = vec![0u16, 7, 3, 5, 1, 6, 2, 4];
        let packed = pack_mse_indices(&indices, 3);
        let decoded = decode_eight_3bit_aligned(&packed, 0);
        assert_eq!(decoded, [0usize, 7, 3, 5, 1, 6, 2, 4]);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn decode_eight_3bit_lanes_avx2_matches_scalar_when_available() {
        use std::arch::x86_64::{_mm256_set1_epi32, _mm256_setr_epi32, _mm256_storeu_si256};

        if !is_x86_feature_detected!("avx2") {
            return;
        }

        let mut rng = ChaCha8Rng::seed_from_u64(8);
        for _ in 0..1_000 {
            let indices = (0..8).map(|_| rng.gen_range(0u16..8)).collect::<Vec<_>>();
            let packed = pack_mse_indices(&indices, 3);
            let scalar = decode_eight_3bit_aligned(&packed, 0);
            let shifts = unsafe { _mm256_setr_epi32(0, 3, 6, 9, 12, 15, 18, 21) };
            let mask = unsafe { _mm256_set1_epi32(0x7) };
            let lanes = unsafe {
                decode_eight_3bit_lanes_avx2(
                    decode_eight_3bit_aligned_word(&packed, 0),
                    shifts,
                    mask,
                )
            };
            let mut avx = [0_i32; 8];
            unsafe { _mm256_storeu_si256(avx.as_mut_ptr().cast(), lanes) };
            assert_eq!(avx.map(|lane| lane as usize), scalar);
        }
    }

    #[test]
    fn qjl_pack_unpack_roundtrip() {
        let signs = vec![
            true, false, true, true, false, false, true, false, true, false,
        ];
        let packed = pack_qjl_signs(&signs);
        let unpacked = unpack_qjl_signs(&packed, signs.len());
        assert_eq!(unpacked, signs);
    }

    #[test]
    fn pack_mse_indices_fast_paths_match_generic() {
        // Exhaustively prove the bytewise fast paths produce
        // byte-for-byte identical output to the generic per-bit loop.
        // Covers every bit width 2..=7, including the production
        // (1536, 4) length and a non-multiple-of-8 boundary case.
        let mut rng = ChaCha8Rng::seed_from_u64(0xC0FFEE);
        for bits in 2..=7_u8 {
            let max_value = 1_u16 << bits;
            for &len in &[1_usize, 2, 3, 7, 8, 9, 16, 17, 257, 1536] {
                let indices = (0..len)
                    .map(|_| rng.gen_range(0..max_value))
                    .collect::<Vec<_>>();
                let dispatched = pack_mse_indices(&indices, bits);
                let generic = pack_mse_indices_generic(&indices, bits);
                assert_eq!(
                    dispatched, generic,
                    "fast path diverged from generic at bits={bits}, len={len}"
                );
                // Round-trip via the existing unpacker too — guards
                // against any accidental change to the bit ordering.
                let unpacked = unpack_mse_indices(&dispatched, len, bits);
                assert_eq!(
                    unpacked, indices,
                    "round-trip failed at bits={bits}, len={len}"
                );
            }
        }
    }

    #[test]
    fn encode_payload_length_matches_spec() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let vector = random_unit_vector(1536, 99);
        let encoded = quantizer.encode(&vector);
        let payload = quantizer.pack_payload(&encoded);
        assert_eq!(payload.len(), 772);
    }

    #[test]
    fn quantizer_1536_4bit_reallocates_qjl_budget_to_mse() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let vector = random_unit_vector(1536, 7);
        let encoded = quantizer.encode(&vector);

        assert_eq!(encoded.mse_packed.len(), 768);
        assert!(encoded.qjl_packed.is_empty());
        assert_eq!(quantizer.pack_payload(&encoded).len(), 772);
    }

    #[test]
    fn quantizer_trait_score_matches_inherent_score_ip_encoded() {
        // ADR-041 stage 0: trait-dispatched `QueryScorer::score` must
        // return bit-exact results vs. the inherent `score_ip_encoded`
        // hot path on the scoring seam.
        use crate::quant::Quantizer;
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 17);
        let candidate = random_unit_vector(1536, 18);

        let payload = quantizer.encode_code(&candidate);
        let scorer = quantizer.prepare_scorer(&query);
        let via_trait = scorer.score(&payload);

        let inherent_prepared = quantizer.prepare_ip_query(&query);
        let inherent_payload = quantizer.pack_payload(&quantizer.encode(&candidate));
        let via_inherent = quantizer.score_ip_encoded(&inherent_prepared, &inherent_payload);

        assert_eq!(via_trait.to_bits(), via_inherent.to_bits());
        assert_eq!(quantizer.code_len(), payload.len());
        assert_eq!(
            <ProdQuantizer as Quantizer>::wire_format_version(&quantizer),
            crate::am::page::INDEX_FORMAT_V1_SCALAR as u32
        );
    }

    #[test]
    fn quantizer_1536_4bit_disables_unused_qjl_and_lut_state() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 8);
        let prepared = quantizer.prepare_ip_query(&query);

        assert_eq!(quantizer.exact_score_mode(), ExactScoreMode::MseNoQjl4Bit);
        assert_eq!(quantizer.exact_score_mode_name(), "mse_no_qjl_4bit");
        assert!(quantizer.qjl_signs.is_empty());
        assert!(prepared.lut.is_empty());
        assert!(prepared.sq.is_empty());
        assert_eq!(prepared.qjl_scale, 0.0);
    }

    #[test]
    fn quantizer_1536_4bit_supports_int8_approx_query_prep() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 12);
        let prepared = quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(&query);

        assert_eq!(prepared.codebook.len(), 16);
        assert_eq!(prepared.rotated.len(), 1536);
        assert!(prepared.score_scale >= 0.0);
    }

    #[test]
    fn quantizer_1536_4bit_supports_explicit_lut_query_prep() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 16);
        let prepared = quantizer.prepare_ip_query_lut_no_qjl_4bit(&query);

        assert_eq!(prepared.lut.len(), 1536 * 16);
    }

    #[test]
    fn quantizer_1536_4bit_supports_tiled_lut_query_prep() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 17);
        let prepared = quantizer.prepare_ip_query_tiled_lut_no_qjl_4bit(&query, 512);

        assert_eq!(prepared.lut.len(), 1536 * 16);
        assert_eq!(prepared.tile_size, 512);
    }

    #[test]
    fn quantizer_1536_4bit_supports_binary_sign_query_prep() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 14);
        let prepared = quantizer.prepare_ip_query_binary_sign_no_qjl_4bit(&query);

        assert_eq!(prepared.words.len(), 24);
        assert!(quantizer.binary_sign_no_qjl_4bit_supported());
    }

    #[test]
    #[should_panic(expected = "int8 approximate query prep requires the no-QJL 4-bit lane")]
    fn int8_approx_query_prep_rejects_qjl_active_lane() {
        let quantizer = ProdQuantizer::new(32, 4, 42);
        let query = random_unit_vector(32, 13);
        let _ = quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(&query);
    }

    #[test]
    #[should_panic(expected = "explicit LUT query prep requires the no-QJL 4-bit lane")]
    fn explicit_lut_query_prep_rejects_qjl_active_lane() {
        let quantizer = ProdQuantizer::new(32, 4, 42);
        let query = random_unit_vector(32, 22);
        let _ = quantizer.prepare_ip_query_lut_no_qjl_4bit(&query);
    }

    #[test]
    #[should_panic(expected = "explicit LUT query prep requires the no-QJL 4-bit lane")]
    fn tiled_lut_query_prep_rejects_qjl_active_lane() {
        let quantizer = ProdQuantizer::new(32, 4, 42);
        let query = random_unit_vector(32, 23);
        let _ = quantizer.prepare_ip_query_tiled_lut_no_qjl_4bit(&query, 16);
    }

    #[test]
    #[should_panic(expected = "binary sign query prep requires the no-QJL 4-bit lane")]
    fn binary_sign_query_prep_rejects_qjl_active_lane() {
        let quantizer = ProdQuantizer::new(32, 4, 42);
        let query = random_unit_vector(32, 15);
        let _ = quantizer.prepare_ip_query_binary_sign_no_qjl_4bit(&query);
    }

    #[test]
    fn int8_approx_no_qjl_4bit_keeps_identical_vector_ranked_first() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 21);
        let prepared_exact = quantizer.prepare_ip_query(&query);
        let prepared_approx = quantizer.prepare_ip_query_int8_approx_no_qjl_4bit(&query);

        let mut scored = Vec::new();
        for seed in 0..16_u64 {
            let vector = if seed == 0 {
                query.clone()
            } else {
                random_unit_vector(1536, 21 + seed)
            };
            let encoded = quantizer.encode(&vector);
            let exact =
                quantizer.score_ip_from_parts(&prepared_exact, encoded.gamma, &encoded.mse_packed);
            let approx = quantizer
                .score_ip_from_parts_int8_approx_no_qjl_4bit(&prepared_approx, &encoded.mse_packed);
            scored.push((seed, exact, approx));
        }

        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .expect("exact scores should be comparable")
        });
        assert_eq!(
            scored[0].0, 0,
            "exact scorer should rank the identical vector first"
        );

        scored.sort_by(|a, b| {
            b.2.partial_cmp(&a.2)
                .expect("approx scores should be comparable")
        });
        assert_eq!(
            scored[0].0, 0,
            "approx scorer should rank the identical vector first"
        );
    }

    #[test]
    fn explicit_lut_no_qjl_4bit_matches_direct_scoring() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 31);
        let prepared_exact = quantizer.prepare_ip_query(&query);
        let prepared_lut = quantizer.prepare_ip_query_lut_no_qjl_4bit(&query);

        for seed in 0..8_u64 {
            let vector = random_unit_vector(1536, 31 + seed);
            let encoded = quantizer.encode(&vector);
            let direct =
                quantizer.score_ip_from_parts(&prepared_exact, encoded.gamma, &encoded.mse_packed);
            let lut =
                quantizer.score_ip_from_parts_lut_no_qjl_4bit(&prepared_lut, &encoded.mse_packed);
            assert!(
                (direct - lut).abs() < 1e-6,
                "explicit LUT scorer should match direct scorer: direct={direct} lut={lut}"
            );
        }
    }

    #[test]
    fn tiled_lut_no_qjl_4bit_matches_direct_scoring() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 41);
        let prepared_exact = quantizer.prepare_ip_query(&query);
        let prepared_tiled = quantizer.prepare_ip_query_tiled_lut_no_qjl_4bit(&query, 512);

        for seed in 0..8_u64 {
            let vector = random_unit_vector(1536, 41 + seed);
            let encoded = quantizer.encode(&vector);
            let direct =
                quantizer.score_ip_from_parts(&prepared_exact, encoded.gamma, &encoded.mse_packed);
            let tiled = quantizer
                .score_ip_from_parts_tiled_lut_no_qjl_4bit(&prepared_tiled, &encoded.mse_packed);
            assert!(
                (direct - tiled).abs() < 1e-6,
                "tiled LUT scorer should match direct scorer: direct={direct} tiled={tiled}"
            );
        }
    }

    #[test]
    fn binary_sign_no_qjl_4bit_keeps_identical_vector_ranked_first() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        let query = random_unit_vector(1536, 31);
        let prepared = quantizer.prepare_ip_query_binary_sign_no_qjl_4bit(&query);

        let mut scored = Vec::new();
        for seed in 0..16_u64 {
            let vector = if seed == 0 {
                query.clone()
            } else {
                random_unit_vector(1536, 31 + seed)
            };
            let encoded = quantizer.encode(&vector);
            let words = quantizer.binary_sign_words_from_packed_no_qjl_4bit(&encoded.mse_packed);
            let approx = quantizer.score_binary_sign_words_no_qjl_4bit(&prepared, &words);
            scored.push((seed, approx));
        }

        scored.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .expect("binary sign scores should be comparable")
        });
        assert_eq!(
            scored[0].0, 0,
            "binary sign scorer should rank the identical vector first"
        );
    }

    #[test]
    fn quantizer_32_4bit_keeps_qjl_and_lut_state_when_active() {
        let quantizer = ProdQuantizer::new(32, 4, 42);
        let query = random_unit_vector(32, 11);
        let prepared = quantizer.prepare_ip_query(&query);

        assert_eq!(quantizer.qjl_signs.len(), quantizer.transform_dim);
        assert!(!prepared.lut.is_empty());
        assert!(!prepared.sq.is_empty());
        assert!(prepared.qjl_scale > 0.0);
    }

    #[test]
    fn quantizer_1536_uses_tiled_working_dimension() {
        let quantizer = ProdQuantizer::new(1536, 4, 42);
        assert_eq!(quantizer.transform_dim, 1536);
        assert_eq!(quantizer.signs.len(), 1536);
    }

    #[test]
    fn encode_is_deterministic() {
        let quantizer = ProdQuantizer::new(64, 4, 42);
        let vector = random_unit_vector(64, 12);
        let payload_a = quantizer.pack_payload(&quantizer.encode(&vector));
        let payload_b = quantizer.pack_payload(&quantizer.encode(&vector));
        assert_eq!(payload_a, payload_b);
    }

    #[test]
    fn encode_decode_has_reasonable_fidelity() {
        let quantizer = ProdQuantizer::new(256, 4, 42);
        let mut total_cosine = 0.0_f32;
        for sample in 0..25 {
            let vector = random_unit_vector(256, sample);
            let payload = quantizer.pack_payload(&quantizer.encode(&vector));
            let decoded = quantizer.decode_approximate(&payload);
            total_cosine += cosine_similarity(&vector, &decoded);
        }
        let average_cosine = total_cosine / 25.0;
        assert!(
            average_cosine > 0.85,
            "average cosine similarity = {average_cosine}"
        );
    }

    #[test]
    fn prepared_query_score_matches_explicit_formula() {
        let quantizer = ProdQuantizer::new(32, 4, 42);
        let query = random_unit_vector(32, 1);
        let candidate = random_unit_vector(32, 2);
        let prepared = quantizer.prepare_ip_query(&query);
        let encoded = quantizer.encode(&candidate);
        let payload = quantizer.pack_payload(&encoded);

        let score = quantizer.score_ip_encoded(&prepared, &payload);

        let mse_indices = unpack_mse_indices(&encoded.mse_packed, 32, 3);
        let qjl_signs = unpack_qjl_signs(&encoded.qjl_packed, 32);
        let mut mse_sum = 0.0_f32;
        for (dim_index, mse_index) in mse_indices.iter().enumerate().take(32) {
            mse_sum += quantizer.codebook[*mse_index as usize] * prepared.rotated[dim_index];
        }
        let qjl_sum = prepared
            .sq
            .iter()
            .zip(qjl_signs.iter())
            .map(|(sq, sign)| if *sign { *sq } else { -*sq })
            .sum::<f32>();
        let expected = mse_sum + encoded.gamma * prepared.qjl_scale * qjl_sum;

        assert!(
            (score - expected).abs() < 1e-6,
            "score={score}, expected={expected}"
        );
    }

    #[test]
    fn score_from_parts_matches_encoded_payload_path() {
        let quantizer = ProdQuantizer::new(32, 4, 42);
        let query = random_unit_vector(32, 7);
        let candidate = quantizer.encode(&random_unit_vector(32, 8));
        let prepared = quantizer.prepare_ip_query(&query);
        let payload = quantizer.pack_payload(&candidate);
        let mut code_bytes = candidate.mse_packed.clone();
        code_bytes.extend_from_slice(&candidate.qjl_packed);

        let payload_score = quantizer.score_ip_encoded(&prepared, &payload);
        let parts_score = quantizer.score_ip_from_parts(&prepared, candidate.gamma, &code_bytes);

        assert_eq!(parts_score, payload_score);
    }

    #[test]
    fn score_from_parts_honors_supplied_gamma() {
        let quantizer = ProdQuantizer::new(32, 4, 42);
        let query = random_unit_vector(32, 9);
        let candidate = quantizer.encode(&random_unit_vector(32, 10));
        let prepared = quantizer.prepare_ip_query(&query);
        let mut code_bytes = candidate.mse_packed.clone();
        code_bytes.extend_from_slice(&candidate.qjl_packed);

        let observed = quantizer.score_ip_from_parts(&prepared, candidate.gamma, &code_bytes);
        let mutated = quantizer.score_ip_from_parts(&prepared, candidate.gamma + 1.25, &code_bytes);

        assert_ne!(observed, mutated);
    }

    #[test]
    fn code_to_code_score_is_symmetric_and_ignores_qjl() {
        let quantizer = ProdQuantizer::new(32, 4, 42);
        let a = quantizer.pack_payload(&quantizer.encode(&random_unit_vector(32, 3)));
        let mut b_encoded = quantizer.encode(&random_unit_vector(32, 4));
        let score_ab = quantizer.score_ip_encoded_lite(&a, &quantizer.pack_payload(&b_encoded));
        let score_ba = quantizer.score_ip_encoded_lite(&quantizer.pack_payload(&b_encoded), &a);
        assert_eq!(score_ab, score_ba);

        b_encoded.gamma += 5.0;
        if let Some(first) = b_encoded.qjl_packed.first_mut() {
            *first ^= 0xFF;
        }
        let score_mutated =
            quantizer.score_ip_encoded_lite(&a, &quantizer.pack_payload(&b_encoded));
        assert_eq!(score_ab, score_mutated);
    }

    #[test]
    fn raw_code_score_matches_encoded_lite_path() {
        let quantizer = ProdQuantizer::new(32, 4, 42);
        let a = quantizer.encode(&random_unit_vector(32, 5));
        let b = quantizer.encode(&random_unit_vector(32, 6));
        let mut code_a = a.mse_packed.clone();
        code_a.extend_from_slice(&a.qjl_packed);
        let mut code_b = b.mse_packed.clone();
        code_b.extend_from_slice(&b.qjl_packed);

        let encoded_score = quantizer
            .score_ip_encoded_lite(&quantizer.pack_payload(&a), &quantizer.pack_payload(&b));
        let code_score = quantizer.score_ip_codes_lite(&code_a, &code_b);

        assert_eq!(encoded_score, code_score);
    }

    #[test]
    fn cached_quantizer_reuses_instances() {
        let first = ProdQuantizer::cached(64, 4, 42);
        let second = ProdQuantizer::cached(64, 4, 42);
        assert!(Arc::ptr_eq(&first, &second));
    }

    #[test]
    fn cached_with_presence_reports_whether_entry_already_existed() {
        let (_, first_present) = ProdQuantizer::cached_with_presence(63, 5, 0xDEADBEEF_u64);
        let (_, second_present) = ProdQuantizer::cached_with_presence(63, 5, 0xDEADBEEF_u64);

        assert!(!first_present);
        assert!(second_present);
    }

    // --- Miri tests (small dimensions for speed) ---
    // Run with: cargo +nightly miri test --lib -- miri_

    #[test]
    fn miri_encode_decode_roundtrip() {
        let q = ProdQuantizer::new(8, 4, 42);
        let v = random_unit_vector(8, 99);
        let encoded = q.encode(&v);
        let payload = q.pack_payload(&encoded);
        let _ = q.decode_approximate(&payload);
    }

    #[test]
    fn miri_pack_unpack_mse() {
        let indices = vec![0u16, 3, 7, 1, 5, 2, 6, 4];
        let packed = pack_mse_indices(&indices, 3);
        let unpacked = unpack_mse_indices(&packed, 8, 3);
        assert_eq!(unpacked, indices);
    }

    #[test]
    fn miri_pack_unpack_qjl() {
        let signs = vec![true, false, true, true, false, false, true, false];
        let packed = pack_qjl_signs(&signs);
        let unpacked = unpack_qjl_signs(&packed, 8);
        assert_eq!(unpacked, signs);
    }

    #[test]
    fn miri_score_ip_encoded() {
        let q = ProdQuantizer::new(8, 4, 42);
        let query = random_unit_vector(8, 1);
        let prepared = q.prepare_ip_query(&query);
        let payload = q.pack_payload(&q.encode(&random_unit_vector(8, 2)));
        let _ = q.score_ip_encoded(&prepared, &payload);
    }

    #[test]
    fn miri_score_ip_codes_lite() {
        let q = ProdQuantizer::new(8, 4, 42);
        let enc_a = q.encode(&random_unit_vector(8, 3));
        let enc_b = q.encode(&random_unit_vector(8, 4));
        let mut code_a = enc_a.mse_packed;
        code_a.extend_from_slice(&enc_a.qjl_packed);
        let mut code_b = enc_b.mse_packed;
        code_b.extend_from_slice(&enc_b.qjl_packed);
        let _ = q.score_ip_codes_lite(&code_a, &code_b);
    }

    #[test]
    fn dispatched_score_matches_scalar_on_random_inputs() {
        let mut rng = ChaCha8Rng::seed_from_u64(77);
        let quantizers = [
            ProdQuantizer::new(32, 2, 42),
            ProdQuantizer::new(64, 3, 42),
            ProdQuantizer::new(128, 4, 42),
            ProdQuantizer::new(256, 6, 42),
            ProdQuantizer::new(256, 8, 42),
        ];

        for _ in 0..1_000 {
            let quantizer = &quantizers[rng.gen_range(0..quantizers.len())];
            let dim = quantizer.original_dim;
            let bits = quantizer.bits;
            let query = random_unit_vector(dim, rng.gen());
            let candidate = quantizer.encode(&random_unit_vector(dim, rng.gen()));
            let prepared = quantizer.prepare_ip_query(&query);
            let payload = quantizer.pack_payload(&candidate);
            let mut code_bytes = candidate.mse_packed.clone();
            code_bytes.extend_from_slice(&candidate.qjl_packed);

            let dispatched = quantizer.score_ip_encoded(&prepared, &payload);
            let scalar = quantizer.score_ip_from_split_parts_scalar(
                &prepared,
                candidate.gamma,
                &candidate.mse_packed,
                &candidate.qjl_packed,
            );
            let lite_dispatched = quantizer.score_ip_codes_lite(&code_bytes, &code_bytes);
            let lite_scalar =
                quantizer.score_ip_mse_codes_scalar(&candidate.mse_packed, &candidate.mse_packed);

            let score_scale = dispatched.abs().max(scalar.abs()).max(1.0);
            assert!(
                ((dispatched - scalar) / score_scale).abs() < 1e-6,
                "dispatched={dispatched} scalar={scalar} dim={dim} bits={bits}"
            );

            let lite_scale = lite_dispatched.abs().max(lite_scalar.abs()).max(1.0);
            assert!(
                ((lite_dispatched - lite_scalar) / lite_scale).abs() < 1e-6,
                "lite_dispatched={lite_dispatched} lite_scalar={lite_scalar} dim={dim} bits={bits}"
            );
        }
    }

    #[test]
    fn dispatched_score_matches_scalar_at_production_dims() {
        // Tolerance scales with sqrt(dim) because the AVX2 4-accumulator tree
        // reduction sums in a different order than the scalar sequential loop.
        // FP non-associativity across N terms gives expected error ~sqrt(N)*eps.
        let mut rng = ChaCha8Rng::seed_from_u64(88);
        let quantizers = [
            ProdQuantizer::new(1024, 4, 42),
            ProdQuantizer::new(1536, 4, 42),
            ProdQuantizer::new(2048, 4, 42),
        ];

        for _ in 0..100 {
            let quantizer = &quantizers[rng.gen_range(0..quantizers.len())];
            let dim = quantizer.original_dim;
            let tol = (dim as f32).sqrt() * 1e-6;
            let query = random_unit_vector(dim, rng.gen());
            let candidate = quantizer.encode(&random_unit_vector(dim, rng.gen()));
            let prepared = quantizer.prepare_ip_query(&query);
            let payload = quantizer.pack_payload(&candidate);
            let mut code_bytes = candidate.mse_packed.clone();
            code_bytes.extend_from_slice(&candidate.qjl_packed);

            let dispatched = quantizer.score_ip_encoded(&prepared, &payload);
            let scalar = quantizer.score_ip_from_split_parts_scalar(
                &prepared,
                candidate.gamma,
                &candidate.mse_packed,
                &candidate.qjl_packed,
            );
            let lite_dispatched = quantizer.score_ip_codes_lite(&code_bytes, &code_bytes);
            let lite_scalar =
                quantizer.score_ip_mse_codes_scalar(&candidate.mse_packed, &candidate.mse_packed);

            let score_scale = dispatched.abs().max(scalar.abs()).max(1.0);
            assert!(
                ((dispatched - scalar) / score_scale).abs() < tol,
                "dispatched={dispatched} scalar={scalar} dim={dim} tol={tol}"
            );

            let lite_scale = lite_dispatched.abs().max(lite_scalar.abs()).max(1.0);
            assert!(
                ((lite_dispatched - lite_scalar) / lite_scale).abs() < tol,
                "lite_dispatched={lite_dispatched} lite_scalar={lite_scalar} dim={dim} tol={tol}"
            );
        }
    }

    #[test]
    fn dispatched_score_matches_scalar_with_tail_dims() {
        let mut rng = ChaCha8Rng::seed_from_u64(89);
        // dim=40: 1×32 outer + 1×8 tail, no scalar tail
        // dim=104: 3×32 outer + 1×8 tail, no scalar tail
        // dim=108: 3×32 outer + 1×8 tail + 4 scalar tail
        // dim=100: 3×32 outer + 0×8 tail + 4 scalar tail
        let quantizers = [
            ProdQuantizer::new(40, 4, 42),
            ProdQuantizer::new(104, 4, 42),
            ProdQuantizer::new(108, 4, 42),
            ProdQuantizer::new(100, 4, 42),
        ];

        for _ in 0..1_000 {
            let quantizer = &quantizers[rng.gen_range(0..quantizers.len())];
            let dim = quantizer.original_dim;
            let query = random_unit_vector(dim, rng.gen());
            let candidate = quantizer.encode(&random_unit_vector(dim, rng.gen()));
            let prepared = quantizer.prepare_ip_query(&query);
            let payload = quantizer.pack_payload(&candidate);
            let mut code_bytes = candidate.mse_packed.clone();
            code_bytes.extend_from_slice(&candidate.qjl_packed);

            let dispatched = quantizer.score_ip_encoded(&prepared, &payload);
            let scalar = quantizer.score_ip_from_split_parts_scalar(
                &prepared,
                candidate.gamma,
                &candidate.mse_packed,
                &candidate.qjl_packed,
            );
            let lite_dispatched = quantizer.score_ip_codes_lite(&code_bytes, &code_bytes);
            let lite_scalar =
                quantizer.score_ip_mse_codes_scalar(&candidate.mse_packed, &candidate.mse_packed);

            let score_scale = dispatched.abs().max(scalar.abs()).max(1.0);
            assert!(
                ((dispatched - scalar) / score_scale).abs() < 1e-6,
                "dispatched={dispatched} scalar={scalar} dim={dim}"
            );

            let lite_scale = lite_dispatched.abs().max(lite_scalar.abs()).max(1.0);
            assert!(
                ((lite_dispatched - lite_scalar) / lite_scale).abs() < 1e-6,
                "lite_dispatched={lite_dispatched} lite_scalar={lite_scalar} dim={dim}"
            );
        }
    }

    #[test]
    fn qjl_sign_lanes_exhaustive() {
        for byte in 0u8..=255 {
            let lanes = qjl_sign_lanes(byte);
            for (bit, lane) in lanes.iter().enumerate() {
                let expected = if (byte >> bit) & 1 == 1 {
                    1.0_f32
                } else {
                    -1.0_f32
                };
                assert_eq!(
                    *lane, expected,
                    "byte={byte:#04x} bit={bit}: got {lane}, expected {expected}",
                );
            }
        }
    }
}
