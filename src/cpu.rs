//! CPU feature detection for SIMD acceleration
//!
//! Detects available CPU instruction sets at runtime and provides
//! this information to the user via GUI status bar and CLI output.
//!
//! SIMD acceleration is used automatically by the following crates:
//! - `blake3` — hashing (AVX2, AVX-512, SSE4.1)
//! - `memchr` — byte search (AVX2, SSE2)
//! - `regex` — pattern matching (AVX2, SSE4.2)
//! - `simd-json` — JSON parsing (AVX2, SSE4.2)
//! - `bytecount` — character/word counting (AVX2, SSE4.1)
//!
//! All these crates use `is_x86_feature_detected!` at runtime,
//! so a single binary works on all CPUs — no separate builds needed.

/// Detected CPU features relevant to SIMD acceleration
#[derive(Debug, Clone)]
pub struct CpuFeatures {
    /// SSE4.1 — baseline SIMD for most 64-bit CPUs (2007+)
    pub sse41: bool,
    /// SSE4.2 — adds string comparison instructions (2008+)
    pub sse42: bool,
    /// AVX — 256-bit vector registers (2011+)
    pub avx: bool,
    /// AVX2 — 256-bit integer SIMD, most important for throughput (2013+)
    pub avx2: bool,
    /// AVX-512 F — 512-bit floating-point (2017+, server/desktop HEDT)
    pub avx512f: bool,
    /// AVX-512 BW — 512-bit byte/word operations (2017+, server/desktop HEDT)
    pub avx512bw: bool,
    /// AVX-512 VL — 128/256-bit vector length extensions
    pub avx512vl: bool,
    /// NEON — ARM SIMD (all 64-bit ARM CPUs)
    pub neon: bool,
}

impl CpuFeatures {
    /// Detect CPU features at runtime using `is_x86_feature_detected!` (x86_64)
    /// or compile-time constants (aarch64).
    ///
    /// This is called once at startup and cached for the lifetime of the application.
    #[inline]
    pub fn detect() -> Self {
        // x86_64 — runtime detection via CPUID
        #[cfg(target_arch = "x86_64")]
        {
            Self {
                sse41: is_x86_feature_detected!("sse4.1"),
                sse42: is_x86_feature_detected!("sse4.2"),
                avx: is_x86_feature_detected!("avx"),
                avx2: is_x86_feature_detected!("avx2"),
                avx512f: is_x86_feature_detected!("avx512f"),
                avx512bw: is_x86_feature_detected!("avx512bw"),
                avx512vl: is_x86_feature_detected!("avx512vl"),
                neon: false,
            }
        }

        // x86 (32-bit) — limited SIMD support
        #[cfg(target_arch = "x86")]
        {
            Self {
                sse41: is_x86_feature_detected!("sse4.1"),
                sse42: is_x86_feature_detected!("sse4.2"),
                avx: false, // AVX not available on 32-bit
                avx2: false,
                avx512f: false,
                avx512bw: false,
                avx512vl: false,
                neon: false,
            }
        }

        // aarch64 — NEON is mandatory
        #[cfg(target_arch = "aarch64")]
        {
            Self {
                sse41: false,
                sse42: false,
                avx: false,
                avx2: false,
                avx512f: false,
                avx512bw: false,
                avx512vl: false,
                neon: true,
            }
        }

        // Other architectures (RISC-V, PowerPC, etc.) — no SIMD detection yet
        #[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
        {
            Self {
                sse41: false,
                sse42: false,
                avx: false,
                avx2: false,
                avx512f: false,
                avx512bw: false,
                avx512vl: false,
                neon: false,
            }
        }
    }

    /// Returns the highest SIMD level detected, for display purposes.
    ///
    /// Order: AVX-512 > AVX2 > AVX > SSE4.2 > SSE4.1 > None
    pub fn simd_level(&self) -> SimdLevel {
        if self.avx512f && self.avx512bw {
            SimdLevel::Avx512
        } else if self.avx2 {
            SimdLevel::Avx2
        } else if self.avx {
            SimdLevel::Avx
        } else if self.sse42 {
            SimdLevel::Sse42
        } else if self.sse41 {
            SimdLevel::Sse41
        } else if self.neon {
            SimdLevel::Neon
        } else {
            SimdLevel::None
        }
    }

    /// Short label for the highest SIMD level — for GUI status bar
    ///
    /// Examples: "AVX-512", "AVX2", "SSE4.2", "NEON", "Scalar"
    pub fn simd_label(&self) -> &'static str {
        match self.simd_level() {
            SimdLevel::Avx512 => "AVX-512",
            SimdLevel::Avx2 => "AVX2",
            SimdLevel::Avx => "AVX",
            SimdLevel::Sse42 => "SSE4.2",
            SimdLevel::Sse41 => "SSE4.1",
            SimdLevel::Neon => "NEON",
            SimdLevel::None => "Scalar",
        }
    }

    /// One-line summary of all detected SIMD features — for CLI output
    ///
    /// Example: "AVX2 | SSE4.2 | SSE4.1"
    pub fn summary(&self) -> String {
        let mut features = Vec::new();
        if self.avx512f && self.avx512bw {
            features.push("AVX-512");
        }
        if self.avx2 {
            features.push("AVX2");
        }
        if self.avx {
            features.push("AVX");
        }
        if self.sse42 {
            features.push("SSE4.2");
        }
        if self.sse41 {
            features.push("SSE4.1");
        }
        if self.neon {
            features.push("NEON");
        }
        if features.is_empty() {
            "No SIMD".to_string()
        } else {
            features.join(" | ")
        }
    }

    /// Detailed multi-line report — for CLI `info` command or verbose mode
    pub fn detailed_report(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("SIMD Level: {}", self.simd_label()));
        lines.push(String::new());

        #[cfg(target_arch = "x86_64")]
        {
            lines.push("x86_64 Features:".to_string());
            lines.push(format!("  SSE4.1:     {}", if self.sse41 { "Yes" } else { "No" }));
            lines.push(format!("  SSE4.2:     {}", if self.sse42 { "Yes" } else { "No" }));
            lines.push(format!("  AVX:        {}", if self.avx { "Yes" } else { "No" }));
            lines.push(format!("  AVX2:       {}", if self.avx2 { "Yes" } else { "No" }));
            lines.push(format!("  AVX-512 F:  {}", if self.avx512f { "Yes" } else { "No" }));
            lines.push(format!("  AVX-512 BW: {}", if self.avx512bw { "Yes" } else { "No" }));
            lines.push(format!("  AVX-512 VL: {}", if self.avx512vl { "Yes" } else { "No" }));
        }

        #[cfg(target_arch = "aarch64")]
        {
            lines.push("AArch64 Features:".to_string());
            lines.push(format!("  NEON:       {}", if self.neon { "Yes" } else { "No" }));
        }

        lines.push(String::new());
        lines.push("SIMD-accelerated operations:".to_string());

        // Show which operations benefit from the detected SIMD level
        let level = self.simd_level();
        match level {
            SimdLevel::Avx512 | SimdLevel::Avx2 | SimdLevel::Avx => {
                lines.push("  Hashing (blake3)    — 512/256-bit SIMD".to_string());
                lines.push("  Byte search (memchr) — 256-bit SIMD".to_string());
                lines.push("  Pattern match (regex) — 256-bit SIMD".to_string());
                if level == SimdLevel::Avx512 {
                    lines.push("  JSON parsing (simd-json) — 512-bit SIMD".to_string());
                } else {
                    lines.push("  JSON parsing (simd-json) — 256-bit SIMD".to_string());
                }
                lines.push("  Word counting (bytecount) — 256-bit SIMD".to_string());
            }
            SimdLevel::Sse42 | SimdLevel::Sse41 => {
                lines.push("  Hashing (blake3)    — 128-bit SIMD".to_string());
                lines.push("  Byte search (memchr) — 128-bit SIMD".to_string());
                lines.push("  Pattern match (regex) — 128-bit SIMD".to_string());
                lines.push("  JSON parsing (simd-json) — 128-bit SIMD".to_string());
                lines.push("  Word counting (bytecount) — 128-bit SIMD".to_string());
            }
            SimdLevel::Neon => {
                lines.push("  Hashing (blake3)    — NEON 128-bit".to_string());
                lines.push("  Byte search (memchr) — NEON 128-bit".to_string());
                lines.push("  Pattern match (regex) — NEON 128-bit".to_string());
                lines.push("  Word counting (bytecount) — NEON 128-bit".to_string());
            }
            SimdLevel::None => {
                lines.push("  No SIMD acceleration available — using scalar fallback".to_string());
            }
        }

        lines.join("\n")
    }
}

/// SIMD level hierarchy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimdLevel {
    None = 0,
    Sse41 = 1,
    Sse42 = 2,
    Neon = 3,
    Avx = 4,
    Avx2 = 5,
    Avx512 = 6,
}

impl std::fmt::Display for SimdLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SimdLevel::None => write!(f, "Scalar"),
            SimdLevel::Sse41 => write!(f, "SSE4.1"),
            SimdLevel::Sse42 => write!(f, "SSE4.2"),
            SimdLevel::Neon => write!(f, "NEON"),
            SimdLevel::Avx => write!(f, "AVX"),
            SimdLevel::Avx2 => write!(f, "AVX2"),
            SimdLevel::Avx512 => write!(f, "AVX-512"),
        }
    }
}

// ── Lazy static — detect once, cache forever ──────────────────────────────

use std::sync::OnceLock;

static CPU_FEATURES: OnceLock<CpuFeatures> = OnceLock::new();

/// Get the cached CPU features (detects on first call, returns reference after).
///
/// This is the primary API — call this from anywhere in the codebase.
#[inline]
pub fn features() -> &'static CpuFeatures {
    CPU_FEATURES.get_or_init(CpuFeatures::detect)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect() {
        let f = CpuFeatures::detect();
        // On x86_64, at least SSE4.1 should be available (all 64-bit CPUs since ~2007)
        #[cfg(target_arch = "x86_64")]
        {
            // SSE4.1 might not be available on very old x86_64 CPUs, but it's
            // extremely rare. We just test that detection doesn't crash.
            let _ = f.sse41;
        }
        // NEON is always available on aarch64
        #[cfg(target_arch = "aarch64")]
        {
            assert!(f.neon);
        }
    }

    #[test]
    fn test_cached_features() {
        let f1 = features();
        let f2 = features();
        assert!(std::ptr::eq(f1, f2), "features() should return the same reference");
    }

    #[test]
    fn test_summary_not_empty() {
        let f = CpuFeatures::detect();
        let summary = f.summary();
        assert!(!summary.is_empty());
    }

    #[test]
    fn test_simd_level_ordering() {
        assert!(SimdLevel::Avx512 > SimdLevel::Avx2);
        assert!(SimdLevel::Avx2 > SimdLevel::Avx);
        assert!(SimdLevel::Avx > SimdLevel::Sse42);
        assert!(SimdLevel::Sse42 > SimdLevel::Sse41);
        assert!(SimdLevel::Sse41 > SimdLevel::None);
    }
}
