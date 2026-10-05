# LombokLocale — LANG (i18n) v0.1.0

Tingkat i18n, katalog ID pesan, dan cakupan bahasa untuk `LombokLocale` (ARCHITECTURE_UTAMA §7, ADR-009: "Library L1+ → hanya menyimpan ID pesan + teks en; terjemahan di-load via LombokLocale").

## 1. Tingkat i18n Repo Ini

**Tingkat F** menurut MASTERPLAN_UTAMA §13 (teks tampil ke pengguna; wajib katalog Core-20 + RTL) dan sekaligus *penyedia* mesin i18n untuk library lain. **Kesenjangan jujur:** katalog baru `en` + `id` (2 dari 20 bahasa Core-20; belum ada RTL).

## 2. Katalog ID Pesan

| `messageId` | Teks EN (default) | Digunakan di |
|---|---|---|
| `locale.negotiation.no_match` | No matching locale found for {requested}; using {fallback} | (dicadangkan) |
| `catalog.missing_message` | Missing translation for {messageId} | (dicadangkan) |

## 3. Cakupan Bahasa (Core-20 + Nusantara)

| Bahasa | Kode BCP-47 | Status katalog |
|---|---|---|
| Bahasa Indonesia | `id` | ✅ `locales/id/lomboklocale.json` |
| English | `en` | ✅ `locales/en/lomboklocale.json` (default/sumber kebenaran) |
| Español, Français, Deutsch, Português, Русский, 日本語, 한국어, 简体中文 | `es` `fr` `de` `pt` `ru` `ja` `ko` `zh-Hans` | 🟡 **draf terjemahan mesin — wajib ditinjau penutur asli** (`locales/STATUS.md`) |
| Core-20 lainnya (`ar`, `hi`, `bn`, `tr`, `vi`, `th`, `it`, `pl`, `nl`, `uk`) + Nusantara (`jv`, `su`, `ms`) | — | ⚪ belum ada (`ar` butuh RTL) |

## 4. Cara Menambah Bahasa Baru

1. Salin `locales/en/lomboklocale.json` ke `locales/<bcp47>/lomboklocale.json`.
2. Terjemahkan setiap value; jangan ubah `messageId` (key).
3. Jalankan test integrasi katalog (`cargo test catalog`) — memverifikasi setiap `messageId` di §2 punya entri di file baru.
4. Perbarui tabel §3 di dokumen ini pada PR yang sama.

## 5. Ketergantungan pada LombokLocale

Repo ini *adalah* LombokLocale; tidak bergantung pada dirinya sendiri.
