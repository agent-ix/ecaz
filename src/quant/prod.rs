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
    pub sq: Vec<f32>,
    pub qjl_scale: f32,
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

        let transform_dim = rotation::transform_dim(dim);
        let codebook = codebook::lloyd_max((bits - 1) as usize, dim, 20_000)
            .into_iter()
            .map(|value| value as f32)
            .collect();
        let signs = rotation::sign_vector(transform_dim, seed);
        let qjl_signs = rotation::sign_vector(transform_dim, seed ^ 0x9E37_79B9_7F4A_7C15);

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

    pub fn encode(&self, vector: &[f32]) -> EncodedTq {
        assert_eq!(
            vector.len(),
            self.original_dim,
            "vector length mismatch: got {}, expected {}",
            vector.len(),
            self.original_dim
        );

        let padded = rotation::pad_input(vector, self.transform_dim);
        let rotated = rotation::srht(&padded, &self.signs);
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

        let qjl_projection = qjl::qjl_project(&residual, &self.qjl_signs);
        let qjl_signs = qjl_projection[..self.original_dim]
            .iter()
            .map(|value| *value >= 0.0)
            .collect::<Vec<_>>();

        EncodedTq {
            gamma,
            mse_packed: pack_mse_indices(&mse_indices, self.bits - 1),
            qjl_packed: pack_qjl_signs(&qjl_signs),
        }
    }

    #[allow(dead_code)]
    pub fn decode_approximate(&self, payload: &[u8]) -> Vec<f32> {
        let (_, mse_packed, _) = self.split_payload(payload);
        let mse_indices = unpack_mse_indices(mse_packed, self.original_dim, self.bits - 1);
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

        let rotated = rotation::srht(&rotation::pad_input(query, self.transform_dim), &self.signs);
        let qjl_projection = qjl::qjl_project(query, &self.qjl_signs);
        let num_centroids = 1usize << (self.bits - 1);

        let mut lut = Vec::with_capacity(self.original_dim * num_centroids);
        for value in &rotated[..self.original_dim] {
            for centroid in &self.codebook {
                lut.push(*centroid * *value);
            }
        }

        PreparedQuery {
            lut,
            sq: qjl_projection[..self.original_dim].to_vec(),
            qjl_scale: (PI / 2.0).sqrt() / self.original_dim as f32,
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

    fn score_ip_from_split_parts(
        &self,
        prepared: &PreparedQuery,
        gamma: f32,
        mse_packed: &[u8],
        qjl_packed: &[u8],
    ) -> f32 {
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

    fn score_ip_from_split_parts_scalar(
        &self,
        prepared: &PreparedQuery,
        gamma: f32,
        mse_packed: &[u8],
        qjl_packed: &[u8],
    ) -> f32 {
        let num_centroids = 1usize << (self.bits - 1);

        let mut mse_sum = 0.0_f32;
        for dim_index in 0..self.original_dim {
            let centroid_index = mse_index_at(mse_packed, dim_index, self.bits - 1) as usize;
            mse_sum += prepared.lut[dim_index * num_centroids + centroid_index];
        }

        let mut qjl_sum = 0.0_f32;
        for dim_index in 0..self.original_dim {
            let sign = if qjl_sign_at(qjl_packed, dim_index) {
                1.0
            } else {
                -1.0
            };
            qjl_sum += prepared.sq[dim_index] * sign;
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
        match backend() {
            #[cfg(target_arch = "x86_64")]
            SimdBackend::Avx2Fma => unsafe { self.score_ip_mse_codes_avx2(mse_a, mse_b) },
            #[cfg(target_arch = "aarch64")]
            SimdBackend::Neon => unsafe { self.score_ip_mse_codes_neon(mse_a, mse_b) },
            SimdBackend::Scalar => self.score_ip_mse_codes_scalar(mse_a, mse_b),
        }
    }

    fn score_ip_mse_codes_scalar(&self, mse_a: &[u8], mse_b: &[u8]) -> f32 {
        let mut mse_sum = 0.0_f32;
        for dim_index in 0..self.original_dim {
            let idx_a = mse_index_at(mse_a, dim_index, self.bits - 1) as usize;
            let idx_b = mse_index_at(mse_b, dim_index, self.bits - 1) as usize;
            mse_sum += self.codebook[idx_a] * self.codebook[idx_b];
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
        let qjl_len = qjl_code_len(self.original_dim);
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
        let qjl_len = qjl_code_len(self.original_dim);
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
        use std::arch::x86_64::{_mm256_loadu_ps, _mm256_mul_ps, _mm256_storeu_ps};

        let bits_per_index = self.bits - 1;
        let num_centroids = 1usize << bits_per_index;
        let mut mse_sum = 0.0_f32;
        let mut qjl_sum = 0.0_f32;
        let mut dim_index = 0usize;

        while dim_index + 8 <= self.original_dim {
            let mut mse_values = [0.0_f32; 8];
            let mut sign_values = [-1.0_f32; 8];
            for lane in 0..8 {
                let absolute = dim_index + lane;
                let centroid_index = mse_index_at(mse_packed, absolute, bits_per_index) as usize;
                mse_values[lane] = prepared.lut[absolute * num_centroids + centroid_index];
                sign_values[lane] = if qjl_sign_at(qjl_packed, absolute) {
                    1.0
                } else {
                    -1.0
                };
            }

            let mut qjl_terms = [0.0_f32; 8];
            _mm256_storeu_ps(
                qjl_terms.as_mut_ptr(),
                _mm256_mul_ps(
                    _mm256_loadu_ps(prepared.sq.as_ptr().add(dim_index)),
                    _mm256_loadu_ps(sign_values.as_ptr()),
                ),
            );

            for lane in 0..8 {
                mse_sum += mse_values[lane];
                qjl_sum += qjl_terms[lane];
            }
            dim_index += 8;
        }

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

        mse_sum + gamma * prepared.qjl_scale * qjl_sum
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2,fma")]
    unsafe fn score_ip_mse_codes_avx2(&self, mse_a: &[u8], mse_b: &[u8]) -> f32 {
        use std::arch::x86_64::{_mm256_loadu_ps, _mm256_mul_ps, _mm256_storeu_ps};

        let bits_per_index = self.bits - 1;
        let mut sum = 0.0_f32;
        let mut dim_index = 0usize;

        while dim_index + 8 <= self.original_dim {
            let mut lhs = [0.0_f32; 8];
            let mut rhs = [0.0_f32; 8];
            for lane in 0..8 {
                let absolute = dim_index + lane;
                let idx_a = mse_index_at(mse_a, absolute, bits_per_index) as usize;
                let idx_b = mse_index_at(mse_b, absolute, bits_per_index) as usize;
                lhs[lane] = self.codebook[idx_a];
                rhs[lane] = self.codebook[idx_b];
            }

            let mut products = [0.0_f32; 8];
            _mm256_storeu_ps(
                products.as_mut_ptr(),
                _mm256_mul_ps(_mm256_loadu_ps(lhs.as_ptr()), _mm256_loadu_ps(rhs.as_ptr())),
            );
            for product in products {
                sum += product;
            }
            dim_index += 8;
        }

        while dim_index < self.original_dim {
            let idx_a = mse_index_at(mse_a, dim_index, bits_per_index) as usize;
            let idx_b = mse_index_at(mse_b, dim_index, bits_per_index) as usize;
            sum += self.codebook[idx_a] * self.codebook[idx_b];
            dim_index += 1;
        }
        sum
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
        use std::arch::aarch64::{vld1q_f32, vmulq_f32, vst1q_f32};

        let bits_per_index = self.bits - 1;
        let num_centroids = 1usize << bits_per_index;
        let mut mse_sum = 0.0_f32;
        let mut qjl_sum = 0.0_f32;
        let mut dim_index = 0usize;

        while dim_index + 4 <= self.original_dim {
            let mut mse_values = [0.0_f32; 4];
            let mut signs = [0.0_f32; 4];
            for lane in 0..4 {
                let absolute = dim_index + lane;
                let centroid_index = mse_index_at(mse_packed, absolute, bits_per_index) as usize;
                mse_values[lane] = prepared.lut[absolute * num_centroids + centroid_index];
                signs[lane] = if qjl_sign_at(qjl_packed, absolute) {
                    1.0
                } else {
                    -1.0
                };
            }

            let mut qjl_terms = [0.0_f32; 4];
            vst1q_f32(
                qjl_terms.as_mut_ptr(),
                vmulq_f32(
                    vld1q_f32(prepared.sq.as_ptr().add(dim_index)),
                    vld1q_f32(signs.as_ptr()),
                ),
            );
            for lane in 0..4 {
                mse_sum += mse_values[lane];
                qjl_sum += qjl_terms[lane];
            }
            dim_index += 4;
        }

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

        mse_sum + gamma * prepared.qjl_scale * qjl_sum
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn score_ip_mse_codes_neon(&self, mse_a: &[u8], mse_b: &[u8]) -> f32 {
        use std::arch::aarch64::{vld1q_f32, vmulq_f32, vst1q_f32};

        let bits_per_index = self.bits - 1;
        let mut sum = 0.0_f32;
        let mut dim_index = 0usize;

        while dim_index + 4 <= self.original_dim {
            let mut lhs = [0.0_f32; 4];
            let mut rhs = [0.0_f32; 4];
            for lane in 0..4 {
                let absolute = dim_index + lane;
                let idx_a = mse_index_at(mse_a, absolute, bits_per_index) as usize;
                let idx_b = mse_index_at(mse_b, absolute, bits_per_index) as usize;
                lhs[lane] = self.codebook[idx_a];
                rhs[lane] = self.codebook[idx_b];
            }

            let mut products = [0.0_f32; 4];
            vst1q_f32(
                products.as_mut_ptr(),
                vmulq_f32(vld1q_f32(lhs.as_ptr()), vld1q_f32(rhs.as_ptr())),
            );
            for product in products {
                sum += product;
            }
            dim_index += 4;
        }

        while dim_index < self.original_dim {
            let idx_a = mse_index_at(mse_a, dim_index, bits_per_index) as usize;
            let idx_b = mse_index_at(mse_b, dim_index, bits_per_index) as usize;
            sum += self.codebook[idx_a] * self.codebook[idx_b];
            dim_index += 1;
        }
        sum
    }
}

pub fn mse_code_len(dim: usize, bits: u8) -> usize {
    let bits_per_index = (bits as usize).saturating_sub(1);
    (dim * bits_per_index).div_ceil(8)
}

pub fn qjl_code_len(dim: usize) -> usize {
    dim.div_ceil(8)
}

pub fn payload_len(dim: usize, bits: u8) -> usize {
    4 + mse_code_len(dim, bits) + qjl_code_len(dim)
}

pub fn pack_mse_indices(indices: &[CodeIndex], bits_per_index: u8) -> Vec<u8> {
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

fn qjl_sign_at(packed: &[u8], dim_index: usize) -> bool {
    (packed[dim_index / 8] >> (dim_index % 8)) & 1 == 1
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
    fn qjl_pack_unpack_roundtrip() {
        let signs = vec![
            true, false, true, true, false, false, true, false, true, false,
        ];
        let packed = pack_qjl_signs(&signs);
        let unpacked = unpack_qjl_signs(&packed, signs.len());
        assert_eq!(unpacked, signs);
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
        let num_centroids = 1usize << (quantizer.bits - 1);
        let mut mse_sum = 0.0_f32;
        for (dim_index, mse_index) in mse_indices.iter().enumerate().take(32) {
            mse_sum += prepared.lut[dim_index * num_centroids + *mse_index as usize];
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
}
