# LombokLocale — go port

BCP 47 tags and negotiation, CLDR 47 plural rules, number/percent/currency and date formatting, an ICU MessageFormat subset, and JSON message catalogs, with the same results as the other four ports. No runtime dependencies.

```bash
go get github.com/codinglombok/lomboklocale/go
```

API mapping for this port: [docs/API_LombokLocale_v0.2.0.md](https://github.com/codinglombok/LombokLocale/blob/main/docs/API_LombokLocale_v0.2.0.md). Behaviour: [docs/SPEC_LombokLocale_v0.2.0.md](https://github.com/codinglombok/LombokLocale/blob/main/docs/SPEC_LombokLocale_v0.2.0.md).

Tests (run from this directory; they include the shared vectors in `../vectors`):

```bash
go test ./...
```

License: code Apache-2.0 OR MIT; embedded CLDR data Unicode License v3. Part of the [Lombok Ecosystem](https://github.com/codinglombok).
