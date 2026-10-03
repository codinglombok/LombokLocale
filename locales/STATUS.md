# Katalog pesan LombokLocale

| Bahasa | Kode | Status |
|---|---|---|
| English | `en` | sumber kebenaran |
| Bahasa Indonesia | `id` | ditulis dan ditinjau pemilik ekosistem |
| Español, Français, Deutsch, Português, Русский, 日本語, 한국어, 简体中文 | `es` `fr` `de` `pt` `ru` `ja` `ko` `zh-Hans` | draf terjemahan mesin; wajib ditinjau penutur asli sebelum dianggap final |

Aturan: kunci (`messageId`) dan placeholder (`{requested}`, `{fallback}`, `{messageId}`) harus identik dengan `en`, dan setiap berkas harus lolos `parse_catalog`. Keduanya diperiksa oleh `rust/tests/catalogs.rs` di CI.

Belum ada: `ar`, `hi`, `bn`, `tr`, `vi`, `th`, `it`, `pl`, `nl`, `uk`, dan bahasa Nusantara (`jv`, `su`, `ms`). Kontribusi penutur asli diterima lewat pull request.
