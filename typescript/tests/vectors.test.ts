import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import {
    canonicalize, dataLocale, formatDate, formatMessage, formatNumber, LocaleError, negotiate, ordinalCategory,
    parseCatalog, pluralCategory, resolve, type Args, type DateStyle, type NumberOptions,
} from '../src/index.js';

const path = fileURLToPath(new URL('../../../vectors/lomboklocale-vectors-v1.json', import.meta.url));
const doc = JSON.parse(readFileSync(path, 'utf8')) as { cases: { id: string; kind: string; input: Record<string, unknown>; expected: unknown }[] };

function args(a: Record<string, { s?: string; n?: string }>): Args {
    const out: Args = {};
    for (const [k, v] of Object.entries(a)) out[k] = v.s !== undefined ? v.s : { num: v.n! };
    return out;
}

function run(kind: string, i: Record<string, any>): unknown {
    switch (kind) {
        case 'parse': return canonicalize(i.tag);
        case 'negotiate': return negotiate(i.requested, i.available, i.default);
        case 'dataLocale': return dataLocale(i.tag);
        case 'plural': return i.ordinal ? ordinalCategory(i.locale, i.value) : pluralCategory(i.locale, i.value);
        case 'number': return formatNumber(i.locale, i.value, {
            style: i.style, currency: i.currency, minFraction: i.minFraction, maxFraction: i.maxFraction,
        } as NumberOptions);
        case 'date': return formatDate(i.locale, i.date, i.style as DateStyle);
        case 'message': return formatMessage(i.locale, i.pattern, args(i.args));
        case 'catalog': return Object.fromEntries(parseCatalog(i.json));
        case 'resolve': return resolve(i.locale, parseCatalog(i.catalog), i.code, i.messageId, args(i.args));
        default: throw new Error(kind);
    }
}

test('vector file has at least 100 cases', () => {
    assert.ok(doc.cases.length >= 100);
});

for (const c of doc.cases) {
    test(`vector ${c.id}`, () => {
        let got: unknown;
        try {
            got = { ok: run(c.kind, c.input) };
        } catch (e) {
            if (!(e instanceof LocaleError)) throw e;
            got = { error: e.code };
        }
        assert.deepStrictEqual(got, c.expected);
    });
}
