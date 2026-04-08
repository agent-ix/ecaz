//! Runtime SIMD backend detection for quantizer hot paths.

use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SimdBackend {
    Scalar,
    #[cfg(target_arch = "x86_64")]
    Avx2Fma,
    #[cfg(target_arch = "aarch64")]
    Neon,
}

pub(crate) fn backend() -> SimdBackend {
    static BACKEND: OnceLock<SimdBackend> = OnceLock::new();
    *BACKEND.get_or_init(detect_backend)
}

fn detect_backend() -> SimdBackend {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2") && std::arch::is_x86_feature_detected!("fma")
        {
            return SimdBackend::Avx2Fma;
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if std::arch::is_aarch64_feature_detected!("neon") {
            return SimdBackend::Neon;
        }
    }

    SimdBackend::Scalar
}
