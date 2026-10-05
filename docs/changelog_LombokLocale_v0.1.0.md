# LombokLocale — CHANGELOG

Format mengikuti [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) dan [SemVer](https://semver.org/). Versi ini dirilis lewat `release-please` (ADR-011) — commit `feat:`/`fix:` men-trigger bump otomatis; jangan edit versi manual di sini.

*Entri terbaru selalu di bagian paling depan (PRINSIP_UNIVERSAL U12).*

## [0.1.0] — 2026-09-28

### Added
- Port TypeScript (lulus 1.831 vektor Rust-generated); uji mutasi
- Plural ordinal + `selectordinal`; `Catalog::iter`
- 8 katalog draf terjemahan mesin + uji paritas kunci/placeholder
- Pseudo-fuzz seluruh API publik; batas kedalaman pesan

### Changed
- Lisensi menjadi `Apache-2.0 OR MIT` (keputusan pemilik; MASTERPLAN_UTAMA §10.1); berkas `LICENSE-APACHE` + `LICENSE-MIT`

### Fixed
- `format_float` panic pada `decimals` besar/NaN/∞/nilai raksasa; `-0.00` pada negatif kecil
- `parse_iso_date` menerima `2026-9-7` dan `2026-02-30`
- `parse_catalog` menerima sisa karakter setelah `}` dan `\\u+041`
- `negotiate` melewati `zh-Hant` saat meminta `zh-Hant-TW`
- Build `no_std` (`f64::powi/round`)

### Security
- —

---

*Untuk riwayat lengkap tiap versi, lihat [GitHub Releases](https://github.com/codinglombok/LombokLocale/releases).*
