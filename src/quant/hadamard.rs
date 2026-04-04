//! Fast Walsh-Hadamard Transform primitives.

/// In-place unnormalized FWHT.
///
/// The input length must be a power of two.
pub fn fwht_in_place(values: &mut [f32]) {
    assert!(
        values.len().is_power_of_two(),
        "fwht input length must be a power of two, got {}",
        values.len()
    );

    let mut width = 1;
    while width < values.len() {
        let step = width * 2;
        for chunk in values.chunks_exact_mut(step) {
            let (left, right) = chunk.split_at_mut(width);
            for i in 0..width {
                let a = left[i];
                let b = right[i];
                left[i] = a + b;
                right[i] = a - b;
            }
        }
        width = step;
    }
}

/// Apply the orthonormalized FWHT in place.
pub fn orthonormal_fwht_in_place(values: &mut [f32]) {
    fwht_in_place(values);
    let scale = (values.len() as f32).sqrt().recip();
    for value in values {
        *value *= scale;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fwht_preserves_norm_after_normalization() {
        let input = vec![1.0_f32, -2.0, 0.5, 3.25, -1.5, 4.0, 2.0, -0.25];
        let mut rotated = input.clone();
        orthonormal_fwht_in_place(&mut rotated);

        let input_norm = input.iter().map(|v| v * v).sum::<f32>().sqrt();
        let output_norm = rotated.iter().map(|v| v * v).sum::<f32>().sqrt();
        let rel_err = ((input_norm - output_norm) / input_norm.max(1.0)).abs();
        assert!(rel_err < 1e-5, "relative error = {rel_err}");
    }
}
