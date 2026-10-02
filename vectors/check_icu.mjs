// Cross-checks the number, currency, percent, date and plural vectors against
// ICU through Node.js Intl. Run with a Node.js whose ICU tracks the same CLDR
// release as data/cldr47.json (Node 22.x with ICU 77 ships CLDR 47).
//   node vectors/check_icu.mjs
import { readFileSync } from 'node:fs';

const doc = JSON.parse(readFileSync(new URL('./lomboklocale-vectors-v1.json', import.meta.url), 'utf8'));
if (process.versions.cldr !== doc.cldr.replace(/\.0\.0$/, '.0') && !process.versions.cldr.startsWith(doc.cldr.split('.')[0] + '.')) {
    console.error(`ICU CLDR ${process.versions.cldr} does not match vectors CLDR ${doc.cldr}; skipping`);
    process.exit(0);
}
// Differences that are understood and documented in SPEC section 9.
const KNOWN = {
    'plural|xx|1|': 'unknown locale: ICU falls back to the default locale, the SPEC to CLDR root (always "other")',
    'date|ru|2026-10-02|long': 'cldr-json 47.0.0 has U+202F before "г."; ICU 77.1 emits U+0020',
    'date|uk|2026-10-02|medium': 'cldr-json 47.0.0 has U+202F before "р."; ICU 77.1 emits U+0020',
    'number|en|1e1000|': 'ICU reads decimal strings only up to about 1e300 and returns infinity beyond',
};
const keyOf = (c) => `${c.kind}|${c.input.locale}|${c.input.value ?? c.input.date}|${c.kind === 'date' ? c.input.style : ''}`;
let checked = 0;
let known = 0;
const bad = [];
const ext = loc => `${loc}-u-nu-latn`;
for (const c of doc.cases) {
    if (!('ok' in c.expected)) continue;
    const i = c.input;
    let got;
    if (c.kind === 'number') {
        const opts = { style: i.style ?? 'decimal', roundingMode: 'halfExpand' };
        if (i.currency) opts.currency = i.currency;
        if (i.minFraction !== undefined) opts.minimumFractionDigits = i.minFraction;
        if (i.maxFraction !== undefined) opts.maximumFractionDigits = i.maxFraction;
        got = new Intl.NumberFormat(ext(i.locale), opts).format(i.value);
    } else if (c.kind === 'date') {
        const [y, m, d] = i.date.split('-').map(Number);
        const t = new Date(Date.UTC(2000, m - 1, d));
        t.setUTCFullYear(y);
        got = new Intl.DateTimeFormat(`${i.locale}-u-ca-gregory-nu-latn`, { dateStyle: i.style, timeZone: 'UTC' }).format(t);
    } else if (c.kind === 'plural') {
        const digits = i.value.replace(/^-/, '').replace('.', '');
        if (digits.length > 15) continue;
        const v = (i.value.split('.')[1] ?? '').length;
        got = new Intl.PluralRules(i.locale, { type: i.ordinal ? 'ordinal' : 'cardinal', minimumFractionDigits: v }).select(Number(i.value));
    } else continue;
    checked++;
    if (got !== c.expected.ok && KNOWN[keyOf(c)]) { known++; continue; }
    if (got !== c.expected.ok) bad.push(`${c.id}: ICU ${JSON.stringify(got)} vs vector ${JSON.stringify(c.expected.ok)}`);
}
console.log(`ICU ${process.versions.icu} (CLDR ${process.versions.cldr}): ${checked} cases checked, ${known} known differences, ${bad.length} unexpected`);
for (const b of bad) console.log('  ' + b);
process.exit(bad.length ? 1 : 0);
