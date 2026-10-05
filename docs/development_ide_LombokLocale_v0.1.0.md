# LombokLocale — DEVELOPMENT IDE / ARAH PENGEMBANGAN v0.1.0

"Ide" di sini = arah pengembangan (development direction), bukan editor kode.

## 1. Roadmap

| Versi | Isi | Status |
|---|---|---|
| 0.1.0 | Inti Rust + port TypeScript + vektor 1.831 kasus + 10 katalog | ✅ (belum dipublikasikan) |
| 0.2.0 | Tetapkan & lacak versi CLDR (generator data); nama bulan/hari Core-20; port Python | ⚪ |
| 0.3.0 | Port Go/PHP; WASM & C-ABI; RTL | ⚪ |
| 1.0.0 | API dibekukan | ⚪ |

## 2. Yang Sengaja Belum Dikerjakan (Deferred Scope)

Waktu/zona waktu, terbilang, kalender non-Gregorian, fungsi MF2 penuh, RTL/BiDi, C-ABI/WASM, port Python/Go/PHP.

## 3. Prinsip Desain untuk Kontributor Baru

Tetap `no_std + alloc`; tanpa dependensi; tambah data locale lewat tabel kecil, bukan kode per-bahasa yang bercabang di banyak tempat.

## 4. Cara Berkontribusi

1. Baca [SPEC_LombokLocale_v0.1.0.md](SPEC_LombokLocale_v0.1.0.md) — perilaku yang diubah harus tetap sesuai kontrak, atau kontrak diajukan perubahan lewat RFC (lihat ARCHITECTURE_UTAMA §2, aturan L0 "perubahan breaking butuh RFC").
2. Tambahkan/ubah test vector di `vectors/lomboklocale-vectors-v1.json` **sebelum** mengubah implementasi (test-first, selaras ADR-002/ADR-015).
3. Jalankan `./scripts/lombok-doctor-docs.sh LombokLocale` — PR yang mengubah API publik wajib memperbarui `API_LombokLocale_v0.1.0.md` di PR yang sama.
4. `cargo test && cargo clippy -- -D warnings` harus hijau sebelum membuka PR.

## 5. Pertanyaan Terbuka (Open Questions)

- Rilis CLDR mana yang dijadikan acuan (dan apakah data plural dihasilkan dari berkasnya)?
- Siapa penutur asli yang meninjau 8 katalog draf?
