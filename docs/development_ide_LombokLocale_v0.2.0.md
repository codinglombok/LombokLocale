# LombokLocale — Development IDE v0.2.0

## 1. Lingkungan

| Alat | Versi |
|---|---|
| Rust | stable (MSRV 1.70, diperiksa CI); `cargo-llvm-cov` untuk coverage |
| Node.js | 20 LTS atau lebih baru (CI: 20, 22, 24) |
| Python | 3.9+ (CI: 3.9, 3.11, 3.13); `pytest`, `coverage` |
| Go | 1.22+ (CI: 1.22, 1.24) |
| PHP | 8.1+ (CI: 8.1, 8.3); ekstensi `pcov` untuk coverage |
| Editor | VS Code, RustRover/GoLand/PhpStorm/PyCharm, atau editor lain |
| Bash | untuk `scripts/lombok-doctor.sh` (Windows: Git Bash atau WSL) |

## 2. Perintah

| Direktori | Perintah | Fungsi |
|---|---|---|
| `rust/` | `cargo test` | unit + doctest + runner vector |
| `rust/` | `cargo clippy --all-targets -- -D warnings` · `cargo fmt --check` | lint |
| `rust/` | `cargo llvm-cov --fail-under-lines 90` | coverage |
| `typescript/` | `npm ci` · `npm run lint` · `npm test` | tipe strict, test |
| `typescript/` | `npm run coverage` | test dengan ambang baris 90, cabang 90, fungsi 85 |
| `python/` | `python -m pytest` · `coverage run --branch --source=lomboklocale -m pytest && coverage report --fail-under=90` | test, coverage |
| `go/` | `go vet ./...` · `go test -cover ./...` | lint, test |
| `php/` | `php tests/run.php` · `php -d pcov.enabled=1 tests/run.php --coverage=90` | test, coverage |
| root | `python3 scripts/gen_ports.py` (`--check` di CI) | bangun ulang tabel data kelima port dari `data/cldr47.json` |
| root | `python3 vectors/build_vectors.py` | bangun ulang berkas vector |
| root | `node vectors/check_icu.mjs` | cek silang vector ke ICU (butuh Node.js dengan CLDR 47, mis. Node 22 ICU 77) |
| root | `python3 scripts/extract_cldr.py <cldr-json>` | ekstrak ulang `data/cldr47.json` dari unduhan `cldr-json` 47.0.0 |
| root | `bash scripts/lombok-doctor.sh LombokLocale` | pemeriksaan standar Lombok v3.6 |

## 3. Alur mengubah perilaku

1. Ubah SPEC lebih dulu (untuk data: ubah `scripts/extract_cldr.py`, lalu `gen_ports.py`).
2. Ubah model di `vectors/build_vectors.py` dan tambah kasus. Nilai harapan kasus tulis tangan harus cocok dengan model; builder menolak bila tidak.
3. Jalankan builder, perbarui `vectors/SHA256SUMS` dan hash di SPEC.
4. Ubah kelima port sampai semua runner hijau.
5. Catat di `CHANGELOG.md`.

## 4. Arah pengembangan

- Menambah locale data (bahasa Nusantara jv/su sudah, berikutnya ban, min, bug, mak dll. bila CLDR tersedia) lewat `LOCALES` di `scripts/extract_cldr.py`.
- Jam dan zona waktu, waktu relatif, daftar (`ListFormat`), satuan.
- Digit asli (`arab`, `deva`, `thai`) dan kalender Buddhis/Hijriah lewat `-u-nu`/`-u-ca`.
- Rantai `parentLocales` CLDR untuk fallback data.
- Pembaruan ke CLDR 48: ekstrak ulang data, bangun ulang vector, cek silang ke ICU yang sesuai.

*Lisensi dokumen: Apache-2.0 OR MIT · © codinglombok*
