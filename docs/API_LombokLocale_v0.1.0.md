# LombokLocale — API REFERENCE v0.1.0

Seluruh API publik, per bahasa, dengan kode error dan status stabilitas (ARCHITECTURE_UTAMA §5, baris "Dokumen" — normatif sejak MASTERPLAN_UTAMA v3.3 §1.1/§5).

Legenda stabilitas: 🟢 **Stable** (breaking change butuh major bump + RFC) · 🟡 **Beta** (boleh berubah, diumumkan di changelog) · 🔴 **Experimental** (bisa hilang tanpa notice).

## Rust (referensi)

| Simbol | Tanda tangan | Stabilitas |
|---|---|---|
| `parse_bcp47` | `(&str) -> Result<LocaleTag, LocaleError>` | 🟢 |
| `negotiate` | `(&[&str] requested, &[&str] available, &str default) -> String` | 🟢 |
| `plural_category` / `ordinal_category` | `(&str locale, u64 n) -> PluralCategory` | 🟢 |
| `format_message` | `(&str locale, &str pattern, &[(&str, ArgValue)]) -> Result<String, MessageError>` | 🟡 |
| `format_integer` | `(i64, &str) -> String` | 🟢 |
| `format_float` | `(f64, usize decimals (≤15), &str) -> String` | 🟡 |
| `format_currency` | `(f64, &str code, &str locale) -> String` | 🟡 |
| `format_date` / `parse_iso_date` | `(SimpleDate, &str, DateStyle) -> String` / `(&str) -> Option<SimpleDate>` | 🟡 |
| `parse_catalog` | `(&str) -> Result<Catalog, CatalogError>`; `Catalog::{get, iter, len, insert}` | 🟢 |
| `resolve` | `(&str locale, &Catalog, &str code, &str message_id, &[(&str, ArgValue)]) -> LocalizedMessage` | 🟢 |

### Kode Error

| Tipe | Varian |
|---|---|
| `LocaleError` | `Empty`, `InvalidSubtag(String)` |
| `MessageError` | `UnbalancedBraces`, `UnknownVariable(String)`, `MalformedSelector(String)`, `TooDeep` |
| `CatalogError` | `UnexpectedEnd`, `UnexpectedChar(char, usize)`, `ExpectedObject` |

## Port Lain

**TypeScript** (`typescript/src/index.ts`): padanan camelCase — `parseBcp47`, `negotiate`, `pluralCategory`, `ordinalCategory`, `formatMessage`, `formatInteger` (menerima `bigint`), `formatFloat`, `formatCurrency`, `formatDate`, `parseIsoDate`, `parseCatalog`, `Catalog`, `resolve`. Hasil kegagalan berbentuk `{ok:false, error}`. Lulus seluruh vektor.

Python/Go/PHP: belum ada.

## Kompatibilitas Lintas Bahasa

Setiap fungsi di atas **harus** menghasilkan output byte-identik lintas port untuk input yang sama, dibuktikan oleh `vectors/lomboklocale-vectors-v1.json` (lihat [SPEC](SPEC_LombokLocale_v0.1.0.md)). Perbedaan hasil antar-port adalah bug, bukan variasi yang sah (ADR-015).
