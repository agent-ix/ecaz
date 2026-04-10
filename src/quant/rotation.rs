//! Seeded SRHT helpers.

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::quant::hadamard::{orthonormal_fwht_in_place, orthonormal_fwht_tiled_in_place};

pub const TILED_FWHT_COMPAT_DIM: usize = 1536;
pub const TILED_FWHT_COMPAT_TILE_DIM: usize = 512;

pub fn transform_dim(dim: usize) -> usize {
    dim.max(1).next_power_of_two()
}

pub fn tile_dim(dim: usize) -> Option<usize> {
    if dim == TILED_FWHT_COMPAT_DIM {
        Some(TILED_FWHT_COMPAT_TILE_DIM)
    } else {
        None
    }
}

pub fn effective_transform_dim(dim: usize) -> usize {
    tile_dim(dim).map_or_else(|| transform_dim(dim), |_| dim)
}

pub fn sign_vector(len: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..len)
        .map(|_| if rng.gen::<bool>() { 1.0 } else { -1.0 })
        .collect()
}

/// In-place forward SRHT. The caller owns the workspace buffer; this
/// function does not allocate. Equivalent to `srht(input, signs)` when
/// the workspace already contains a copy of `input`.
pub fn srht_in_place(workspace: &mut [f32], signs: &[f32]) {
    assert_eq!(
        workspace.len(),
        signs.len(),
        "srht input/sign length mismatch"
    );
    for (value, sign) in workspace.iter_mut().zip(signs) {
        *value *= *sign;
    }
    if let Some(tile_size) = tile_dim(workspace.len()) {
        orthonormal_fwht_tiled_in_place(workspace, tile_size);
    } else {
        orthonormal_fwht_in_place(workspace);
    }
}

/// In-place inverse SRHT. The caller owns the workspace buffer; this
/// function does not allocate. Equivalent to `inverse_srht(input, signs)`
/// when the workspace already contains a copy of `input`.
pub fn inverse_srht_in_place(workspace: &mut [f32], signs: &[f32]) {
    assert_eq!(
        workspace.len(),
        signs.len(),
        "inverse srht input/sign length mismatch"
    );
    if let Some(tile_size) = tile_dim(workspace.len()) {
        orthonormal_fwht_tiled_in_place(workspace, tile_size);
    } else {
        orthonormal_fwht_in_place(workspace);
    }
    for (value, sign) in workspace.iter_mut().zip(signs) {
        *value *= *sign;
    }
}

pub fn srht(input: &[f32], signs: &[f32]) -> Vec<f32> {
    let mut workspace = input.to_vec();
    srht_in_place(&mut workspace, signs);
    workspace
}

pub fn inverse_srht(input: &[f32], signs: &[f32]) -> Vec<f32> {
    let mut workspace = input.to_vec();
    inverse_srht_in_place(&mut workspace, signs);
    workspace
}

pub fn pad_input(input: &[f32], padded_len: usize) -> Vec<f32> {
    let mut padded = vec![0.0_f32; padded_len];
    padded[..input.len()].copy_from_slice(input);
    padded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn srht_preserves_norm() {
        let input = vec![0.5_f32, -1.5, 2.0, 3.0, -0.5];
        let padded = pad_input(&input, transform_dim(input.len()));
        let signs = sign_vector(padded.len(), 42);
        let rotated = srht(&padded, &signs);
        let input_norm = padded.iter().map(|v| v * v).sum::<f32>().sqrt();
        let output_norm = rotated.iter().map(|v| v * v).sum::<f32>().sqrt();
        let rel_err = ((input_norm - output_norm) / input_norm.max(1.0)).abs();
        assert!(rel_err < 1e-5, "relative error = {rel_err}");
    }

    #[test]
    fn tiled_srht_roundtrip_1536() {
        let input = (0..1536)
            .map(|i| ((i as f32) * 0.013).cos())
            .collect::<Vec<_>>();
        let signs = sign_vector(input.len(), 42);
        let rotated = srht(&input, &signs);
        let recovered = inverse_srht(&rotated, &signs);

        for (expected, actual) in input.iter().zip(recovered.iter()) {
            assert!(
                (*expected - *actual).abs() < 1e-4,
                "mismatch: expected {expected}, got {actual}"
            );
        }
    }
}
