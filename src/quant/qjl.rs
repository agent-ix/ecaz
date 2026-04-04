//! QJL-stage helpers.

use crate::quant::rotation::{inverse_srht, pad_input, srht};

pub fn qjl_project(input: &[f32], signs: &[f32]) -> Vec<f32> {
    let padded = pad_input(input, signs.len());
    srht(&padded, signs)
}

pub fn decode_mse_only(rotated_domain: &[f32], signs: &[f32], dim: usize) -> Vec<f32> {
    let decoded = inverse_srht(rotated_domain, signs);
    decoded[..dim].to_vec()
}
