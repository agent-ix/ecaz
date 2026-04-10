//! Scalar MSE-stage helpers.

use crate::quant::CodeIndex;

pub fn nearest_centroid_index(codebook: &[f32], value: f32) -> CodeIndex {
    let mut best_index = 0usize;
    let mut best_distance = f32::INFINITY;
    for (index, centroid) in codebook.iter().enumerate() {
        let distance = (value - *centroid).abs();
        if distance < best_distance {
            best_distance = distance;
            best_index = index;
        }
    }
    best_index as CodeIndex
}

pub fn quantize_to_indices(codebook: &[f32], rotated: &[f32], dim: usize) -> Vec<CodeIndex> {
    let mut out = vec![0_u16; dim];
    quantize_to_indices_into(codebook, rotated, &mut out);
    out
}

/// Reusable-buffer variant of `quantize_to_indices`. Writes
/// `out.len()` indices into `out`, reading the first `out.len()`
/// elements of `rotated`. The caller owns the output buffer.
pub fn quantize_to_indices_into(codebook: &[f32], rotated: &[f32], out: &mut [CodeIndex]) {
    debug_assert!(
        rotated.len() >= out.len(),
        "quantize_to_indices_into: rotated buffer too short"
    );
    for (slot, value) in out.iter_mut().zip(rotated.iter()) {
        *slot = nearest_centroid_index(codebook, *value);
    }
}

pub fn decode_indices(codebook: &[f32], indices: &[CodeIndex]) -> Vec<f32> {
    let mut out = vec![0.0_f32; indices.len()];
    decode_indices_into(codebook, indices, &mut out);
    out
}

/// Reusable-buffer variant of `decode_indices`. Writes one decoded
/// centroid value per input index into `out`. The caller owns the
/// output buffer.
pub fn decode_indices_into(codebook: &[f32], indices: &[CodeIndex], out: &mut [f32]) {
    debug_assert_eq!(
        out.len(),
        indices.len(),
        "decode_indices_into: output buffer length mismatch"
    );
    for (slot, index) in out.iter_mut().zip(indices.iter()) {
        *slot = codebook[*index as usize];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nearest_centroid_index_prefers_lower_index_on_tie() {
        let codebook = [-1.0_f32, 0.0, 1.0];
        assert_eq!(nearest_centroid_index(&codebook, 0.5), 1);
    }
}
