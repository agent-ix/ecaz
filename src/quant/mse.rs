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
    rotated[..dim]
        .iter()
        .map(|value| nearest_centroid_index(codebook, *value))
        .collect()
}

pub fn decode_indices(codebook: &[f32], indices: &[CodeIndex]) -> Vec<f32> {
    indices
        .iter()
        .map(|index| codebook[*index as usize])
        .collect()
}
