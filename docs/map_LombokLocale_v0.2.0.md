# LombokLocale — Map v0.2.0

## 1. Posisi di ekosistem

```
Cluster 01 Fondasi & Runtime · tingkat L0 (tanpa dependensi Lombok wajib)

L0  LombokLocale
     dependensi wajib    : (tidak ada)
     dependensi opsional : (tidak ada)
     data tertanam       : CLDR 47.0.0 (Unicode License v3)
     dependensi dev      : serde_json (runner vector Rust), typescript + @types/node (TS), pytest + coverage (Python, CI), Node.js (check_icu.mjs)
```

## 2. Contoh pemakai di ekosistem

Library ini mandiri dan dapat dipakai siapa pun. Library dan aplikasi Lombok yang dapat memakainya (arah dependensi selalu pemakai ke library):

| Pemakai | Pemakaian |
|---|---|
| LombokValidator (L1) | pesan error terlokalisasi lewat katalog dan `resolve` |
| LombokCLIParse | rencana: judul help dan pesan error terlokalisasi (opsional) |
| LombokCSV, LombokTableSheet, LombokCharts | angka, mata uang dan tanggal sesuai locale di tabel dan sumbu grafik |
| LombokPDF, LombokDocx | angka dan tanggal di dokumen terbangkit |

## 3. Peta fitur x port

| Fitur | Rust | TypeScript | Python | Go | PHP |
|---|---|---|---|---|---|
| tag BCP 47, negosiasi, data locale (SPEC §2) | YA | YA | YA | YA | YA |
| aturan plural kardinal/ordinal (§4) | YA | YA | YA | YA | YA |
| angka, persen, mata uang (§5) | YA | YA | YA | YA | YA |
| tanggal (§6) | YA | YA | YA | YA | YA |
| MessageFormat (§7) | YA | YA | YA | YA | YA |
| katalog dan resolve (§8) | YA | YA | YA | YA | YA |
| `no_std` | YA | - | - | - | - |
| vector | 342/342 | 342/342 | 342/342 | 342/342 | 342/342 |

*Lisensi dokumen: Apache-2.0 OR MIT · © codinglombok*
