<?php

declare(strict_types=1);

// API behaviour not covered by the shared vectors.

require_once __DIR__ . '/bootstrap.php';

use LombokLocale\LanguageTag;
use LombokLocale\Locale;
use LombokLocale\LocaleError;
use LombokLocale\Num;

$t = Locale::parseTag('sl-Latn-IT-rozaj-u-ca-gregory-x-priv');
check($t->language === 'sl' && $t->script === 'Latn' && $t->region === 'IT', 'tag parts');
check($t->base() === 'sl-Latn-IT-rozaj', 'base');
check($t->lookupChain() === ['sl-Latn-IT-rozaj', 'sl-Latn-IT', 'sl-Latn', 'sl'], 'lookup chain');
check((string) $t === 'sl-Latn-IT-rozaj-u-ca-gregory-x-priv', 'string form');
check(Locale::canonicalize('en-b-aa-a-bb') === 'en-a-bb-b-aa', 'extension order');
check((string) new LanguageTag('id') === 'id', 'constructor');

$e = new LocaleError('INVALID_SUBTAG', '$');
check($e->getMessage() === 'INVALID_SUBTAG: $' && (new LocaleError('EMPTY'))->getMessage() === 'EMPTY', 'error message');
check(Locale::cldrVersion() === '47.0.0' && Locale::MAX_DEPTH === 16 && Locale::MAX_EXPONENT === 1000, 'constants');

check(Locale::toDecimalString(1.5) === '1.5', 'float');
check(Locale::toDecimalString(NAN) === 'NaN' && Locale::toDecimalString(INF) === 'Infinity' && Locale::toDecimalString(-INF) === '-Infinity', 'specials');
check(Locale::toDecimalString(new Num('1.50')) === '1.50' && Locale::toDecimalString(7) === '7', 'num and int');
check(Locale::formatNumber('en', 0.1 + 0.2) === '0.3', 'shortest float');
check(Locale::formatNumber('en', 1e21) === '1,000,000,000,000,000,000,000', 'big float');
check(Locale::formatNumber('en', -0.0) === '-0', 'negative zero');
check(Locale::pluralCategory('en', 1) === 'one' && Locale::ordinalCategory('en', 2) === 'two', 'plural ints');
throwsCode(fn () => Locale::pluralCategory('en', NAN), 'BAD_NUMBER', 'nan plural');
throwsCode(fn () => Locale::formatNumber('en', 1, 'scientific'), 'BAD_OPTION', 'style');
throwsCode(fn () => Locale::formatNumber('en', 1, 'decimal', null, -1), 'BAD_OPTION', 'negative digits');

check(Locale::parseDate('2026-10-02') === [2026, 10, 2] && Locale::weekday(2026, 10, 2) === 5, 'dates');
check(Locale::formatDate('en', '2026-10-02') === 'Oct 2, 2026', 'default date style');

check(Locale::formatMessage('en', '{a} {b} {c} {d}', ['a' => 1234.5, 'b' => 7, 'c' => 'x', 'd' => new Num('1.50')]) === '1,234.5 7 x 1.5', 'args');
check(Locale::formatMessage('en', 'plain') === 'plain', 'plain');
check(Locale::formatMessage('en', '{n, select, 1{one} other{x}}', ['n' => 1]) === 'one', 'numeric select');
check(Locale::formatMessage('en', '{n, plural, offset: 1 other{#}}', ['n' => 3]) === '2', 'spaced offset');
check(Locale::formatMessage('en', '{n, plural, offset:2 other{#}}', ['n' => new Num('-1.5')]) === '-3.5', 'negative offset result');
check(Locale::formatMessage('en', '{n, plural, offset:1 other{#}}', ['n' => new Num('0.5')]) === '-0.5', 'offset crossing zero');
check(Locale::formatMessage('en', '{n, plural, offset:1 other{#}}', ['n' => new Num('99.9')]) === '98.9', 'offset with borrow');
throwsCode(fn () => Locale::formatMessage('en', '{n, plural, other{#}}', ['n' => true]), 'BAD_ARGUMENT', 'bool arg');
foreach (['{n, plural, other{#}', '{n, plural, other x}', '{n, plural,', '{n, plural, offset:x other{#}}', '{n, date, huge}', '{n', '{n,', '{n, number,'] as $p) {
    throwsCode(fn () => Locale::formatMessage('en', $p, ['n' => 1]), 'SYNTAX', $p);
}
check(Locale::formatMessage('en', "it's '{' unterminated") === "it's { unterminated", 'unterminated quote');
check(Locale::formatMessage('en', "'{a''b}'") === "{a'b}", 'quoted apostrophe');

foreach (['{"a": tru}', '{"a": -}', '{"a": 1.}', '{"a" "b"}', '{"a": "\\x"}', '{"a": "\\u12"}', '{"a": "\\udc00"}',
    '{"a": "\\ud800\\u0041"}', '{"a": [1 2]}', str_repeat('[', 100) . str_repeat(']', 100), '{"a": "x"', "{\"a\": \"\xff\"}", '{"a": "\\', '{"a', '{"a":', '['] as $bad) {
    throwsCode(fn () => Locale::parseCatalog($bad), 'BAD_JSON', "catalog $bad");
}
throwsCode(fn () => Locale::parseCatalog('{"a": {}, "b": []}'), 'NON_STRING_VALUE', 'nested');
check(Locale::parseCatalog('{"a": "\\b\\f\\r\\/\\"\\\\\\u00e9\\u20ac"}')['a'] === "\x08\x0C\r/\"\\é€", 'escapes');
$m = Locale::resolve('id', ['c' => '{n, plural, other{# hal}}'], 'C', 'c', ['n' => 2000]);
check($m->text === '2.000 hal' && !$m->fallback && $m->code === 'C' && $m->messageId === 'c', 'resolve');
check(Locale::resolve('id', [], 'X', 'y')->text === '!!y!!', 'resolve fallback');
finish('api');
