# LombokLocale — Guide How to Use v0.2.0

## 1. Pemasangan

| Bahasa | Perintah | Syarat |
|---|---|---|
| Rust | `cargo add lomboklocale` | Rust 1.70+ (`default-features = false` untuk `no_std`) |
| TypeScript/JS | `npm install lomboklocale` | Node.js 20+, Deno, Bun, browser modern |
| Python | `pip install lomboklocale` | Python 3.9+ |
| Go | `go get github.com/codinglombok/lomboklocale/go` | Go 1.22+ |
| PHP | `composer require codinglombok/lomboklocale` | PHP 8.1+ |

## 2. Memilih locale

```ts
const tag = negotiate(navigatorLanguages, ['en', 'id', 'ja'], 'en');   // RFC 4647 lookup
canonicalize('EN_us');                                                 // "en-US"
dataLocale('zh-TW');                                                   // "zh-Hant" (data yang dipakai untuk format)
```

Simpan hasil `negotiate` per pengguna; semua fungsi format menerima tag apa pun dan tidak gagal karena locale.

## 3. Angka dan mata uang

Berikan angka sebagai teks desimal bila nilainya penting secara hukum (uang, pajak): `"1234.50"` dibulatkan secara eksak, tidak lewat float.

| Panggilan (TS) | Hasil |
|---|---|
| `formatNumber('id', '1234567.891')` | `1.234.567,891` |
| `formatNumber('es', '1234')` | `1234` (minimum grouping 2) |
| `formatNumber('id', '15000', { style: 'currency', currency: 'IDR' })` | `Rp 15.000,00` (U+00A0) |
| `formatNumber('ja', '15000', { style: 'currency', currency: 'JPY' })` | `￥15,000` (JPY tanpa desimal) |
| `formatNumber('en', '0.256', { style: 'percent' })` | `26%` |
| `formatNumber('en', '2.5', { maxFraction: 0 })` | `3` (setengah menjauhi nol) |

## 4. Tanggal

`formatDate('id', '2026-10-02', 'full')` → `Jumat, 02 Oktober 2026`; gaya `full`, `long`, `medium` (bawaan), `short`. Masukan selalu `YYYY-MM-DD`; tidak ada jam atau zona waktu, jadi tanggal tidak bergeser karena zona waktu server.

## 5. Plural dan pesan

```ts
pluralCategory('ru', 22);   // "few"
formatMessage('ru', '{n, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}', { n: 22 });  // "22 файла"
formatMessage('en', '{n, plural, offset:1 =0{nobody} =1{{who}} one{{who} and # other} other{{who} and # others}}',
              { n: 5, who: 'Ana' });                      // "Ana and 4 others"
formatMessage('id', 'Jatuh tempo {d, date}', { d: '2026-10-02' });   // "Jatuh tempo 2 Okt 2026"
```

Selalu sertakan `other`. Untuk tanda kurung kurawal literal tulis `'{'`; untuk apostrof tulis `''`.

## 6. Katalog

```ts
const id = parseCatalog(readFileSync('locales/id/app.json', 'utf8'));
const msg = resolve('id', id, 'TOO_SHORT', 'validator.too_short', { min: 8 });
if (msg.fallback) log.warn('missing translation', msg.messageId);
show(msg.text);
```

Program membandingkan `msg.code`, bukan teks. Terjemahan yang hilang tampil sebagai `!!id!!` sehingga mudah ditemukan saat uji.

## 7. Skenario netral

1. Toko daring dengan frontend TypeScript dan backend PHP: total keranjang dan pesan "3 barang" sama di kedua sisi.
2. Aplikasi lapangan offline (Rust `no_std` di perangkat) yang mencetak struk dengan format rupiah.
3. Laporan PDF dari Python dan dasbor web yang menampilkan angka dan tanggal yang sama.
4. Layanan Go yang memilih bahasa email dari header `Accept-Language` dengan `negotiate`.

## 8. Pemecahan masalah

| Gejala | Penyebab | Solusi |
|---|---|---|
| spasi di angka tidak cocok dengan string uji | CLDR memakai U+00A0 / U+202F | bandingkan dengan karakter yang sama, bukan spasi biasa |
| `0.1 + 0.2` tampil `0.3` | default maks 3 digit pecahan | atur `maxFraction` |
| `xx-YY` diformat seperti `en` | tidak ada data locale itu | lihat daftar 28 locale di SPEC §1 |
| `BAD_ARGUMENT` pada plural | argumen berupa teks | kirim angka (Python: `Num("1.50")`, TS: `{ num: "1.50" }`) |

*Lisensi dokumen: Apache-2.0 OR MIT · © codinglombok*
