# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-04

### Added

- Initial release: `Q` (signed), `UQ` (unsigned), and `CQ` (signed complex)
  fixed-point types with type-level (`typenum`) integer/fractional widths and
  overflow-free widening arithmetic.
- Checked, `wrapping_*`, and `saturating_*` operations, plus `truncate` /
  `saturate` narrowing back to a chosen width.
- `core::iter::Sum` and checked `try_sum` reductions into a caller-chosen wider
  accumulator for `Q`, `UQ`, and `CQ`.
- Memory-compact packed containers — `PackedArray` / `PackedVec` (byte-granular)
  and `PackedBits` / `PackedBitsVec` (bit-exact) — for bulk fixed-point storage.
- Optional `serde` and `alloc` feature flags.
