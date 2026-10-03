# LombokLocale — SPEC v0.2.0

This document is the normative cross-language contract. Every language port MUST produce byte-identical output for all specified inputs. Deviations from this specification are bugs.

| Atribut | Nilai |
|---|---|
| Versi SPEC | 0.2.0 (berlaku untuk paket `lomboklocale` 0.2.x di crates.io, npm, PyPI, Packagist, dan modul Go) |
| Acuan | BCP 47 (RFC 5646) dan RFC 4647 (lookup); Unicode CLDR 47.0.0 (`cldr-json` tag `47.0.0`, LDML / UTS #35 bagian Numbers, Dates, Plural Rules); ICU MessageFormat (sintaks, mode apostrof DOUBLE_OPTIONAL); RFC 8259 (JSON); ISO 4217 (kode mata uang); tinjauan 2026-10-02 |
| Data | `data/cldr47.json`, diekstrak oleh `scripts/extract_cldr.py`; setiap port menyematkan tabel yang dibangkitkan `scripts/gen_ports.py` (CI menolak tabel basi) |
| Vector | `vectors/lomboklocale-vectors-v1.json` — 342 kasus (plural 92, number 83, message 51, date 34, parse 30, catalog 18, negotiate 16, dataLocale 13, resolve 5) — SHA-256 `0808e4753412577cef7c6d350e1193660906ca69d4744c0fefe5c2f75d8da898` |
| Pemeriksa referensi | model Python independen `vectors/build_vectors.py` (kasus tulis tangan divalidasi terhadapnya) dan pemeriksaan silang ke ICU 77.1 / CLDR 47.0 lewat `vectors/check_icu.mjs` (193 kasus angka, mata uang, persen, tanggal, plural; 4 perbedaan yang diketahui, §9) |
| Port | Rust, TypeScript, Python, Go, PHP — kelimanya menjalankan seluruh vector |
| Tanggal tinjauan | 2026-10-02 |

Kata MUST, MUST NOT, SHOULD, MAY mengikuti RFC 2119.

## 0. Konvensi

1. Semua teks adalah Unicode; posisi dihitung dalam code point.
2. "Locale" adalah tag BCP 47 dalam bentuk teks; fungsi pemformat tidak pernah gagal karena locale (§2.3).
3. Angka diberikan sebagai **teks desimal** (§3). Port boleh menerima tipe angka bahasanya, yang lebih dulu diubah menjadi teks desimal terpendek yang terbaca kembali sebagai nilai yang sama (`f64`/`float`/`number`), `NaN`, `Infinity`, atau `-Infinity`.
4. Semua angka keluaran memakai digit Latin (`latn`) dan kalender Gregorian; ekstensi `-u-nu-*` dan `-u-ca-*` diabaikan.

## 1. Data CLDR

Data yang dipakai adalah CLDR 47.0.0. Untuk 28 locale data — `en`, `en-GB`, `en-IN`, `id`, `ms`, `jv`, `su`, `de`, `fr`, `es`, `pt`, `pt-PT`, `it`, `nl`, `ru`, `uk`, `pl`, `tr`, `ar`, `hi`, `zh`, `zh-Hant`, `ja`, `ko`, `vi`, `th`, `sv`, `cy` — diambil: simbol angka `latn` (desimal, grup, minus, persen, tak hingga, NaN), `minimumGroupingDigits`, pola standar desimal, persen dan mata uang, simbol 14 mata uang (USD, EUR, GBP, JPY, CNY, IDR, MYR, SGD, INR, KRW, SAR, AUD, THB, VND; simbol tak tersedia = kode ISO), pola tanggal full/long/medium/short, nama bulan dan hari (konteks *format*, singkat dan lengkap), serta singkatan era Masehi. Aturan plural kardinal (219 locale) dan ordinal (104 locale) diambil seluruhnya. Digit pecahan mata uang diambil dari `supplemental/currencyData` (kode tak dikenal: 2).

## 2. Tag bahasa

### 2.1 Parsing

Spasi di awal/akhir diabaikan; `-` dan `_` adalah pemisah. Teks kosong → `EMPTY`. Setiap subtag MUST berupa 1–8 karakter ASCII alfanumerik, jika tidak → `INVALID_SUBTAG` (detail: subtag itu). Urutan:

1. bahasa: 2–3 atau 5–8 huruf (huruf kecil);
2. opsional skrip: 4 huruf (huruf pertama besar, sisanya kecil);
3. opsional wilayah: 2 huruf (besar) atau 3 digit;
4. varian: 5–8 alfanumerik atau 4 karakter berawalan digit (kecil); varian ganda → `DUPLICATE_VARIANT`;
5. ekstensi: singleton selain `x`, diikuti minimal satu subtag 2–8 karakter (kecil); singleton ganda → `DUPLICATE_EXTENSION`; singleton tanpa subtag → `INVALID_SUBTAG`;
6. opsional `x` diikuti minimal satu subtag privat (kecil); `x` tanpa subtag → `INVALID_SUBTAG`;
7. subtag tersisa → `INVALID_SUBTAG`.

Bentuk kanonik: bagian-bagian di atas digabung dengan `-`, ekstensi diurutkan menurut teksnya. Extlang, tag grandfathered, dan tag yang hanya berisi bagian privat tidak didukung (→ `INVALID_SUBTAG`).

### 2.2 Negosiasi (RFC 4647 lookup)

`negotiate(requested, available, default)`: untuk setiap tag yang diminta, sesuai urutan (tag yang gagal di-parse dilewati), coba bentuk dasarnya (bahasa, skrip, wilayah, varian, tanpa ekstensi) lalu setiap prefiks yang lebih pendek. Pencocokan dengan `available` tidak membedakan huruf besar/kecil; hasilnya adalah entri `available` seperti ditulis. Tanpa kecocokan → `default`.

### 2.3 Locale data

`data_locale(tag)`: jika tag tidak dapat di-parse → `en`. Untuk bahasa `zh` tanpa skrip dengan wilayah TW, HK atau MO, skrip dianggap `Hant`. Lalu rantai lookup (§2.2) dicocokkan dengan 28 locale data; tanpa kecocokan → `en`. Aturan plural memakai rantai yang sama terhadap tabel plural; tanpa kecocokan → aturan `root` (selalu `other`).

## 3. Teks desimal

Bentuk: `[+-]digit+[.digit+][(e|E)[+-]digit{1,4}]`, atau `NaN`, `Infinity`, `+Infinity`, `-Infinity`. Eksponen dengan besaran lebih dari 1000 → `BAD_NUMBER`; bentuk lain → `BAD_NUMBER` (termasuk `1.`, `.5`, spasi). Nilai dibaca secara eksak menjadi tanda, digit bulat (tanpa nol di depan, minimal `0`) dan digit pecahan (nol di belakang dipertahankan). Tanda negatif dipertahankan untuk nol (`-0`).

## 4. Aturan plural

`plural_category(locale, value)` dan `ordinal_category(locale, value)` mengembalikan `zero`, `one`, `two`, `few`, `many` atau `other`. `NaN` dan tak hingga → `BAD_NUMBER`.

Operan (UTS #35): `n` nilai mutlak; `i` digit bulat; `v` jumlah digit pecahan yang terlihat; `w` sama tanpa nol di belakang; `f` digit pecahan sebagai bilangan bulat; `t` sama tanpa nol di belakang; `c` dan `e` selalu 0. Relasi `x [% m] = daftar` benar jika `x` (setelah modulus) adalah bilangan bulat dan berada dalam salah satu rentang; `!=` adalah negasinya. Untuk `n` dengan pecahan bukan nol, `=` selalu salah. Kategori pertama (urutan zero, one, two, few, many) yang aturannya cocok dipilih; jika tidak ada → `other`. Nilai di atas 10^18 tanpa modulus dianggap tidak cocok dengan rentang apa pun.

## 5. Format angka

`format_number(locale, value, style, currency, min_fraction, max_fraction)`; `style` = `decimal` (bawaan), `percent`, atau `currency`.

1. Digit pecahan bawaan: decimal 0–3, percent 0–0, currency = digit mata uang (min = maks). Jika hanya `min` diberikan: `max = max(min, bawaan maks)`; jika hanya `max`: `min = min(bawaan min, max)`. Rentang harus `0 ≤ min ≤ max ≤ 20`, jika tidak → `BAD_OPTION` (`fractionDigits`). Gaya lain → `BAD_OPTION` (`style`). Mata uang wajib tiga huruf ASCII (diubah ke huruf besar), jika tidak → `BAD_OPTION` (`currency`).
2. Percent: nilai dikalikan 100 secara eksak.
3. Pembulatan ke `max` digit pecahan, setengah menjauhi nol (half away from zero); lalu nol di belakang dibuang sampai tersisa `min` digit, atau ditambah sampai `min`.
4. Pengelompokan: dari bagian angka pola positif, `primary` = panjang grup terakhir sebelum titik, `secondary` = panjang grup sebelumnya (atau `primary`). Pengelompokan diterapkan hanya bila jumlah digit bulat ≥ `primary` + `minimumGroupingDigits`.
5. Pola dibelah pada `;` (subpola negatif opsional). Afiks dibentuk dari teks sebelum/sesudah bagian angka (`#0,.`); di dalam afiks: `¤` → simbol mata uang, `%` → simbol persen, `-` → simbol minus, `'teks'` literal, `''` → apostrof.
6. Nilai negatif (termasuk `-0` dan hasil pembulatan ke nol): subpola negatif bila ada; jika tidak, `-` + afiks awal positif.
7. Currency spacing (CLDR `currencySpacing`): bila `¤` adalah karakter terakhir afiks awal, karakter terakhir simbol bukan kategori Unicode S/Z, dan teks angka diawali digit, sisipkan U+00A0 setelah simbol; simetris untuk `¤` di awal afiks akhir.
8. `NaN` dan tak hingga memakai simbol locale sebagai teks angka (tanpa pengelompokan dan pecahan).

## 6. Format tanggal

`format_date(locale, date, style)`; `date` berbentuk `YYYY-MM-DD` tepat sepuluh karakter, tanggal Gregorian proleptik 0001-01-01 sampai 9999-12-31, jika tidak → `BAD_DATE`; `style` ∈ `full`, `long`, `medium` (bawaan), `short`, jika tidak → `BAD_OPTION`. Pola CLDR dibaca kiri ke kanan: deretan huruf ASCII sama adalah satu medan; `'...'` literal, `''` apostrof. Medan: `G` singkatan era; `y` tahun (lebar n diisi nol; `yy` dua digit terakhir); `M`/`L` bulan (1–2 angka, 3 singkat, 4 lengkap); `d` hari (diisi nol sampai lebar); `E` nama hari (1–3 singkat, 4 lengkap). Karakter lain disalin apa adanya. Hari dalam minggu dihitung dari 0001-01-01 (Senin).

## 7. MessageFormat (subset ICU)

### 7.1 Teks

Teks disalin apa adanya kecuali `{`, `}`, `'`, dan `#` di dalam sub-pesan plural. Apostrof: `''` → `'`; `'` yang diikuti `{`, `}`, `|` atau (dalam plural) `#` memulai kutipan sampai `'` berikutnya (`''` di dalamnya → `'`; kutipan tanpa penutup berlanjut sampai akhir); `'` lain adalah literal. `}` di tingkat teratas → `SYNTAX`.

### 7.2 Argumen

Spasi (spasi, tab, CR, LF) boleh di sekitar nama, tipe, gaya, dan kunci. Nama: `[A-Za-z0-9_]+`, jika tidak → `SYNTAX`.

| Bentuk | Hasil |
|---|---|
| `{n}` | teks apa adanya; angka diformat seperti `number` |
| `{n, number}` / `{n, number, integer}` / `{n, number, percent}` | §5 decimal / decimal dengan `max = 0` / percent |
| `{d, date}` / `{d, date, full\|long\|medium\|short}` | §6, gaya bawaan `medium`; argumen harus teks tanggal |
| `{n, plural, [offset:k] kunci{pesan} ...}` | kunci `zero one two few many other` atau `=desimal` |
| `{n, selectordinal, ...}` | seperti plural dengan aturan ordinal, tanpa offset |
| `{x, select, kunci{pesan} ...}` | kunci `[A-Za-z0-9_]+` |

Tipe lain atau gaya lain → `SYNTAX`. Kunci ganda, kunci tidak valid, `{`/`}` yang hilang → `SYNTAX`. Pilihan tanpa `other` → `MISSING_OTHER`. Kedalaman sub-pesan lebih dari 16 → `TOO_DEEP`.

### 7.3 Nilai argumen

Argumen adalah teks atau angka (teks desimal §3). Argumen yang tidak ada → `MISSING_ARGUMENT`. Angka yang tidak terbaca pada `{n}`/`number`, teks pada `number`/`plural`/`selectordinal`, angka pada `date`, tanggal tidak valid, dan `NaN`/tak hingga pada plural → `BAD_ARGUMENT`.

### 7.4 Pemilihan

`select`: kunci yang sama dengan teks argumen (angka: teks desimalnya), atau `other`. `plural`/`selectordinal`: kunci `=x` yang nilainya sama secara numerik dengan argumen (sebelum offset; `0` dan `-0` sama) dipilih lebih dulu; jika tidak ada, kategori dihitung (§4) dari `argumen − offset` yang dibulatkan ke 3 digit pecahan (setengah menjauhi nol) tanpa nol di belakang; kunci kategori itu, atau `other`. Di dalam sub-pesan (termasuk select di dalamnya), `#` diganti `argumen − offset` yang diformat seperti `number`. Untuk offset bukan nol, argumen dengan lebih dari 36 digit → `BAD_ARGUMENT`.

## 8. Katalog dan error

`parse_catalog(json)`: dokumen MUST berupa JSON RFC 8259 yang valid dan UTF-8 (teks/escape surrogate tunggal, karakter kontrol mentah, angka tidak valid, sisa teks, kedalaman lebih dari 64 → `BAD_JSON`), dengan objek di tingkat teratas (`NOT_OBJECT`), seluruh nilai berupa string (`NON_STRING_VALUE`, detail: kunci), dan kunci unik (`DUPLICATE_KEY`). `BAD_JSON` diperiksa lebih dulu atas seluruh dokumen; kemudian pasangan diperiksa berurutan.

`resolve(locale, catalog, code, message_id, args)` → `{code, messageId, text, fallback}`: jika `message_id` ada dan polanya terformat, `text` adalah hasilnya dan `fallback` salah; selain itu `text = "!!message_id!!"` dan `fallback` benar.

Kode error: `EMPTY`, `INVALID_SUBTAG`, `DUPLICATE_VARIANT`, `DUPLICATE_EXTENSION`, `BAD_NUMBER`, `BAD_OPTION`, `BAD_DATE`, `SYNTAX`, `MISSING_OTHER`, `MISSING_ARGUMENT`, `BAD_ARGUMENT`, `TOO_DEEP`, `BAD_JSON`, `NOT_OBJECT`, `NON_STRING_VALUE`, `DUPLICATE_KEY`. Vector membandingkan kode; teks pesan error bergantung port.

## 9. Perbedaan dengan ICU yang diketahui

| Kasus | SPEC | ICU 77.1 | Alasan |
|---|---|---|---|
| plural locale tak dikenal (`xx`) | aturan `root` → `other` | locale bawaan sistem | hasil tidak boleh bergantung pada mesin |
| tanggal `ru` long / `uk` medium | U+202F sebelum `г.`/`р.` (data `cldr-json` 47.0.0) | U+0020 | SPEC mengikuti data CLDR yang diterbitkan |
| `1e1000` | diformat eksak | ∞ (batas pembacaan ICU) | SPEC memakai aritmetika desimal eksak |

## 10. Keamanan (normatif)

1. Tidak ada eksekusi kode, akses berkas, jaringan, atau variabel lingkungan; data CLDR tertanam di dalam paket.
2. Masukan dibatasi: subtag 8 karakter, eksponen 1000, kedalaman pesan 16, kedalaman JSON 64, operan plural tanpa modulus di atas 10^18 diperlakukan sebagai "besar". Waktu linear terhadap panjang masukan kecuali panjang angka hasil (eksponen ≤ 1000).
3. Teks argumen disisipkan apa adanya; aplikasi yang menampilkan hasil di HTML MUST melakukan escape sendiri.
4. Rust: `#![forbid(unsafe_code)]`, `no_std` + `alloc`. Tidak ada dependensi runtime di semua port.

## 11. Perubahan dari 0.1.0

0.1.0 tidak pernah terbit; hanya Rust dan TypeScript yang berjalan, nilai harapan vector dibangkitkan dari Rust sendiri, dan banyak aturan ditulis tangan secara keliru. Perubahan utama:

- aturan plural kini dievaluasi dari data CLDR 47 untuk semua locale (sebelumnya daftar tulis tangan; contoh salah: `pl` 21 → one, `hr` 5 → many, tanpa kategori `many` untuk jutaan di `fr`/`es`/`pt`);
- angka, mata uang dan tanggal memakai pola, simbol, digit mata uang dan nama bulan/hari CLDR (sebelumnya: hanya `en`/`id` untuk tanggal, simbol mata uang di depan untuk `de`/`es`, JPY dengan dua desimal);
- persen dan pengelompokan India/minimum grouping ditambahkan;
- MessageFormat: apostrof, offset, `=n`, `number`, `date`, `#` terformat sesuai locale, error berkode;
- tag BCP 47: ekstensi, privat, varian, dan error yang lebih ketat; negosiasi mengikuti RFC 4647 lookup murni (tidak lagi mencoba `zh-TW` sebelum `zh` untuk `zh-Hant-TW`);
- katalog JSON ketat (duplikat, surrogate, kedalaman);
- port Python, Go, PHP diimplementasikan; lisensi data CLDR (Unicode License v3) dicantumkan.

*Lisensi dokumen: Apache-2.0 OR MIT · © codinglombok*
