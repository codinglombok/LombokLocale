# LombokLocale — Full Summary Project v0.2.0

| Item | Nilai |
|---|---|
| Deskripsi | Tag BCP 47 dan negosiasi, aturan plural CLDR 47, format angka/persen/mata uang/tanggal, subset ICU MessageFormat, dan katalog pesan JSON, dengan hasil sama di 5 bahasa; tanpa dependensi runtime |
| Cluster · tingkat | 01 Fondasi & Runtime · L0 |
| Referensi | SPEC + model Python di `vectors/build_vectors.py` + pemeriksaan silang ICU 77.1 (`vectors/check_icu.mjs`) |
| Port | Rust (`no_std` + `alloc`), TypeScript, Python, Go, PHP |
| Vector | 342 kasus · SHA-256 `0808e475...5d8da898` |
| Test | Rust 8 tes API + runner vector + tes katalog · TS 350 · Python 350 · Go tes API + runner vector · PHP 58 cek API + 343 cek vector |
| Coverage | Rust 95,9% baris · TS 98,8% baris / 96,2% cabang · Python 97% · Go 94,7% · PHP 96,9% baris |
| Registry | crates.io, npm, PyPI `lomboklocale`; Go `github.com/codinglombok/lomboklocale/go`; Packagist `codinglombok/lomboklocale` (semua belum terbit) |
| Lisensi | kode Apache-2.0 OR MIT; data CLDR Unicode License v3 |

## 1. Tabel gap vs pembanding (jujur)

| Kemampuan | LombokLocale 0.2.0 | ICU4X (Rust) | Intl (JS) | Babel (Python) | golang.org/x/text | intl ICU (PHP) |
|---|---|---|---|---|---|---|
| Hasil identik lintas bahasa (SPEC + vector) | YA | TIDAK | TIDAK | TIDAK | TIDAK | TIDAK |
| Versi CLDR terkunci di paket | YA (47) | YA | bergantung runtime | YA | YA | bergantung ICU sistem |
| Aturan plural semua locale | YA | YA | YA | YA | YA | YA |
| Format angka/mata uang/tanggal | 28 locale | semua | semua | semua | sebagian | semua |
| Waktu, zona waktu, relatif, daftar, satuan | TIDAK | YA | YA | YA | sebagian | YA |
| Kalender non-Gregorian, digit non-Latin | TIDAK | YA | YA | YA | sebagian | YA |
| MessageFormat | subset ICU | TIDAK | TIDAK (bawaan) | TIDAK (gettext) | TIDAK | YA (MessageFormatter) |
| Angka sebagai desimal eksak | YA | YA (fixed_decimal) | YA (string, ES2023) | YA (Decimal) | TIDAK | parsial |
| Tanpa dependensi native | YA | YA | bawaan | YA | YA | butuh ekstensi intl |

Posisi unik yang dibuktikan test: teks yang sama, sampai ke spasi tak putus, untuk angka, mata uang, tanggal, plural dan pesan di lima bahasa, dengan versi data tertanam yang sama.

## 2. Batasan yang Diketahui

1. Data angka/tanggal hanya untuk 28 locale; locale lain diformat sebagai `en` (tetapi memakai aturan pluralnya sendiri).
2. Hanya digit Latin dan kalender Gregorian; `-u-nu`/`-u-ca` diabaikan (tanggal `th` memakai tahun Masehi, angka `ar` memakai digit Latin).
3. Tidak ada jam, zona waktu, waktu relatif, daftar, satuan, notasi ringkas atau ilmiah, mata uang accounting.
4. MessageFormat adalah subset ICU: tanpa `choice`, `spellout`, gaya `currency`; bukan sintaks MessageFormat 2.
5. Rantai fallback data mengikuti pemotongan tag, bukan `parentLocales` CLDR (mis. `en-AU` memakai `en`, bukan `en-001`; `pt-AO` memakai `pt`, bukan `pt-PT`).
6. Empat perbedaan dengan ICU 77.1 tercatat di SPEC §9.
7. Katalog selain `en` dan `id` adalah draf terjemahan mesin.
8. Paket PHP berada di subdirektori; Packagist butuh repositori split.

## 3. Prinsip Universal (ringkas, untuk publik)

| Prinsip | Status | Bukti |
|---|---|---|
| U1 Mandiri | YA | README tanpa klaim kepemilikan |
| U2 Modern | YA | CLDR 47.0.0, RFC 5646/4647, tanggal tinjauan di SPEC |
| U3 Multi-platform | YA | CI ubuntu/windows/macos; Rust `no_std` + `alloc` |
| U4 Multi-bahasa | YA | 5 port, runner vector di semua port |
| U5 Rentang skala | YA | perangkat kecil (`no_std`) sampai server |
| U6 Lengkap & unik | SEBAGIAN | tabel gap di atas |
| U7 Aman & teruji | YA | SPEC §10; `forbid(unsafe_code)`; coverage ≥ 94% di semua port; cek silang ICU |
| U8 Ekosistem tanpa kopling | YA | 0 dependensi wajib |
| U9 Internasional | YA | inti i18n ekosistem; Lang_ |
| U10 Lisensi | YA | Apache-2.0 OR MIT + Unicode License v3 untuk data |
| U11 Siap registri | SEBAGIAN | crates/npm/PyPI/Go siap; Packagist butuh split repo |
| U12 Dokumentasi | YA | 10 dokumen publik + 2 internal |
| U13 Kerahasiaan & dokumen bersih | YA | `lombok-doctor.sh`: 0 emoji, `.gitignore` ADR-024 |

*Lisensi dokumen: Apache-2.0 OR MIT · © codinglombok*
