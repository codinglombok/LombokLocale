# LombokLocale — GUIDE: HOW TO USE v0.1.0

## Instalasi

```toml
# Rust — saat ini via git (belum di crates.io)
[dependencies]
lomboklocale = { git = "https://github.com/codinglombok/LombokLocale", package = "lomboklocale" }
```

```bash
# TypeScript — saat ini dari sumber (belum di npm)
cd typescript && npm install && npm run build
```

Setelah rilis pertama: `cargo add lomboklocale` · `npm i lomboklocale`. Python/Go/PHP: belum ada port.

## Konsep Dasar

- **`code`** = kontrak stabil untuk mesin; **`messageId`** = kunci terjemahan; **teks** = hasil `resolve()` untuk manusia.
- **Katalog** = objek JSON datar per bahasa per repo.
- **Negosiasi** = pilih locale tersedia terbaik dari daftar preferensi pengguna.

## Contoh Penggunaan

```rust
use lomboklocale::*;
let picked = negotiate(&["id-ID", "en"], &["en", "id"], "en"); // "id"
let args = [("count", ArgValue::UInt(5))];
format_message("en", "{count, plural, one{# file} other{# files}}", &args).unwrap(); // "5 files"
format_currency(15000.0, "IDR", "id"); // "Rp15.000,00"
let cat = parse_catalog(r#"{"greet": "Halo, {name}!"}"#).unwrap();
resolve("id", &cat, "GREET", "greet", &[("name", ArgValue::Str("Bali".into()))]).text; // "Halo, Bali!"
```

## Pola Pemakaian Umum (Recipes)

- **Memilih bahasa dari header `Accept-Language`**: urutkan tag berdasarkan q, lalu `negotiate(&daftar, &tersedia, "en")`.
- **Library baru**: simpan hanya `messageId` + teks `en`; muat katalog bahasa terpilih lalu `resolve()`.

## Kesalahan Umum (Common Pitfalls)

- Jangan memakai teks terjemahan sebagai kunci — pakai `messageId`.
- `{count, plural, ...}` butuh argumen numerik non-negatif; string akan menghasilkan `MalformedSelector`.
- Di `id`, `one{...}` tidak pernah terpilih (Indonesia tanpa plural) — isi `other{...}`.

## Lihat Juga

- [API Reference lengkap](API_LombokLocale_v0.1.0.md)
- [Full Summary](full_summary_project_LombokLocale_v0.1.0.md)
