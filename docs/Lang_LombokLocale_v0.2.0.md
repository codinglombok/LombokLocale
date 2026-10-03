# LombokLocale — Bahasa & i18n v0.2.0

| Atribut | Nilai |
|---|---|
| Versi | 0.2.0 |
| Peran | inti i18n ekosistem: library lain memakai katalog, `resolve`, dan format dari sini |
| Tingkat i18n (masterplan §13) | **A** untuk kemampuan (plural semua locale CLDR, format 28 locale); katalog pesan library sendiri: en + id ditinjau, 8 draf |
| Data | CLDR 47.0.0 (Unicode License v3) |
| Cakupan katalog saat ini | en, id (ditinjau); de, es, fr, ja, ko, pt, ru, zh-Hans (draf mesin) — 10/20; Nusantara: id saja |

## 1. Locale data (SPEC §1)

| Kelompok | Locale |
|---|---|
| Asia Tenggara | `id`, `ms`, `jv`, `su`, `vi`, `th` |
| Asia Timur dan Selatan | `zh`, `zh-Hant`, `ja`, `ko`, `hi`, `en-IN` |
| Eropa | `en-GB`, `de`, `fr`, `es`, `pt-PT`, `it`, `nl`, `ru`, `uk`, `pl`, `tr`, `sv`, `cy` |
| Lainnya | `en`, `pt`, `ar` |

## 2. Prinsip

1. Program MUST memeriksa kode error dan `code` pesan, bukan teks.
2. Spasi dalam angka dan tanggal mengikuti CLDR (U+00A0 atau U+202F); jangan menggantinya dengan spasi biasa.
3. `ar` memakai digit Latin dan tanda arah (U+200E/U+200F) dari CLDR; tampilan RTL diserahkan ke lapisan tampilan.
4. Pesan library ini sendiri ada di `locales/<tag>/lomboklocale.json`; kunci dan placeholder diperiksa sama dengan `en` di CI.

## 3. Katalog ID pesan library ini

| ID | en | id |
|---|---|---|
| `locale.negotiation.no_match` | No matching locale found for {requested}; using {fallback} | Tidak ada locale yang cocok untuk {requested}; memakai {fallback} |
| `catalog.missing_message` | Missing translation for {messageId} | Terjemahan untuk {messageId} tidak ditemukan |

## 4. Rencana

- Data angka dan tanggal untuk bahasa Nusantara lain begitu tersedia di CLDR (mis. `ban`, `min`, `bug`, `mad`).
- Peninjauan penutur asli untuk 8 katalog draf.
- Digit asli dan kalender lain (`-u-nu`, `-u-ca`).

*Lisensi dokumen: Apache-2.0 OR MIT · © codinglombok*
