# LombokLocale — STRUCTURE REPO v0.1.0

## 1. Struktur Folder

```
LombokLocale/
├── rust/            # src/{lib,bcp47,plural,message,number,date,catalog}.rs · tests/{vectors,catalogs,robustness}.rs
├── typescript/      # src/{index,compat}.ts · test/vectors.test.ts (lulus vektor bersama)
├── python/ go/ php/ # stub README
├── locales/{en,id,es,fr,de,pt,ru,ja,ko,zh-Hans}/lomboklocale.json  + STATUS.md
├── vectors/         # lomboklocale-vectors-v1.json (1.831 kasus) + gen_inputs.py
├── docs/  scripts/  .github/workflows/{ci,release}.yml
├── README.md  LICENSE-APACHE  LICENSE-MIT  .gitignore
```

## 2. Konvensi Penamaan (mengikuti MASTERPLAN_UTAMA §4)

| Platform | Nama di repo ini |
|---|---|
| GitHub repo | `codinglombok/LombokLocale` |
| Package registry (npm/PyPI/crates.io) | `lomboklocale` |
| Packagist | `codinglombok/lomboklocale` |
| Go module | `github.com/codinglombok/lomboklocale/go` (tag `go/vX.Y.Z`) |
| Namespace/Class | `LombokLocale` (Rust: `lomboklocale::`) |

## 3. File Wajib di Root

| File | Ada? |
|---|---|
| `README.md` | ✅ |
| `LICENSE` (dual-license: `LICENSE-APACHE` + `LICENSE-MIT`) | ✅ |
| `.gitignore` | ✅ |
| `CHANGELOG.md` → lihat `changelog_LombokLocale_v0.1.0.md` | ✅ |
| `.github/workflows/ci.yml` | ✅ |
| `.github/workflows/release.yml` | ✅ |
| 12 dokumen standar (`docs/`) | ✅ — lihat daftar di [Masterplan](masterplan_LombokLocale_v0.1.0.md) |

## 4. Struktur per Bahasa Port

- `rust/` — implementasi referensi; `LOMBOK_REGEN=1 cargo test --test vectors` menulis ulang ekspektasi vektor dari referensi.
- `typescript/` — port native (ESM, tanpa dependensi runtime); `npm test` menjalankan seluruh vektor.
- `python/ go/ php/` — stub.

## 5. Catatan

Go module untuk port Go: `github.com/codinglombok/lomboklocale/go` dengan tag `go/vX.Y.Z` (MASTERPLAN_UTAMA §4).
