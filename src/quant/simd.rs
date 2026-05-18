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
    let requested = env::var("ECAZ_SIMD").ok()?;
    match requested.trim().to_ascii_lowercase().as_str() {
        "" | "auto" => None,
        "scalar" => Some(SimdBackend::Scalar),
        #[cfg(target_arch = "x86_64")]
        "avx2" | "avx2_fma" | "avx2+fma" => Some(SimdBackend::Avx2Fma),
        #[cfg(target_arch = "aarch64")]
        "neon" => Some(SimdBackend::Neon),
        other => panic!("unsupported ECAZ_SIMD value: {other}"),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::panic;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn with_simd_env<T>(value: Option<&str>, f: impl FnOnce() -> T) -> T {
        let _guard = ENV_LOCK.lock().expect("ECAZ_SIMD test lock poisoned");
        let previous = env::var_os("ECAZ_SIMD");
        set_simd_env(value.map(OsString::from));
        let result = f();
        set_simd_env(previous);
        result
    }

    fn set_simd_env(value: Option<OsString>) {
        match value {
            Some(value) => env::set_var("ECAZ_SIMD", value),
            None => env::remove_var("ECAZ_SIMD"),
        }
    }

    #[test]
    fn forced_backend_accepts_auto_empty_and_absent() {
        with_simd_env(None, || assert_eq!(forced_backend_from_env(), None));
        with_simd_env(Some(""), || assert_eq!(forced_backend_from_env(), None));
        with_simd_env(Some(" auto "), || assert_eq!(forced_backend_from_env(), None));
    }

    #[test]
    fn forced_backend_accepts_supported_backend_names() {
        with_simd_env(Some("SCALAR"), || {
            assert_eq!(forced_backend_from_env(), Some(SimdBackend::Scalar));
        });

        #[cfg(target_arch = "x86_64")]
        with_simd_env(Some("avx2+fma"), || {
            assert_eq!(forced_backend_from_env(), Some(SimdBackend::Avx2Fma));
        });

        #[cfg(target_arch = "aarch64")]
        with_simd_env(Some("neon"), || {
            assert_eq!(forced_backend_from_env(), Some(SimdBackend::Neon));
        });
    }

    #[test]
    fn forced_backend_rejects_unknown_name() {
        with_simd_env(Some("bogus"), || {
            let result = panic::catch_unwind(forced_backend_from_env);
            assert!(result.is_err());
        });
    }

    #[test]
    fn detect_backend_honors_forced_scalar() {
        with_simd_env(Some("scalar"), || {
            assert_eq!(detect_backend(), SimdBackend::Scalar);
        });
    }

    #[test]
    fn backend_name_matches_detected_backend() {
        match backend() {
            SimdBackend::Scalar => assert_eq!(backend_name(), "scalar"),
            #[cfg(target_arch = "x86_64")]
            SimdBackend::Avx2Fma => assert_eq!(backend_name(), "avx2+fma"),
            #[cfg(target_arch = "aarch64")]
            SimdBackend::Neon => assert_eq!(backend_name(), "neon"),
        }
    }
}
