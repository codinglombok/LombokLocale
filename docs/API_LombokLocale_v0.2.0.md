# LombokLocale — API v0.2.0

Perilaku normatif ada di SPEC. Dokumen ini memetakan konsep SPEC ke nama di setiap port.

## 1. Ringkasan lintas port

| Konsep (SPEC) | Rust | TypeScript | Python | Go | PHP (`Locale::`) |
|---|---|---|---|---|---|
| parse tag (§2.1) | `parse_tag(t) -> Result<LanguageTag>` | `parseTag(t)` | `parse_tag(t)` | `ParseTag(t) (LanguageTag, error)` | `parseTag($t)` |
| kanonik | `canonicalize(t)` | `canonicalize(t)` | `canonicalize(t)` | `Canonicalize(t)` | `canonicalize($t)` |
| negosiasi (§2.2) | `negotiate(&req, &avail, def)` | `negotiate(req, avail, def)` | `negotiate(req, avail, default)` | `Negotiate(req, avail, def)` | `negotiate($req, $avail, $def)` |
| locale data (§2.3) | `data_locale(t)` | `dataLocale(t)` | `data_locale(t)` | `DataLocale(t)` | `dataLocale($t)` |
| plural (§4) | `plural_category(l, "1.5")` / `ordinal_category` | `pluralCategory(l, v)` / `ordinalCategory` | `plural_category(l, v)` / `ordinal_category` | `PluralCategory(l, "1.5")` / `OrdinalCategory` | `pluralCategory($l, $v)` / `ordinalCategory` |
| angka (§5) | `format_number(l, "1.5", &NumberOptions)` | `formatNumber(l, v, { style, currency, minFraction, maxFraction })` | `format_number(l, v, style=, currency=, min_fraction=, max_fraction=)` | `FormatNumber(l, "1.5", NumberOptions{Style, Currency, MinFraction, MaxFraction})` | `formatNumber($l, $v, $style, $currency, $min, $max)` |
| tanggal (§6) | `format_date(l, "2026-10-02", DateStyle::Full)` | `formatDate(l, d, 'full')` | `format_date(l, d, "full")` | `FormatDate(l, d, "full")` | `formatDate($l, $d, 'full')` |
| pesan (§7) | `format_message(l, p, &[("n", Arg::from(3u64))])` | `formatMessage(l, p, { n: 3 })` | `format_message(l, p, {"n": 3})` | `FormatMessage(l, p, map[string]Arg{"n": Int(3)})` | `formatMessage($l, $p, ['n' => 3])` |
| katalog (§8) | `parse_catalog(json) -> Catalog` | `parseCatalog(json): Map` | `parse_catalog(json) -> dict` | `ParseCatalog(json) (map[string]string, error)` | `parseCatalog($json): array` |
| resolve | `resolve(l, &cat, code, id, args) -> LocalizedMessage` | `resolve(...)` | `resolve(...)` | `Resolve(...)` | `resolve(...)` |
| error | `Error { code: ErrorCode, detail }`, `code.as_str()` | `LocaleError` (`code`, `detail`) | `LocaleError` (`code`, `detail`; turunan `ValueError`) | `*Error{Code, Detail}` (`errors.As`) | `LocaleError` (`errorCode`, `detail`) |
| versi CLDR | `CLDR_VERSION` | `CLDR_VERSION` | `CLDR_VERSION` | `CLDRVersion()` | `cldrVersion()` |

## 2. Nilai angka per port

| Port | Teks desimal eksak | Angka bahasa |
|---|---|---|
| Rust | `&str` di semua fungsi; `Arg::num("1.50")` | `Arg::from(i64 / u64 / f64)` (teks terpendek) |
| TypeScript | `string`; di pesan `{ num: "1.50" }` | `number`, `bigint` (`-0` dipertahankan) |
| Python | `str`; di pesan `Num("1.50")` atau `Decimal` | `int`, `float`, `Decimal` (`bool` ditolak) |
| Go | `string`; `Num("1.50")` | `Int(int64)`, `Float(float64)`, `FloatString(f)` |
| PHP | `string`; `new Num('1.50')` | `int`, `float` (`var_export` terpendek) |

Di pesan, teks biasa selalu dianggap teks: `{"n": "5"}` adalah teks dan menghasilkan `BAD_ARGUMENT` pada plural.

## 3. Tipe tambahan

| Port | Tipe |
|---|---|
| Rust | `LanguageTag { language, script, region, variants, extensions, private_use }` + `base()`, `lookup_chain()`, `Display`; `Date { year, month, day }` + `parse`, `weekday`; `NumberOptions::{decimal, percent, currency, fraction}`; `Style`; `PluralCategory::as_str` |
| TypeScript | `LanguageTag` (objek), `baseTag`, `formatTag`, `lookupChain`, `parseDate`, `weekday`, `toDecimalString` |
| Python | `LanguageTag` (dataclass) + `base()`, `lookup_chain()`, `str()`; `parse_date`, `weekday`, `to_decimal_string`, `LocalizedMessage` (`message_id`) |
| Go | `LanguageTag` + `Base()`, `LookupChain()`, `String()`; `ParseDate`, `Weekday`, `Digits(n)` |
| PHP | `LanguageTag` + `base()`, `lookupChain()`, `__toString()`; `parseDate`, `weekday`, `toDecimalString`; `LocalizedMessage` |

## 4. Kompatibilitas dengan 0.1.0

0.1.0 tidak terbit. Nama yang dipertahankan di Rust: `negotiate`, `plural_category`, `ordinal_category`, `format_message`, `parse_catalog`, `resolve`, `format_date`, `Catalog`, `PluralCategory`. Yang berubah: angka diberikan sebagai teks (`plural_category(l, "1")`), `format_integer`/`format_float`/`format_currency` digabung menjadi `format_number`, `format_date` menerima teks tanggal dan `DateStyle` baru (`Iso` dihapus), `ArgValue` menjadi `Arg`, `parse_bcp47` menjadi `parse_tag`, semua error menjadi `Error` dengan `ErrorCode`.

*Lisensi dokumen: Apache-2.0 OR MIT · © codinglombok*
