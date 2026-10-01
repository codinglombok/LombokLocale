// Cross-language conformance: every case in ../vectors/lomboklocale-vectors-v1.json
// must produce exactly the expected `out` / `err` (ADR-002 / ADR-015).
import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import * as L from "../src/index.js";

const here = dirname(fileURLToPath(import.meta.url));
const doc = JSON.parse(readFileSync(join(here, "../../../vectors/lomboklocale-vectors-v1.json"), "utf8")) as {
  cases: Array<{ fn: string; in: any; out?: unknown; err?: string }>;
};

const num = (v: unknown): number => (typeof v === "string" ? ({ NaN: NaN, Infinity: Infinity, "-Infinity": -Infinity } as Record<string, number>)[v] ?? Number(v) : (v as number));
const res = <T>(r: L.Result<T, string>, f: (v: T) => unknown) => (r.ok ? { out: f(r.value) } : { err: r.error });

function evalCase(fn: string, i: any): { out?: unknown; err?: string } {
  switch (fn) {
    case "parse_bcp47": return res(L.parseBcp47(i), L.localeToString);
    case "negotiate": return { out: L.negotiate(i.requested, i.available, i.default) };
    case "plural_category": return { out: L.pluralCategory(i.locale, i.n) };
    case "ordinal_category": return { out: L.ordinalCategory(i.locale, i.n) };
    case "format_integer": return { out: L.formatInteger(typeof i.value === "string" ? BigInt(i.value) : i.value, i.locale) };
    case "format_float": return { out: L.formatFloat(num(i.value), i.decimals, i.locale) };
    case "format_currency": return { out: L.formatCurrency(num(i.value), i.code, i.locale) };
    case "format_date": return { out: L.formatDate({ year: i.year, month: i.month, day: i.day }, i.locale, i.style) };
    case "parse_iso_date": return { out: L.parseIsoDate(i) };
    case "format_message": return res(L.formatMessage(i.locale, i.pattern, i.args), (v) => v);
    case "parse_catalog":
      return res(L.parseCatalog(i), (c) => Object.fromEntries([...c.iter()]));
    case "resolve": {
      const cat = new L.Catalog();
      for (const [k, v] of Object.entries(i.catalog)) cat.insert(k, v as string);
      const m = L.resolve(i.locale, cat, i.code, i.messageId, i.args);
      return { out: { code: m.code, messageId: m.messageId, text: m.text } };
    }
    default: throw new Error("unknown fn " + fn);
  }
}

test(`conformance: ${doc.cases.length} vectors`, () => {
  const failures: string[] = [];
  doc.cases.forEach((c, n) => {
    const got = evalCase(c.fn, c.in);
    try {
      assert.deepStrictEqual(got.out, c.out);
      assert.deepStrictEqual(got.err, c.err);
    } catch {
      failures.push(`case ${n} ${c.fn} in=${JSON.stringify(c.in)} expected=${JSON.stringify(c.out ?? c.err)} got=${JSON.stringify(got.out ?? got.err)}`);
    }
  });
  assert.equal(failures.length, 0, `${failures.length} failures:\n` + failures.slice(0, 8).join("\n"));
});
