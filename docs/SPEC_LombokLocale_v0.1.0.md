# LombokLocale — SPEC v0.1.0

> This document is the normative cross-language contract. Every language port MUST produce byte-identical output for all specified inputs. Deviations from this specification are bugs.

(Kalimat di atas wajib verbatim per MASTERPLAN_UTAMA_v3.3 §5 — jangan diterjemahkan atau diparafrase.)

## 1. Vektor Uji (Test Vectors)

| Atribut | Nilai |
|---|---|
| Berkas | `vectors/lomboklocale-vectors-v1.json` |
| SHA-256 | `5df3a761b07ecdeca280653f420bcb2d453e04b144058c22e43f54e7e256d6e9` |
| Diverifikasi CI oleh | `lombok-ci.yml` job `docs-and-vectors` (`sha256sum -c`) |

Setiap perubahan pada `vectors/lomboklocale-vectors-v1.json` **wajib** memperbarui SHA-256 di atas pada PR yang sama, atau CI gagal (ADR-015).

## 2. Kontrak per Fungsi

- **`parse_bcp47`**: `trim` (Unicode White_Space) lalu pisah pada `-`/`_`; language 2–8 huruf ASCII → huruf kecil; script 4 huruf → Titlecase; region 2 huruf/3 digit → HURUF BESAR; varian ≥4 alfanumerik → huruf kecil; selain itu `InvalidSubtag`; kosong → `Empty`.
- **`negotiate`**: untuk tiap tag diminta (urutan prioritas; tag tak valid dilewati) coba: tag penuh → `bahasa[-script]-region` → `bahasa[-script]` → `bahasa-region` (bila ada script) → `bahasa`; duplikat berurutan dibuang; cocok tanpa peka huruf besar-kecil ASCII; kembalikan entri `available` apa adanya; tak ada → `default`.
- **`plural_category` / `ordinal_category`**: tabel keluarga bahasa pada `plural.rs`/`index.ts` (bahasa = subtag sebelum `-`/`_`, peka huruf); `u64` eksak.
- **`format_message`**: `{name}` → tampilan argumen; `{v, plural|selectordinal, k{..}}` memilih kunci = kategori, lalu `other`, lalu `*`; `#` diganti angka (semua kemunculan dalam teks terpilih); `{v, select, k{..}}` cocok string persis, lalu `*`, lalu `other`; urutan galat: variabel hilang → kurung tak seimbang pada kasus → tipe selector; kedalaman > 16 → `TooDeep`.
- **`format_integer/float`**: pemisah grup & desimal per keluarga bahasa; pembulatan setengah-naik pada |x|; `decimals` dijepit ke 15; `NaN` → `NaN`, ±∞ → `±∞`, |x| ≥ 1e20 → notasi ilmiah (`1e30`); negatif yang dibulatkan ke nol tidak bertanda.
- **`format_currency`**: simbol dari tabel; `en`/`id` simbol di depan; `fr/sv/pl/fi` jumlah + NBSP + simbol; lainnya `simbol␠jumlah`.
- **`parse_iso_date`**: tepat `YYYY-MM-DD` (digit ASCII), hari valid menurut kalender Gregorian termasuk tahun kabisat.
- **`parse_catalog`**: objek JSON datar string→string; sisa karakter non-spasi setelah `}` → `UnexpectedChar`; `\\uXXXX` wajib 4 digit heksa (selain itu dibuang; surrogate dibuang).
- **`resolve`**: id tak ada atau pola cacat → `!!<message_id>!!`.

## 3. Batasan yang Didefinisikan secara Eksplisit (Explicitly Defined Edge Cases)

- `parse_bcp47("")` → `Empty`; `"en--US"` → `InvalidSubtag`.
- `i64::MIN` diformat benar; `format_float(1.0, 40, …)` tidak panic (dijepit).
- `2026-02-29` ditolak, `2028-02-29` diterima; `2026-9-7` ditolak.
- Pola bersarang > 16 tingkat → `TooDeep`, bukan stack overflow.

## 4. Non-Goals (di luar kontrak normatif ini)

Kesesuaian penuh CLDR/ICU, MessageFormat 2 penuh, waktu & zona waktu, kalender non-Gregorian.

## 5. Riwayat Perubahan Kontrak

| Versi SPEC | Perubahan | Alasan |
|---|---|---|
| 0.1.0 | Kontrak awal | Rilis pertama |
