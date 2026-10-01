# Katalog terjemahan

| Bahasa | Kode | Status |
|---|---|---|
| English | `en` | ✅ sumber kebenaran |
| Bahasa Indonesia | `id` | ✅ ditulis pemilik ekosistem |
| Español, Français, Deutsch, Português, Русский, 日本語, 한국어, 简体中文 | `es` `fr` `de` `pt` `ru` `ja` `ko` `zh-Hans` | 🟡 **draf terjemahan mesin — wajib ditinjau penutur asli** sebelum dianggap final (MASTERPLAN_UTAMA §13) |

Aturan: kunci (`messageId`) dan placeholder (`{field}`, `{min}`, …) harus identik dengan `en`; diperiksa oleh `cargo test` (tests/catalogs.rs).
Belum ada dari Core-20: `ar` (butuh RTL/BiDi), `hi`, `bn`, `tr`, `vi`, `th`, `it`, `pl`, `nl`, `uk`, dan bahasa Nusantara (`jv`, `su`, `ms`).
