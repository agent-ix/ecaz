//! Seeded SRHT helpers.

use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use crate::quant::hadamard::orthonormal_fwht_in_place;

pub fn transform_dim(dim: usize) -> usize {
    dim.max(1).next_power_of_two()
}

pub fn sign_vector(len: usize, seed: u64) -> Vec<f32> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..len)
        .map(|_| if rng.gen::<bool>() { 1.0 } else { -1.0 })
        .collect()
}

pub fn srht(input: &[f32], signs: &[f32]) -> Vec<f32> {
    assert_eq!(input.len(), signs.len(), "srht input/sign length mismatch");
    let mut workspace = input.to_vec();
    for (value, sign) in workspace.iter_mut().zip(signs) {
        *value *= *sign;
    }
    orthonormal_fwht_in_place(&mut workspace);
    workspace
}

pub fn inverse_srht(input: &[f32], signs: &[f32]) -> Vec<f32> {
    assert_eq!(
        input.len(),
        signs.len(),
        "inverse srht input/sign length mismatch"
    );
    let mut workspace = input.to_vec();
    orthonormal_fwht_in_place(&mut workspace);
    for (value, sign) in workspace.iter_mut().zip(signs) {
        *value *= *sign;
    }
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
}
