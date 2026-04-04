//! Lloyd-Max optimal scalar quantizer codebook generation.
//!
//! Generates centroids for the per-coordinate distribution after SRHT rotation.
//! The distribution is a scaled Beta distribution on [-1, 1] determined by the
//! vector dimension — no training data needed.
//!
//! Extracted from TurboQuantDB. Original: src/quantizer/codebook.rs

use std::f64::consts::PI;

/// Numerically stable log-gamma using Lanczos approximation (g=7, n=8 coefficients).
fn log_gamma(z: f64) -> f64 {
    debug_assert!(z > 0.0, "log_gamma undefined for z <= 0, got {z}");
    if z < 0.5 {
        return (PI / (PI * z).sin()).ln() - log_gamma(1.0 - z);
    }
    let p = [
        676.5203681218851,
        -1259.1392167224028,
        771.32342877765313,
        -176.61502916214059,
        12.507343278226905,
        -0.13857109526572012,
        9.9843695780195716e-6,
        1.5056327351493116e-7,
    ];
    let mut y = 0.99999999999980993;
    for (i, &val) in p.iter().enumerate() {
        y += val / (z + i as f64);
    }
    let t = z + 6.5;
    0.5 * (2.0 * PI).ln() + (z - 0.5) * t.ln() - t + y.ln()
}

/// Beta distribution PDF for the post-SRHT per-coordinate distribution.
/// f_X(x) = (Γ(d/2) / (√π · Γ((d-1)/2))) · (1 - x²)^((d-3)/2)
pub fn beta_pdf(x: f64, d: usize) -> f64 {
    if d < 2 || x.abs() >= 1.0 {
        return 0.0;
    }
    let df = d as f64;
    let log_coeff = log_gamma(df / 2.0) - 0.5 * PI.ln() - log_gamma((df - 1.0) / 2.0);
    let base = (1.0 - x * x).max(0.0);
    if base == 0.0 {
        return 0.0;
    }
    let log_term = ((df - 3.0) / 2.0) * base.ln();
    (log_coeff + log_term).exp()
}

/// Compute optimal Lloyd-Max centroids for `2^b` levels under the Beta distribution
/// parameterized by dimension `d`.
pub fn lloyd_max(b: usize, d: usize, num_points: usize) -> Vec<f64> {
    let num_centroids = 1 << b;
    let step = 2.0 / (num_points as f64 - 1.0);

    let grid: Vec<(f64, f64)> = (0..num_points)
        .map(|i| {
            let x = -1.0 + i as f64 * step;
            let prob = beta_pdf(x, d) * step;
            (x, prob)
        })
        .filter(|(_, p)| *p > 1e-10)
        .collect();

    let bound = (3.0 / (d as f64).sqrt()).min(0.99);
    let mut centroids: Vec<f64> = (0..num_centroids)
        .map(|i| -bound + (2.0 * bound) * (i as f64 + 0.5) / (num_centroids as f64))
        .collect();

    let mut boundaries = vec![0.0; num_centroids - 1];

    let max_iter = 1000;
    let tol = 1e-7;
    for _ in 0..max_iter {
        for i in 0..(num_centroids - 1) {
            boundaries[i] = (centroids[i] + centroids[i + 1]) / 2.0;
        }

        let mut next_centroids = vec![0.0; num_centroids];
        let mut masses = vec![0.0; num_centroids];

        for &(x, p) in &grid {
            let mut cluster_idx = 0;
            for i in 0..(num_centroids - 1) {
                if x > boundaries[i] {
                    cluster_idx = i + 1;
                }
            }
            next_centroids[cluster_idx] += x * p;
            masses[cluster_idx] += p;
        }

        let mut max_diff = 0.0_f64;
        for i in 0..num_centroids {
            if masses[i] > 0.0 {
                let new_c = next_centroids[i] / masses[i];
                max_diff = max_diff.max((new_c - centroids[i]).abs());
                centroids[i] = new_c;
            }
        }

        if max_diff < tol {
            break;
        }
    }

    centroids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_gamma_known_values() {
        assert!((log_gamma(1.0)).abs() < 1e-9);
        assert!((log_gamma(2.0)).abs() < 1e-9);
        let expected = 24.0_f64.ln();
        assert!((log_gamma(5.0) - expected).abs() < 1e-7);
    }

    #[test]
    fn beta_pdf_integrates_to_one() {
        let d = 1536;
        let n = 100_000;
        let step = 2.0 / (n as f64 - 1.0);
        let sum: f64 = (0..n)
            .map(|i| beta_pdf(-1.0 + i as f64 * step, d) * step)
            .sum();
        assert!((sum - 1.0).abs() < 1e-3, "integral = {sum}");
    }

    #[test]
    fn lloyd_max_b1_centroids_symmetric() {
        let centroids = lloyd_max(1, 1536, 20_000);
        assert_eq!(centroids.len(), 2);
        assert!((centroids[0] + centroids[1]).abs() < 1e-3);
    }

    #[test]
    fn beta_pdf_out_of_range() {
        assert_eq!(beta_pdf(1.0, 16), 0.0);
        assert_eq!(beta_pdf(-1.0, 16), 0.0);
        assert_eq!(beta_pdf(1.5, 16), 0.0);
    }
}
