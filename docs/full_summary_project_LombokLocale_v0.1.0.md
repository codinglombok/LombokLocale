# LombokLocale — FULL SUMMARY PROJECT v0.1.0

Ringkasan satu-halaman untuk pembaca baru (reviewer, calon kontributor, aplikasi yang mengevaluasi dependensi) — tidak perlu membaca 11 dokumen lain untuk memahami apa, mengapa, dan status proyek ini.

## Apa Ini?

Inti i18n zero-dependency: negosiasi locale BCP-47, aturan plural CLDR (subset), runtime MessageFormat 2-lite, format angka/mata uang/tanggal, dan katalog pesan JSON (`locales/<bcp47>/<repo>.json`) yang dipakai identik oleh semua library dan port.

## Mengapa Dibuat (bukan pakai library pihak ketiga)?

ICU/ICU4X terlalu besar untuk MCU dan tidak menjamin keluaran byte-identik lintas 5 bahasa; FormatJS hanya JS. Locale membatasi diri pada subset yang dibutuhkan ekosistem sehingga bisa `no_std`, kecil, dan diverifikasi vector.

## Fitur Utama

*Library ini mandiri dan universal (PRINSIP_UNIVERSAL U1); tidak menjadi bagian dari aplikasi/framework mana pun.*

- BCP-47 parse + negosiasi urutan RFC 4647 (`zh-Hant-TW` → `zh-Hant` → `zh-TW` → `zh`)
- Plural kardinal CLDR (subset) + **ordinal** (en/fr/it/sv/hu) + `selectordinal`
- MessageFormat 2-lite: placeholder, plural, select, selectordinal, `#`; batas kedalaman 16
- Angka/mata uang/tanggal per-locale; `NaN`/∞/nilai besar ditangani tanpa panic; tanggal ISO ketat (kalender Gregorian, tahun kabisat)
- Katalog JSON ketat (sisa karakter ditolak, `\\uXXXX` 4 digit heksa) + `resolve()` (`code` + `messageId`)
- 10 katalog: en, id (sumber) + 8 draf terjemahan mesin

## Status Saat Ini

| Aspek | Nilai |
|---|---|
| Status | 🔵 BUILDING — inti Rust + port TypeScript lulus vektor bersama; belum dipublikasikan ke GitHub/registry |
| Versi | 0.1.0 |
| Bahasa referensi | Rust |
| Port tersedia | TypeScript ✅ (lulus seluruh vektor); Python, Go, PHP — folder stub, belum ada kode |
| Test | 54 test Rust (unit + robustness 40.000 iterasi + paritas katalog + 1.831 vektor); port TypeScript lulus 1.831 vektor yang sama; `cargo build --no-default-features` (no_std) lulus; `clippy -D warnings` bersih; uji mutasi pada port TS: perubahan aturan plural dan pengelompokan angka tertangkap |
| Dependensi wajib | — (L0: tanpa dependensi Lombok wajib) |
| Tingkat (tier) | L0 |

## Contoh Pemakai (ilustrasi, bukan kepemilikan)

- Siapa pun yang menampilkan teks kepada manusia; tidak ada pemakai yang memiliki library ini.
- Contoh dependen di ekosistem Lombok dicatat hanya di `map_` (arah dependensi).

## Batasan yang Diketahui (Known Limitations)

- Aturan plural ditulis tangan berdasarkan CLDR; **versi CLDR acuan belum dinyatakan/diverifikasi** terhadap rilis tertentu. Aturan Slavia disederhanakan; input `u64`.
- Nama bulan hanya `id`/`en`; locale lain memakai nama Inggris.
- Simbol mata uang hanya USD, IDR, EUR, GBP, JPY, CNY.
- MF2-lite bukan MessageFormat 2 penuh (tanpa fungsi/deklarasi/markup).
- Argumen bertipe float pada port TS ditampilkan dengan aturan JS (`String(n)`): untuk |x| ≥ 1e21 atau < 1e-6 bisa berbeda dari Rust (yang tidak memakai notasi eksponen). Tidak ada di vektor.
- Terjemahan 8 bahasa masih draf mesin; RTL/BiDi belum ada.
- Port Python/Go/PHP belum ada; C-ABI/WASM belum dibuat.

## Untuk Info Lebih Lanjut

- Cara pakai → [Guide How to Use](guide_how_to_use_LombokLocale_v0.1.0.md)
- Cara instal/deploy → [How to Dist](how_to_dist_LombokLocale_v0.1.0.md)
- API lengkap → [API](API_LombokLocale_v0.1.0.md)
- Kontrak lintas bahasa → [SPEC](SPEC_LombokLocale_v0.1.0.md)

## Gap vs Pembanding (U6)

| Pembanding | Kelebihan pembanding | Posisi LombokLocale (jujur) |
|---|---|---|
| ICU / ICU4X | Cakupan CLDR penuh, zona waktu, kalender | Belum setara; Locale unggul pada ukuran kecil, `no_std + alloc`, tanpa dependensi, katalog satu berkas identik lintas port |
| FormatJS / gettext | Ekosistem alat & ekstraksi pesan | Belum ada alat ekstraksi; kontrak `code` + `messageId` bersifat lintas bahasa |
| Keunikan yang diklaim | — | Satu katalog + satu perilaku terverifikasi vector di banyak bahasa (baru Rust yang ada) |
