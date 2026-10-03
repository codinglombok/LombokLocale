# LombokLocale — Structure Repo v0.2.0

```
LombokLocale/
├── README.md · CHANGELOG.md · LICENSE-APACHE · LICENSE-MIT · LICENSE-UNICODE (data CLDR)
├── .github/workflows/  ci.yml (5 port + standards + pemeriksaan ICU) · release.yml (terbit pada tag v*)
├── docs/               10 dokumen publik; masterplan_ dan architecture_ adalah dokumen internal (ADR-024), tidak di-commit
├── data/               cldr47.json (subset CLDR 47.0.0)
├── locales/            <tag>/lomboklocale.json (katalog pesan library ini) · STATUS.md
├── scripts/            extract_cldr.py · gen_ports.py · lombok-doctor.sh
├── vectors/            lomboklocale-vectors-v1.json · SHA256SUMS · build_vectors.py (model referensi) · check_icu.mjs
├── rust/               Cargo.toml · src/{lib,tag,decimal,plural,number,date,message,catalog,error,tables,data}.rs · tests/{vectors,api,catalogs}.rs
├── typescript/         package.json · src/{index,data}.ts · tests/{api,vectors}.test.ts · scripts/coverage.mjs
├── python/             pyproject.toml · lomboklocale/{__init__,_data}.py · tests/{test_api,test_vectors}.py
├── go/                 go.mod · locale.go · data.go · {api,vectors}_test.go
└── php/                composer.json · src/{Locale,LanguageTag,LocaleError,LocalizedMessage,Num,MessageParser,JsonReader,Data}.php · tests/{run,api,vectors,bootstrap}.php
```

Aturan: `data.rs`, `data.ts`, `_data.py`, `data.go`, `Data.php` dibangkitkan dan tidak diedit tangan (`python3 scripts/gen_ports.py --check` di CI); perubahan perilaku mengubah SPEC dan vector lebih dulu, lalu kelima port; hasil build tidak di-commit.

*Lisensi dokumen: Apache-2.0 OR MIT · © codinglombok*
