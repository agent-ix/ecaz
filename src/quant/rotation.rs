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
    apply_signs_in_place_scalar(&mut workspace, signs);
    orthonormal_fwht_in_place(&mut workspace);
    workspace
}

pub fn srht_padded(input: &[f32], signs: &[f32]) -> Vec<f32> {
    assert!(
        input.len() <= signs.len(),
        "srht_padded input longer than signs: got {}, max {}",
        input.len(),
        signs.len()
    );
    if input.len() == signs.len() {
        return srht(input, signs);
    }

    let mut workspace = vec![0.0_f32; signs.len()];
    for ((value, input_value), sign) in workspace[..input.len()]
        .iter_mut()
        .zip(input.iter())
        .zip(signs.iter())
    {
        *value = *input_value * *sign;
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
    apply_signs_in_place_scalar(&mut workspace, signs);
    workspace
}

pub fn pad_input(input: &[f32], padded_len: usize) -> Vec<f32> {
    let mut padded = vec![0.0_f32; padded_len];
    padded[..input.len()].copy_from_slice(input);
    padded
}

fn apply_signs_in_place_scalar(values: &mut [f32], signs: &[f32]) {
    for (value, sign) in values.iter_mut().zip(signs) {
        *value *= *sign;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quant::hadamard::fwht_in_place_scalar;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    use rand::Rng;

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
    fn srht_runtime_path_matches_scalar_on_random_inputs() {
        let mut rng = ChaCha8Rng::seed_from_u64(44);
        for _ in 0..1_000 {
            let size = 1usize << rng.gen_range(1..=9);
            let input = (0..size)
                .map(|_| rng.gen_range(-10.0_f32..10.0_f32))
                .collect::<Vec<_>>();
            let signs = sign_vector(size, rng.gen());

            let mut scalar = input.clone();
            apply_signs_in_place_scalar(&mut scalar, &signs);
            fwht_in_place_scalar(&mut scalar);
            let scale = (size as f32).sqrt().recip();
            for value in &mut scalar {
                *value *= scale;
            }

            let dispatched = srht(&input, &signs);

            for (lhs, rhs) in scalar.iter().zip(dispatched.iter()) {
                let scale = lhs.abs().max(rhs.abs()).max(1.0);
                assert!(
                    ((lhs - rhs) / scale).abs() < 1e-6,
                    "lhs={lhs} rhs={rhs} size={size}"
                );
            }
        }
    }

    #[test]
    fn srht_padded_matches_pad_input_plus_srht() {
        let mut rng = ChaCha8Rng::seed_from_u64(45);
        for _ in 0..1_000 {
            let dim = rng.gen_range(3..=257);
            let input = (0..dim)
                .map(|_| rng.gen_range(-10.0_f32..10.0_f32))
                .collect::<Vec<_>>();
            let padded_len = transform_dim(dim);
            let signs = sign_vector(padded_len, rng.gen());

            let padded = pad_input(&input, padded_len);
            let expected = srht(&padded, &signs);
            let actual = srht_padded(&input, &signs);

            for (lhs, rhs) in expected.iter().zip(actual.iter()) {
                let scale = lhs.abs().max(rhs.abs()).max(1.0);
                assert!(
                    ((lhs - rhs) / scale).abs() < 1e-6,
                    "lhs={lhs} rhs={rhs} dim={dim} padded_len={padded_len}"
                );
            }
        }
    }
}
