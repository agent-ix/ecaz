//! Runtime SIMD backend detection for quantizer hot paths.

use std::env;
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
    if let Some(forced) = forced_backend_from_env() {
        return forced;
    }

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

fn forced_backend_from_env() -> Option<SimdBackend> {
    let requested = env::var("TQVECTOR_SIMD").ok()?;
    match requested.trim().to_ascii_lowercase().as_str() {
        "" | "auto" => None,
        "scalar" => Some(SimdBackend::Scalar),
        #[cfg(target_arch = "x86_64")]
        "avx2" | "avx2_fma" | "avx2+fma" => Some(SimdBackend::Avx2Fma),
        #[cfg(target_arch = "aarch64")]
        "neon" => Some(SimdBackend::Neon),
        other => panic!("unsupported TQVECTOR_SIMD value: {other}"),
    }
}

pub(crate) fn backend_name() -> &'static str {
    match backend() {
        SimdBackend::Scalar => "scalar",
        #[cfg(target_arch = "x86_64")]
        SimdBackend::Avx2Fma => "avx2+fma",
        #[cfg(target_arch = "aarch64")]
        SimdBackend::Neon => "neon",
    }
}
