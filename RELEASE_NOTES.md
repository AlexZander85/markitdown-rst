## MDrust v1.4.0 — SIMD Acceleration & Auto-Detection

This release adds SIMD-accelerated parsing with automatic CPU feature detection, making MDrust even faster on modern processors.

### New Features

- **SIMD-accelerated JSON parsing** — `simd-json` replaces `serde_json` for 2–5× faster JSON parsing on x86_64 and aarch64 CPUs. Falls back to `serde_json` on 32-bit x86 automatically.
- **SIMD-optimized regex** — `regex` crate now uses `perf-inline` and `perf-literal` features for SIMD-accelerated pattern matching (AVX2, SSE4.2).
- **SIMD word/byte counting** — `bytecount` crate accelerates character and word counting with AVX2/SSE4.1/NEON instructions.
- **CPU feature detection** — New `src/cpu.rs` module with `CpuFeatures::detect()` using `is_x86_feature_detected!` (x86_64) and compile-time constants (aarch64). Detects: AVX-512 F/BW/VL, AVX2, AVX, SSE4.2, SSE4.1, NEON.
- **GUI status bar — SIMD indicator** — The bottom status bar now shows the detected SIMD level (e.g., "SIMD: AVX2" in green, "SIMD: AVX-512" in green, "SIMD: SSE4.2" in yellow).
- **CLI `cpu-info` command** — New `mdrust-cli cpu-info` command shows detailed CPU SIMD capabilities and which operations are accelerated:
  ```
  CPU SIMD Features

  SIMD Level: AVX2

  x86_64 Features:
    SSE4.1:     Yes
    SSE4.2:     Yes
    AVX:        Yes
    AVX2:       Yes
    AVX-512 F:  No
    AVX-512 BW: No
    AVX-512 VL: No

  SIMD-accelerated operations:
    Hashing (blake3)    — 256-bit SIMD
    Byte search (memchr) — 256-bit SIMD
    Pattern match (regex) — 256-bit SIMD
    JSON parsing (simd-json) — 256-bit SIMD
    Word counting (bytecount) — 256-bit SIMD
  ```
- **`simd` feature flag** — SIMD acceleration is enabled by default (`default = ["gui", "ocr", "preview", "simd"]`). Can be disabled with `--no-default-features` for scalar-only builds.

### Already SIMD-accelerated (existing, now visible)

These crates already used SIMD internally before v1.4.0 but weren't advertised:
- `blake3` — content hashing (AVX-512, AVX2, SSE4.1, NEON)
- `memchr` — byte search (AVX2, SSE2, NEON)

### Technical Details

- **Runtime detection** — All SIMD usage is runtime-detected via `is_x86_feature_detected!` / `std::arch::is_aarch64_feature_detected!`. A single binary works on all CPUs — no separate builds needed.
- **`OnceLock` caching** — CPU features are detected once at startup and cached for the lifetime of the application.
- **Aligned buffers** — `simd-json` requires aligned input buffers; the code handles re-allocation automatically.
- **Scalar fallback** — On CPUs without SIMD support (very old x86, exotic architectures), all operations fall back to scalar implementations.

### Downloads

| File | Edition | OS | Arch |
|------|---------|----|------|
| `mdrust-full-linux-x64.tar.gz` | Full (GUI + OCR + Preview) | Linux | x86_64 |
| `mdrust-full-macos-x64.tar.gz` | Full | macOS | x86_64 |
| `mdrust-full-windows-x64.exe` | Full | Windows | x86_64 |
| `mdrust-light-linux-x64.tar.gz` | Light (GUI, no OCR) | Linux | x86_64 |
| `mdrust-light-macos-x64.tar.gz` | Light | macOS | x86_64 |
| `mdrust-light-windows-x64.exe` | Light | Windows | x86_64 |
| `mdrust-cli-linux-x64.tar.gz` | CLI-only (OCR) | Linux | x86_64 |
| `mdrust-cli-macos-x64.tar.gz` | CLI-only | macOS | x86_64 |
| `mdrust-cli-windows-x64.exe` | CLI-only | Windows | x86_64 |

---

**Full Changelog**: https://github.com/AlexZander85/MDrust/compare/v1.3.0...v1.4.0
