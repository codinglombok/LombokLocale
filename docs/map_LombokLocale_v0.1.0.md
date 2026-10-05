# LombokLocale — MAP v0.1.0

Peta relasi `LombokLocale` di dalam Lombok Ecosystem — untuk peta lengkap 88 repo lihat MAP_UTAMA_v3.3.md.

## 1. Posisi Dependensi

```
LombokLocale (L0)
   (tidak bergantung pada apa pun)
        ▲
        │  ADR-009: pesan via katalog
   LombokValidator (L1) ... semua L1+
```

## 2. Contoh Dependen di Ekosistem (arah dependensi; bukan kepemilikan)

Daftar ini mencatat siapa yang *dapat* memakai `LombokLocale` (aplikasi → library). Library tetap mandiri dan tidak diklaim sebagai bagian dari salah satunya.

| Repo | Tingkat | Status pemakaian |
|---|---|---|
| LombokValidator | L1 | ✅ dipakai (dependensi wajib) |
| LombokTemplates, LombokCLI, LombokServer, LombokClarion, LombokPDF | L1–P/A | ⚪ direncanakan (ADR-009) |

## 3. `LombokLocale` Bergantung pada

Tidak ada.

## 4. Jalur Kontrak Normatif

`LombokLocale` → `SPEC_LombokLocale_v0.1.0.md` → `vectors/lomboklocale-vectors-v1.json` → dijalankan oleh **LombokTest** di setiap port bahasa (lihat [SPEC](SPEC_LombokLocale_v0.1.0.md)).

## 5. Peta Folder Lintas Cluster

| Cluster | Path di monorepo katalog |
|---|---|
| 00 · Fondasi & Primitif Inti (00.04) | `ecosystem/00_Fondasi-Core/04_LombokLocale/LombokLocale/` |
