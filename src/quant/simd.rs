//! Runtime SIMD backend detection for quantizer hot paths.

use std::env;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SimdBackend {
    Scalar,
    #[cfg(target_arch = "x86_64")]
    Avx512 {
        vpopcntdq: bool,
        bw: bool,
        bf16: bool,
    },
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
        let x86 = detect_x86_features(
            std::arch::is_x86_feature_detected!("avx2"),
            std::arch::is_x86_feature_detected!("fma"),
            std::arch::is_x86_feature_detected!("avx512f"),
            std::arch::is_x86_feature_detected!("avx512vpopcntdq"),
            std::arch::is_x86_feature_detected!("avx512bw"),
            std::arch::is_x86_feature_detected!("avx512bf16"),
        );
        if let Some(backend) = x86.avx512_backend() {
            return backend;
        }
        if has_avx2_fma(x86.avx2, x86.fma) {
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

#[cfg(any(target_arch = "x86_64", test))]
fn has_avx2_fma(avx2: bool, fma: bool) -> bool {
    avx2 && fma
}

#[cfg(any(target_arch = "x86_64", test))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct X86SimdFeatures {
    pub(crate) avx2: bool,
    pub(crate) fma: bool,
    pub(crate) avx512f: bool,
    pub(crate) avx512vpopcntdq: bool,
    pub(crate) avx512bw: bool,
    pub(crate) avx512bf16: bool,
}

#[cfg(any(target_arch = "x86_64", test))]
pub(crate) fn detect_x86_features(
    avx2: bool,
    fma: bool,
    avx512f: bool,
    avx512vpopcntdq: bool,
    avx512bw: bool,
    avx512bf16: bool,
) -> X86SimdFeatures {
    X86SimdFeatures {
        avx2,
        fma,
        avx512f,
        avx512vpopcntdq,
        avx512bw,
        avx512bf16,
    }
}

#[cfg(any(target_arch = "x86_64", test))]
impl X86SimdFeatures {
    pub(crate) fn has_avx512_bits1(&self) -> bool {
        self.avx512f && self.avx512vpopcntdq
    }

    pub(crate) fn has_avx512_bits4(&self) -> bool {
        self.avx512f && self.avx512bw
    }

    pub(crate) fn has_avx512_bits8(&self) -> bool {
        self.avx512f
    }

    pub(crate) fn has_avx512_bf16(&self) -> bool {
        self.avx512f && self.avx512bf16
    }

    #[cfg(target_arch = "x86_64")]
    fn avx512_backend(self) -> Option<SimdBackend> {
        if self.has_avx512_bits1() || self.has_avx512_bits4() || self.has_avx512_bits8() {
            Some(SimdBackend::Avx512 {
                vpopcntdq: self.avx512vpopcntdq,
                bw: self.avx512bw,
                bf16: self.has_avx512_bf16(),
            })
        } else {
            None
        }
    }
}

fn forced_backend_from_env() -> Option<SimdBackend> {
    let requested = env::var("ECAZ_SIMD").ok()?;
    match requested.trim().to_ascii_lowercase().as_str() {
        "" | "auto" => None,
        "scalar" => Some(SimdBackend::Scalar),
        #[cfg(target_arch = "x86_64")]
        "avx512" | "avx512f" => Some(SimdBackend::Avx512 {
            vpopcntdq: true,
            bw: true,
            bf16: false,
        }),
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
        SimdBackend::Avx512 {
            bf16: true,
            vpopcntdq: true,
            bw: true,
        } => "avx512f+vpopcntdq+bw+bf16",
        #[cfg(target_arch = "x86_64")]
        SimdBackend::Avx512 {
            vpopcntdq: true,
            bw: true,
            bf16: false,
        } => "avx512f+vpopcntdq+bw",
        #[cfg(target_arch = "x86_64")]
        SimdBackend::Avx512 {
            vpopcntdq: true,
            bw: false,
            bf16: _,
        } => "avx512f+vpopcntdq",
        #[cfg(target_arch = "x86_64")]
        SimdBackend::Avx512 {
            vpopcntdq: false,
            bw: true,
            bf16: _,
        } => "avx512f+bw",
        #[cfg(target_arch = "x86_64")]
        SimdBackend::Avx512 {
            vpopcntdq: false,
            bw: false,
            bf16: _,
        } => "avx512f",
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
        with_simd_env(Some(" auto "), || {
            assert_eq!(forced_backend_from_env(), None)
        });
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

        #[cfg(target_arch = "x86_64")]
        with_simd_env(Some("avx512"), || {
            assert_eq!(
                forced_backend_from_env(),
                Some(SimdBackend::Avx512 {
                    vpopcntdq: true,
                    bw: true,
                    bf16: false,
                })
            );
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
    fn x86_backend_gate_requires_avx2_and_fma() {
        assert!(has_avx2_fma(true, true));
        assert!(!has_avx2_fma(true, false));
        assert!(!has_avx2_fma(false, true));
        assert!(!has_avx2_fma(false, false));
    }

    #[test]
    fn x86_feature_slots_model_task67_kernel_requirements() {
        let all = detect_x86_features(true, true, true, true, true, true);
        assert!(all.has_avx512_bits1());
        assert!(all.has_avx512_bits4());
        assert!(all.has_avx512_bits8());
        assert!(all.has_avx512_bf16());

        let no_vpopcnt = detect_x86_features(true, true, true, false, true, true);
        assert!(!no_vpopcnt.has_avx512_bits1());
        assert!(no_vpopcnt.has_avx512_bits4());
        assert!(no_vpopcnt.has_avx512_bits8());

        let no_bw = detect_x86_features(true, true, true, true, false, false);
        assert!(no_bw.has_avx512_bits1());
        assert!(!no_bw.has_avx512_bits4());
        assert!(no_bw.has_avx512_bits8());
        assert!(!no_bw.has_avx512_bf16());

        let avx2_only = detect_x86_features(true, true, false, true, true, true);
        assert!(!avx2_only.has_avx512_bits1());
        assert!(!avx2_only.has_avx512_bits4());
        assert!(!avx2_only.has_avx512_bits8());
        assert!(!avx2_only.has_avx512_bf16());
    }

    #[test]
    fn backend_name_matches_detected_backend() {
        match backend() {
            SimdBackend::Scalar => assert_eq!(backend_name(), "scalar"),
            #[cfg(target_arch = "x86_64")]
            SimdBackend::Avx512 { .. } => assert!(backend_name().starts_with("avx512f")),
            #[cfg(target_arch = "x86_64")]
            SimdBackend::Avx2Fma => assert_eq!(backend_name(), "avx2+fma"),
            #[cfg(target_arch = "aarch64")]
            SimdBackend::Neon => assert_eq!(backend_name(), "neon"),
        }
    }
}
