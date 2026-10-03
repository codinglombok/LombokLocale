import { test } from 'node:test';
import assert from 'node:assert/strict';
import {
    baseTag, canonicalize, CLDR_VERSION, formatDate, formatMessage, formatNumber, formatTag, LocaleError, lookupChain, MAX_DEPTH,
    MAX_EXPONENT, ordinalCategory, parseCatalog, parseDate, parseTag, pluralCategory, resolve, toDecimalString, weekday,
} from '../src/index.js';

const code = (fn: () => unknown): string => {
    try {
        fn();
    } catch (e) {
        if (e instanceof LocaleError) return e.code;
        throw e;
    }
    return 'none';
};

test('tags expose parts', () => {
    const t = parseTag('sl-Latn-IT-rozaj-u-ca-gregory-x-priv');
    assert.equal(t.language, 'sl');
    assert.equal(t.script, 'Latn');
    assert.equal(t.region, 'IT');
    assert.deepEqual(t.variants, ['rozaj']);
    assert.deepEqual(t.extensions, [['u', 'ca', 'gregory']]);
    assert.deepEqual(t.privateUse, ['priv']);
    assert.equal(baseTag(t), 'sl-Latn-IT-rozaj');
    assert.deepEqual(lookupChain(t), ['sl-Latn-IT-rozaj', 'sl-Latn-IT', 'sl-Latn', 'sl']);
    assert.equal(formatTag(t), 'sl-Latn-IT-rozaj-u-ca-gregory-x-priv');
    assert.equal(canonicalize('en-b-aa-a-bb'), 'en-a-bb-b-aa');
    assert.equal(canonicalize('en-a-bb-a1-cc'), 'en-a-bb-a1-cc');
});

test('errors carry code and detail', () => {
    const e = new LocaleError('INVALID_SUBTAG', '$');
    assert.equal(e.message, 'INVALID_SUBTAG: $');
    assert.equal(new LocaleError('EMPTY').message, 'EMPTY');
    assert.equal(e.name, 'LocaleError');
    assert.equal(code(() => parseTag('en-$')), 'INVALID_SUBTAG');
});

test('numeric inputs', () => {
    assert.equal(toDecimalString(1.5), '1.5');
    assert.equal(toDecimalString(-0), '-0');
    assert.equal(toDecimalString(10n ** 25n), '10000000000000000000000000');
    assert.equal(toDecimalString(NaN), 'NaN');
    assert.equal(toDecimalString(Infinity), 'Infinity');
    assert.equal(toDecimalString(-Infinity), '-Infinity');
    assert.equal(toDecimalString(1e21), '1e+21');
    assert.equal(formatNumber('en', 0.1 + 0.2), '0.3');
    assert.equal(formatNumber('en', 1e21), '1,000,000,000,000,000,000,000');
    assert.equal(formatNumber('en', 12345678901234567890n), '12,345,678,901,234,567,890');
    assert.equal(formatNumber('en', -0), '-0');
    assert.equal(pluralCategory('en', 1), 'one');
    assert.equal(ordinalCategory('en', 2n), 'two');
    assert.equal(code(() => pluralCategory('en', NaN)), 'BAD_NUMBER');
    assert.equal(code(() => formatNumber('en', 1, { style: 'scientific' as never })), 'BAD_OPTION');
    assert.equal(code(() => formatNumber('en', 1, { minFraction: 1.5 })), 'BAD_OPTION');
    assert.equal(code(() => formatNumber('en', 1, { minFraction: -1 })), 'BAD_OPTION');
    assert.equal(formatNumber('en', '0.5', { style: 'percent' }), '50%');
});

test('dates', () => {
    assert.deepEqual(parseDate('2026-10-02'), { year: 2026, month: 10, day: 2 });
    assert.equal(weekday(2026, 10, 2), 5);
    assert.equal(formatDate('en', '2026-10-02'), 'Oct 2, 2026');
    assert.equal(code(() => formatDate('en', '2026-10-02', 'huge' as never)), 'BAD_OPTION');
    assert.equal(code(() => parseDate('2026-02-30')), 'BAD_DATE');
});

test('message arguments from JavaScript values', () => {
    assert.equal(formatMessage('en', '{a} {b} {c} {d}', { a: 1234.5, b: 7n, c: 'x', d: { num: '1.50' } }), '1,234.5 7 x 1.5');
    assert.equal(formatMessage('en', 'plain'), 'plain');
    assert.equal(formatMessage('en', '{n, plural, one{one} other{#}}', { n: 1 }), 'one');
    assert.equal(formatMessage('en', '{n, select, 1{one} other{x}}', { n: 1 }), 'one');
    assert.equal(code(() => formatMessage('en', '{n, number}', { n: 'x' })), 'BAD_ARGUMENT');
    assert.equal(code(() => formatMessage('en', '{n, plural, other{#}}', { n: Infinity })), 'BAD_ARGUMENT');
    assert.equal(code(() => formatMessage('en', '{n, plural, other{#}', { n: 1 })), 'SYNTAX');
    assert.equal(code(() => formatMessage('en', '{n, plural, other x}', { n: 1 })), 'SYNTAX');
    assert.equal(code(() => formatMessage('en', '{n, plural,', { n: 1 })), 'SYNTAX');
    assert.equal(code(() => formatMessage('en', '{n, plural, offset:x other{#}}', { n: 1 })), 'SYNTAX');
    assert.equal(formatMessage('en', '{n, plural, offset: 1 other{#}}', { n: 3 }), '2');
    assert.equal(code(() => formatMessage('en', '{n, date, huge}', { n: 'x' })), 'SYNTAX');
    assert.equal(code(() => formatMessage('en', '{n', {})), 'SYNTAX');
    assert.equal(formatMessage('en', "it's '{' unterminated", {}), "it's { unterminated");
    assert.equal(formatMessage('en', "'{a''b}'", {}), "{a'b}");
});

test('catalogs', () => {
    const c = parseCatalog('{"b": "2", "a": "1"}');
    assert.deepEqual([...c.entries()], [['b', '2'], ['a', '1']]);
    for (const bad of ['{"a": tru}', '{"a": -}', '{"a": 1.}', '{"a" "b"}', '{"a": "\\x"}', '{"a": "\\u12"}', '{"a": "\\udc00"}',
        '{"a": "\\ud800\\u0041"}', '{"a": [1 2]}', '{"a": "\ud800"}', '{"a": "\udc00x"}', '[' .repeat(100) + ']'.repeat(100), '{"a": "x"']) {
        assert.equal(code(() => parseCatalog(bad)), 'BAD_JSON', bad);
    }
    assert.equal(code(() => parseCatalog('{"a": {}, "b": []}')), 'NON_STRING_VALUE');
    assert.equal(parseCatalog('{"a": "\\b\\f\\r\\/\\"\\\\"}').get('a'), '\b\f\r/"\\');
    assert.equal(parseCatalog('{"a": "😀"}').get('a'), '\u{1F600}');
    const m = resolve('id', new Map([['c', '{n, plural, other{# hal}}']]), 'C', 'c', { n: 2000 });
    assert.deepEqual(m, { code: 'C', messageId: 'c', text: '2.000 hal', fallback: false });
    assert.equal(resolve('id', new Map(), 'X', 'y').text, '!!y!!');
});

test('constants', () => {
    assert.equal(CLDR_VERSION, '47.0.0');
    assert.equal(MAX_DEPTH, 16);
    assert.equal(MAX_EXPONENT, 1000);
});
