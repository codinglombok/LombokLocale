# LombokLocale message catalogs

| Language | Code | Status |
|---|---|---|
| English | `en` | source of truth |
| Bahasa Indonesia | `id` | written and reviewed by the ecosystem owner |
| Español, Français, Deutsch, Português, Русский, 日本語, 한국어, 简体中文 | `es` `fr` `de` `pt` `ru` `ja` `ko` `zh-Hans` | machine-translated drafts; must be reviewed by native speakers before they are considered final |

Rules: the keys (`messageId`) and the placeholders (`{requested}`, `{fallback}`, `{messageId}`) must stay identical to `en`, and every file must pass `parse_catalog`. Both properties are enforced by `rust/tests/catalogs.rs` in CI.

Still missing: `ar`, `hi`, `bn`, `tr`, `vi`, `th`, `it`, `pl`, `nl`, `uk`, and Nusantara languages (`jv`, `su`, `ms`). Contributions from native speakers are welcome via pull request.
