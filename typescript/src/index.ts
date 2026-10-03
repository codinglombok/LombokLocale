/**
 * LombokLocale: BCP 47 tags and negotiation, CLDR 47 plural rules,
 * number/percent/currency and date formatting, an ICU MessageFormat subset,
 * and JSON message catalogs. Same results as the Rust, Python, Go and PHP
 * ports (docs/SPEC_LombokLocale_v0.2.0.md). No dependencies.
 */
import { DATA, type LocaleData, type Rules } from './data.js';

/** CLDR release of the embedded data. */
export const CLDR_VERSION = DATA.cldr;
/** Maximum nesting of message sub-messages. */
export const MAX_DEPTH = 16;
/** Largest exponent magnitude accepted in decimal strings. */
export const MAX_EXPONENT = 1000;

/** Error codes shared by every port (SPEC section 8). */
export type ErrorCode =
    | 'EMPTY' | 'INVALID_SUBTAG' | 'DUPLICATE_VARIANT' | 'DUPLICATE_EXTENSION'
    | 'BAD_NUMBER' | 'BAD_OPTION' | 'BAD_DATE'
    | 'SYNTAX' | 'MISSING_OTHER' | 'MISSING_ARGUMENT' | 'BAD_ARGUMENT' | 'TOO_DEEP'
    | 'BAD_JSON' | 'NOT_OBJECT' | 'NON_STRING_VALUE' | 'DUPLICATE_KEY';

/** Thrown by every function that can fail; `detail` names the offending input. */
export class LocaleError extends Error {
    readonly code: ErrorCode;
    readonly detail: string;

    constructor(code: ErrorCode, detail = '') {
        super(detail === '' ? code : `${code}: ${detail}`);
        this.name = 'LocaleError';
        this.code = code;
        this.detail = detail;
    }
}

// ---------------------------------------------------------------- tags

/** A well-formed language tag in canonical case. */
export interface LanguageTag {
    language: string;
    script?: string;
    region?: string;
    variants: string[];
    /** Each extension: singleton followed by its subtags, sorted by singleton. */
    extensions: string[][];
    privateUse: string[];
}

const ALNUM = /^[A-Za-z0-9]{1,8}$/;
const ALPHA = /^[A-Za-z]+$/;
const DIGITS = /^[0-9]+$/;

/** Parses a BCP 47 tag; `_` is accepted as a separator, outer spaces are ignored. */
export function parseTag(tag: string): LanguageTag {
    const t = tag.trim();
    if (t === '') throw new LocaleError('EMPTY');
    const subs = t.split(/[-_]/);
    for (const s of subs) if (!ALNUM.test(s)) throw new LocaleError('INVALID_SUBTAG', s);
    const lang = subs[0]!;
    if (!(ALPHA.test(lang) && lang.length !== 1 && lang.length !== 4)) throw new LocaleError('INVALID_SUBTAG', lang);
    const out: LanguageTag = { language: lang.toLowerCase(), variants: [], extensions: [], privateUse: [] };
    let i = 1;
    const at = (k: number): string => subs[k] ?? '';
    if (at(i).length === 4 && ALPHA.test(at(i))) {
        out.script = at(i)[0]!.toUpperCase() + at(i).slice(1).toLowerCase();
        i++;
    }
    if ((at(i).length === 2 && ALPHA.test(at(i))) || (at(i).length === 3 && DIGITS.test(at(i)))) {
        out.region = at(i).toUpperCase();
        i++;
    }
    while (i < subs.length && (at(i).length >= 5 || (at(i).length === 4 && /^[0-9]/.test(at(i))))) {
        const v = at(i).toLowerCase();
        if (out.variants.includes(v)) throw new LocaleError('DUPLICATE_VARIANT', v);
        out.variants.push(v);
        i++;
    }
    while (i < subs.length && at(i).length === 1 && at(i).toLowerCase() !== 'x') {
        const single = at(i).toLowerCase();
        if (out.extensions.some(e => e[0] === single)) throw new LocaleError('DUPLICATE_EXTENSION', single);
        i++;
        const ext = [single];
        while (i < subs.length && at(i).length >= 2) ext.push(at(i++).toLowerCase());
        if (ext.length === 1) throw new LocaleError('INVALID_SUBTAG', single);
        out.extensions.push(ext);
    }
    if (i < subs.length && at(i).toLowerCase() === 'x') {
        i++;
        if (i === subs.length) throw new LocaleError('INVALID_SUBTAG', 'x');
        while (i < subs.length) out.privateUse.push(at(i++).toLowerCase());
    }
    if (i < subs.length) throw new LocaleError('INVALID_SUBTAG', at(i));
    out.extensions.sort((a, b) => (a.join('-') < b.join('-') ? -1 : a.join('-') > b.join('-') ? 1 : 0));
    return out;
}

/** language, script, region and variants joined with `-`. */
export function baseTag(t: LanguageTag): string {
    return [t.language, t.script, t.region, ...t.variants].filter((p): p is string => !!p).join('-');
}

/** Canonical string form including extensions and private use. */
export function formatTag(t: LanguageTag): string {
    const parts = [baseTag(t), ...t.extensions.flat()];
    if (t.privateUse.length) parts.push('x', ...t.privateUse);
    return parts.join('-');
}

/** Canonical form of `tag`, e.g. `"EN_us"` -> `"en-US"`. */
export function canonicalize(tag: string): string {
    return formatTag(parseTag(tag));
}

/** RFC 4647 lookup chain: the base tag, then shorter prefixes. */
export function lookupChain(t: LanguageTag): string[] {
    const parts = baseTag(t).split('-');
    const out: string[] = [];
    for (let k = parts.length; k > 0; k--) out.push(parts.slice(0, k).join('-'));
    return out;
}

function tryParse(tag: string): LanguageTag | undefined {
    try {
        return parseTag(tag);
    } catch {
        return undefined;
    }
}

/**
 * RFC 4647 lookup: for each requested tag in order, try it without extensions
 * and then each shorter prefix against `available` (case-insensitive); return
 * the match as written in `available`, or `defaultLocale`.
 */
export function negotiate(requested: readonly string[], available: readonly string[], defaultLocale: string): string {
    const avail: [string, string][] = [];
    for (const a of available) {
        const t = tryParse(a);
        if (t) avail.push([baseTag(t).toLowerCase(), a]);
    }
    for (const r of requested) {
        const t = tryParse(r);
        if (!t) continue;
        for (const cand of lookupChain(t)) {
            const hit = avail.find(([c]) => c === cand.toLowerCase());
            if (hit) return hit[1];
        }
    }
    return defaultLocale;
}

/** The CLDR data locale used to format for `tag` (SPEC section 2.3). */
export function dataLocale(tag: string): string {
    const t = tryParse(tag);
    if (!t) return 'en';
    if (t.language === 'zh' && t.script === undefined && ['TW', 'HK', 'MO'].includes(t.region ?? '')) t.script = 'Hant';
    for (const cand of lookupChain(t)) if (Object.hasOwn(DATA.locales, cand)) return cand;
    return 'en';
}

function localeData(tag: string): LocaleData {
    return DATA.locales[dataLocale(tag)]!;
}

// ---------------------------------------------------------------- decimals

/** A number argument: number, bigint, or an exact decimal string such as `"1.50"`. */
export type Numeric = number | bigint | string;

type Dec = { kind: 'finite'; neg: boolean; int: string; frac: string } | { kind: 'nan' } | { kind: 'inf'; neg: boolean };

const DEC = /^([+-]?)([0-9]+)(?:\.([0-9]+))?(?:[eE]([+-]?[0-9]{1,4}))?$/;

/** Shortest decimal text of a value (numbers round-trip; `-0` keeps its sign). */
export function toDecimalString(v: Numeric): string {
    if (typeof v === 'string') return v;
    if (typeof v === 'bigint') return v.toString();
    if (Number.isNaN(v)) return 'NaN';
    if (v === Infinity) return 'Infinity';
    if (v === -Infinity) return '-Infinity';
    if (Object.is(v, -0)) return '-0';
    return String(v);
}

function parseDec(v: Numeric): Dec {
    const s = toDecimalString(v);
    if (s === 'NaN') return { kind: 'nan' };
    if (s === 'Infinity' || s === '+Infinity') return { kind: 'inf', neg: false };
    if (s === '-Infinity') return { kind: 'inf', neg: true };
    const m = DEC.exec(s);
    if (!m) throw new LocaleError('BAD_NUMBER', s);
    const exp = Number(m[4] ?? '0');
    if (Math.abs(exp) > MAX_EXPONENT) throw new LocaleError('BAD_NUMBER', s);
    let digits = m[2]! + (m[3] ?? '');
    let point = m[2]!.length + exp;
    if (point <= 0) {
        digits = '0'.repeat(1 - point) + digits;
        point = 1;
    }
    if (point > digits.length) digits += '0'.repeat(point - digits.length);
    const int = digits.slice(0, point).replace(/^0+/, '') || '0';
    return { kind: 'finite', neg: m[1] === '-', int, frac: digits.slice(point) };
}

function roundHalfExpand(int: string, frac: string, max: number): [string, string] {
    if (frac.length <= max) return [int, frac];
    const keep = int + frac.slice(0, max);
    let digits = keep;
    if (frac.charCodeAt(max) >= 0x35) {
        const arr = keep.split('');
        let k = arr.length - 1;
        for (;;) {
            if (k < 0) {
                arr.unshift('1');
                break;
            }
            if (arr[k] === '9') {
                arr[k] = '0';
                k--;
            } else {
                arr[k] = String.fromCharCode(arr[k]!.charCodeAt(0) + 1);
                break;
            }
        }
        digits = arr.join('');
    }
    const split = digits.length - max;
    return [digits.slice(0, split).replace(/^0+/, '') || '0', digits.slice(split)];
}

function digitsMod(digits: string, m: number): number {
    let r = 0;
    for (let i = 0; i < digits.length; i++) r = (r * 10 + digits.charCodeAt(i) - 48) % m;
    return r;
}

// ---------------------------------------------------------------- plural

/** CLDR plural category. */
export type PluralCategory = 'zero' | 'one' | 'two' | 'few' | 'many' | 'other';

function operandValue(int: string, frac: string, op: string, mod: number): [boolean, number | undefined] {
    const trimmed = frac.replace(/0+$/, '');
    const fromDigits = (s: string): number | undefined => (mod ? digitsMod(s, mod) : s.length > 15 ? undefined : Number(s));
    const small = (n: number): number => (mod ? n % mod : n);
    switch (op) {
        case 'n':
            return [/^0*$/.test(frac), fromDigits(int)];
        case 'i':
            return [true, fromDigits(int)];
        case 'v':
            return [true, small(frac.length)];
        case 'w':
            return [true, small(trimmed.length)];
        case 'f':
            return [true, fromDigits(frac || '0')];
        case 't':
            return [true, fromDigits(trimmed || '0')];
        default:
            return [true, 0];
    }
}

function selectCategory(table: Record<string, Rules>, locale: string, value: Numeric): PluralCategory {
    const d = parseDec(value);
    if (d.kind !== 'finite') throw new LocaleError('BAD_NUMBER', toDecimalString(value));
    const t = tryParse(locale);
    let rules: Rules | undefined;
    if (t) for (const cand of lookupChain(t)) if (Object.hasOwn(table, cand)) {
        rules = table[cand];
        break;
    }
    rules ??= table['root'] ?? [];
    for (const [cat, groups] of rules) {
        const hit = groups.some(rels => rels.every(([op, mod, neg, ranges]) => {
            const [isInt, val] = operandValue(d.int, d.frac, op, mod);
            const inside = isInt && val !== undefined && ranges.some(([lo, hi]) => lo <= val && val <= hi);
            return inside !== (neg === 1);
        }));
        if (hit) return cat as PluralCategory;
    }
    return 'other';
}

/** Cardinal plural category; decimal strings keep trailing zeros as visible digits. */
export function pluralCategory(locale: string, value: Numeric): PluralCategory {
    return selectCategory(DATA.plurals, locale, value);
}

/** Ordinal plural category (1st, 2nd, 3rd, ...). */
export function ordinalCategory(locale: string, value: Numeric): PluralCategory {
    return selectCategory(DATA.ordinals, locale, value);
}

// ---------------------------------------------------------------- numbers

/** Options of {@link formatNumber}. */
export interface NumberOptions {
    style?: 'decimal' | 'percent' | 'currency';
    /** ISO 4217 code, required for the currency style. */
    currency?: string;
    /** 0..20 */
    minFraction?: number;
    /** 0..20 */
    maxFraction?: number;
}

function splitPattern(p: string): [string, string, string] {
    let quoted = false;
    let start = -1;
    let end = 0;
    for (let i = 0; i < p.length; i++) {
        const c = p[i]!;
        if (c === "'") quoted = !quoted;
        else if (!quoted && '#0,.'.includes(c)) {
            if (start < 0) start = i;
            end = i + 1;
        }
    }
    if (start < 0) start = end = p.length;
    return [p.slice(0, start), p.slice(start, end), p.slice(end)];
}

function grouping(numpart: string): [number, number] {
    const groups = numpart.split('.')[0]!.split(',');
    const primary = groups.length > 1 ? groups[groups.length - 1]!.length : 0;
    const secondary = groups.length > 2 ? groups[groups.length - 2]!.length : primary;
    return [primary, secondary];
}

function groupInt(ip: string, primary: number, secondary: number, minGrouping: number, sep: string): string {
    if (primary === 0 || ip.length < primary + minGrouping) return ip;
    let head = ip.slice(0, ip.length - primary);
    const parts = [ip.slice(ip.length - primary)];
    while (head.length > secondary) {
        parts.unshift(head.slice(head.length - secondary));
        head = head.slice(0, head.length - secondary);
    }
    if (head) parts.unshift(head);
    return parts.join(sep);
}

function expandAffix(raw: string, d: LocaleData, symbol: string): string {
    let out = '';
    let quoted = false;
    for (let i = 0; i < raw.length; i++) {
        const c = raw[i]!;
        if (c === "'") {
            if (raw[i + 1] === "'") {
                out += "'";
                i++;
                continue;
            }
            quoted = !quoted;
        } else if (quoted) out += c;
        else if (c === '¤') out += symbol;
        else if (c === '%') out += d.percent;
        else if (c === '-') out += d.minus;
        else out += c;
    }
    return out;
}

const isDigit = (c: string | undefined): boolean => c !== undefined && c >= '0' && c <= '9';

/** Formats a number for `locale` (half away from zero; CLDR grouping and symbols). */
export function formatNumber(locale: string, value: Numeric, opts: NumberOptions = {}): string {
    const d = localeData(locale);
    const style = opts.style ?? 'decimal';
    let code = '';
    let dmin: number;
    let dmax: number;
    let pattern: string;
    if (style === 'currency') {
        if (typeof opts.currency !== 'string' || !/^[A-Za-z]{3}$/.test(opts.currency)) throw new LocaleError('BAD_OPTION', 'currency');
        code = opts.currency.toUpperCase();
        dmin = dmax = DATA.currencyDigits[code] ?? 2;
        pattern = d.currencyPattern;
    } else if (style === 'percent') {
        [dmin, dmax, pattern] = [0, 0, d.percentPattern];
    } else if (style === 'decimal') {
        [dmin, dmax, pattern] = [0, 3, d.decimalPattern];
    } else throw new LocaleError('BAD_OPTION', 'style');
    let minf = opts.minFraction;
    let maxf = opts.maxFraction;
    if (minf !== undefined && maxf === undefined) maxf = Math.max(minf, dmax);
    if (maxf !== undefined && minf === undefined) minf = Math.min(dmin, maxf);
    minf ??= dmin;
    maxf ??= dmax;
    if (!Number.isInteger(minf) || !Number.isInteger(maxf) || minf < 0 || maxf > 20 || minf > maxf) {
        throw new LocaleError('BAD_OPTION', 'fractionDigits');
    }
    const semi = pattern.indexOf(';');
    const pos = semi < 0 ? pattern : pattern.slice(0, semi);
    const negp = semi < 0 ? undefined : pattern.slice(semi + 1);
    const [prefix, numpart, suffix] = splitPattern(pos);
    const v = parseDec(value);
    let neg: boolean;
    let body: string;
    if (v.kind === 'nan') [neg, body] = [false, d.nan];
    else if (v.kind === 'inf') [neg, body] = [v.neg, d.infinity];
    else {
        neg = v.neg;
        let { int, frac } = v;
        if (style === 'percent') {
            frac += '00';
            int = (int + frac.slice(0, 2)).replace(/^0+/, '') || '0';
            frac = frac.slice(2);
        }
        let [ip, fp] = roundHalfExpand(int, frac, maxf);
        fp = fp.padEnd(minf, '0');
        while (fp.length > minf && fp.endsWith('0')) fp = fp.slice(0, -1);
        const [primary, secondary] = grouping(numpart);
        body = groupInt(ip, primary, secondary, d.minGrouping, d.group);
        if (fp) body += d.decimal + fp;
    }
    let np: string;
    let ns: string;
    if (neg && negp !== undefined) {
        const [a, , b] = splitPattern(negp);
        [np, ns] = [a, b];
    } else if (neg) [np, ns] = ['-' + prefix, suffix];
    else [np, ns] = [prefix, suffix];
    if (style === 'currency') {
        const entry = d.currencySymbols[code] ?? [code, 0, 0];
        let pre = expandAffix(np, d, entry[0]);
        let suf = expandAffix(ns, d, entry[0]);
        if (np.endsWith('¤') && !entry[2] && isDigit(body[0])) pre += ' ';
        if (ns.startsWith('¤') && !entry[1] && isDigit(body[body.length - 1])) suf = ' ' + suf;
        return pre + body + suf;
    }
    return expandAffix(np, d, '') + body + expandAffix(ns, d, '');
}

// ---------------------------------------------------------------- dates

/** CLDR date style. */
export type DateStyle = 'full' | 'long' | 'medium' | 'short';

const DATE_RE = /^([0-9]{4})-([0-9]{2})-([0-9]{2})$/;

const isLeap = (y: number): boolean => y % 4 === 0 && (y % 100 !== 0 || y % 400 === 0);
const daysInMonth = (y: number, m: number): number => (m === 2 ? (isLeap(y) ? 29 : 28) : [4, 6, 9, 11].includes(m) ? 30 : 31);

/** Parses `YYYY-MM-DD` (0001-01-01 to 9999-12-31). */
export function parseDate(s: string): { year: number; month: number; day: number } {
    const m = DATE_RE.exec(s);
    if (!m) throw new LocaleError('BAD_DATE', s);
    const [year, month, day] = [Number(m[1]), Number(m[2]), Number(m[3])];
    if (year < 1 || month < 1 || month > 12 || day < 1 || day > daysInMonth(year, month)) throw new LocaleError('BAD_DATE', s);
    return { year, month, day };
}

/** Day of the week of a valid date, 0 = Sunday. */
export function weekday(year: number, month: number, day: number): number {
    const y = year - 1;
    let days = y * 365 + Math.floor(y / 4) - Math.floor(y / 100) + Math.floor(y / 400);
    for (let m = 1; m < month; m++) days += daysInMonth(year, m);
    days += day - 1;
    return (days + 1) % 7;
}

/** Formats `YYYY-MM-DD` with the CLDR pattern of `style` (Gregorian calendar). */
export function formatDate(locale: string, date: string, style: DateStyle = 'medium'): string {
    if (!['full', 'long', 'medium', 'short'].includes(style)) throw new LocaleError('BAD_OPTION', 'style');
    const { year, month, day } = parseDate(date);
    const d = localeData(locale);
    const pat = d.dateFormats[style];
    let out = '';
    let i = 0;
    while (i < pat.length) {
        const c = pat[i]!;
        if (c === "'") {
            let j = i + 1;
            while (j < pat.length) {
                if (pat[j] === "'") {
                    if (pat[j + 1] === "'") {
                        out += "'";
                        j += 2;
                        continue;
                    }
                    break;
                }
                out += pat[j];
                j++;
            }
            if (j === i + 1) out += "'";
            i = j + 1;
            continue;
        }
        if (/[A-Za-z]/.test(c)) {
            let j = i;
            while (pat[j] === c) j++;
            const n = j - i;
            if (c === 'G') out += d.era;
            else if (c === 'y') out += n === 2 ? String(year % 100).padStart(2, '0') : String(year).padStart(n, '0');
            else if (c === 'M' || c === 'L') out += n <= 2 ? String(month).padStart(n, '0') : d.months[n === 3 ? 0 : 1][month - 1];
            else if (c === 'd') out += String(day).padStart(n, '0');
            else if (c === 'E') out += d.days[n === 4 ? 1 : 0][weekday(year, month, day)];
            else throw new Error(`pattern letter ${c} is not used by the CLDR 47 subset`);
            i = j;
            continue;
        }
        out += c;
        i++;
    }
    return out;
}

// ---------------------------------------------------------------- messages

/** A message argument: text, or a number (number, bigint, or `{ num: "1.50" }`). */
export type Arg = string | number | bigint | { num: string };
/** Named message arguments. */
export type Args = Record<string, Arg>;

type Part =
    | { t: 'text'; v: string }
    | { t: 'pound' }
    | { t: 'simple'; name: string }
    | { t: 'number'; name: string; style?: string }
    | { t: 'date'; name: string; style: DateStyle }
    | { t: 'choice'; kind: 'plural' | 'selectordinal' | 'select'; name: string; offset: number; cases: [string, Part[]][] };

const NAME = /^[A-Za-z0-9_]+$/;
const KEYWORDS = ['zero', 'one', 'two', 'few', 'many', 'other'];

class Parser {
    i = 0;
    constructor(private readonly s: string[]) {}

    private syntax(): LocaleError {
        return new LocaleError('SYNTAX', String(this.i));
    }

    message(depth: number, inPlural: boolean, top: boolean): Part[] {
        if (depth > MAX_DEPTH) throw new LocaleError('TOO_DEEP');
        const parts: Part[] = [];
        let buf = '';
        const s = this.s;
        while (this.i < s.length) {
            const c = s[this.i]!;
            if (c === "'") {
                const next = s[this.i + 1];
                if (next === "'") {
                    buf += "'";
                    this.i += 2;
                } else if (next === '{' || next === '}' || next === '|' || (next === '#' && inPlural)) {
                    this.i++;
                    while (this.i < s.length) {
                        if (s[this.i] === "'") {
                            if (s[this.i + 1] === "'") {
                                buf += "'";
                                this.i += 2;
                                continue;
                            }
                            this.i++;
                            break;
                        }
                        buf += s[this.i++];
                    }
                } else {
                    buf += "'";
                    this.i++;
                }
            } else if (c === '{') {
                if (buf) parts.push({ t: 'text', v: buf });
                buf = '';
                this.i++;
                parts.push(this.argument(depth, inPlural));
            } else if (c === '}') {
                if (top) throw this.syntax();
                break;
            } else if (c === '#' && inPlural) {
                if (buf) parts.push({ t: 'text', v: buf });
                buf = '';
                parts.push({ t: 'pound' });
                this.i++;
            } else {
                buf += c;
                this.i++;
            }
        }
        if (buf) parts.push({ t: 'text', v: buf });
        return parts;
    }

    private ws(): void {
        while (this.i < this.s.length && ' \t\r\n'.includes(this.s[this.i]!)) this.i++;
    }

    private word(): string {
        this.ws();
        const start = this.i;
        while (this.i < this.s.length && !' \t\r\n,{}'.includes(this.s[this.i]!)) this.i++;
        return this.s.slice(start, this.i).join('');
    }

    private expect(c: string): void {
        this.ws();
        if (this.s[this.i] !== c) throw this.syntax();
        this.i++;
    }

    private argument(depth: number, inPlural: boolean): Part {
        const name = this.word();
        if (!NAME.test(name)) throw this.syntax();
        this.ws();
        if (this.i >= this.s.length) throw this.syntax();
        if (this.s[this.i] === '}') {
            this.i++;
            return { t: 'simple', name };
        }
        this.expect(',');
        const kind = this.word();
        if (kind === 'number' || kind === 'date') {
            this.ws();
            let style: string | undefined;
            if (this.s[this.i] === ',') {
                this.i++;
                style = this.word();
                const allowed = kind === 'number' ? ['integer', 'percent'] : ['full', 'long', 'medium', 'short'];
                if (!allowed.includes(style)) throw this.syntax();
            }
            this.expect('}');
            return kind === 'number' ? { t: 'number', name, style } : { t: 'date', name, style: (style ?? 'medium') as DateStyle };
        }
        if (kind !== 'plural' && kind !== 'selectordinal' && kind !== 'select') throw this.syntax();
        this.expect(',');
        let offset = 0;
        let seenOffset = false;
        const cases: [string, Part[]][] = [];
        for (;;) {
            this.ws();
            if (this.i >= this.s.length) throw this.syntax();
            if (this.s[this.i] === '}') {
                this.i++;
                break;
            }
            const key = this.word();
            if (key.startsWith('offset:') && kind === 'plural' && cases.length === 0 && !seenOffset) {
                let rest = key.slice(7);
                if (rest === '') rest = this.word();
                if (!/^[0-9]+$/.test(rest)) throw this.syntax();
                offset = Number(rest);
                seenOffset = true;
                continue;
            }
            if (key === '' || cases.some(([k]) => k === key)) throw this.syntax();
            const valid = kind === 'select'
                ? NAME.test(key)
                : key.startsWith('=') ? /^=[0-9]+(\.[0-9]+)?$/.test(key) : KEYWORDS.includes(key);
            if (!valid) throw this.syntax();
            this.expect('{');
            const body = this.message(depth + 1, kind !== 'select' || inPlural, false);
            if (this.s[this.i] !== '}') throw this.syntax();
            this.i++;
            cases.push([key, body]);
        }
        if (!cases.some(([k]) => k === 'other')) throw new LocaleError('MISSING_OTHER', name);
        return { t: 'choice', kind, name, offset, cases };
    }
}

function argNumber(a: Arg): string | undefined {
    if (typeof a === 'string') return undefined;
    if (typeof a === 'object') return a.num;
    return toDecimalString(a);
}

function finite(v: string): { neg: boolean; int: string; frac: string } | undefined {
    try {
        const d = parseDec(v);
        return d.kind === 'finite' ? d : undefined;
    } catch {
        return undefined;
    }
}

function decEq(a: string, b: string): boolean {
    const x = finite(a);
    const y = finite(b);
    if (!x || !y) return false;
    const zero = (d: { int: string; frac: string }): boolean => d.int === '0' && /^0*$/.test(d.frac);
    if (zero(x) && zero(y)) return true;
    return x.neg === y.neg && x.int === y.int && x.frac.replace(/0+$/, '') === y.frac.replace(/0+$/, '');
}

function decSub(value: string, offset: number): string | undefined {
    if (offset === 0) return value;
    const d = finite(value)!;
    const scale = d.frac.length;
    if ((d.int + d.frac).length > 36) return undefined;
    const n = BigInt(d.int + d.frac) * (d.neg ? -1n : 1n) - BigInt(offset) * 10n ** BigInt(scale);
    const s = (n < 0n ? -n : n).toString().padStart(scale + 1, '0');
    return (n < 0n ? '-' : '') + s.slice(0, s.length - scale) + (scale ? '.' + s.slice(s.length - scale) : '');
}

function forPlural(value: string): string {
    const d = finite(value)!;
    const [ip, fp] = roundHalfExpand(d.int, d.frac, 3);
    const t = fp.replace(/0+$/, '');
    return t ? `${ip}.${t}` : ip;
}

function lookup(args: Args, name: string): Arg {
    if (!Object.hasOwn(args, name)) throw new LocaleError('MISSING_ARGUMENT', name);
    return args[name]!;
}

function formatArg(locale: string, name: string, value: string, opts?: NumberOptions): string {
    try {
        return formatNumber(locale, value, opts);
    } catch {
        throw new LocaleError('BAD_ARGUMENT', name);
    }
}

function render(locale: string, parts: Part[], args: Args, pound: string | undefined): string {
    let out = '';
    for (const p of parts) {
        switch (p.t) {
            case 'text':
                out += p.v;
                break;
            case 'pound':
                out += pound === undefined ? '#' : formatNumber(locale, pound);
                break;
            case 'simple': {
                const a = lookup(args, p.name);
                const n = argNumber(a);
                out += n === undefined ? (a as string) : formatArg(locale, p.name, n);
                break;
            }
            case 'number': {
                const n = argNumber(lookup(args, p.name));
                if (n === undefined) throw new LocaleError('BAD_ARGUMENT', p.name);
                const opts: NumberOptions = p.style === 'integer' ? { maxFraction: 0 } : p.style === 'percent' ? { style: 'percent' } : {};
                out += formatArg(locale, p.name, n, opts);
                break;
            }
            case 'date': {
                const a = lookup(args, p.name);
                if (typeof a !== 'string') throw new LocaleError('BAD_ARGUMENT', p.name);
                try {
                    out += formatDate(locale, a, p.style);
                } catch {
                    throw new LocaleError('BAD_ARGUMENT', p.name);
                }
                break;
            }
            case 'choice': {
                const a = lookup(args, p.name);
                const other = p.cases.find(([k]) => k === 'other')![1];
                if (p.kind === 'select') {
                    const key = argNumber(a) ?? (a as string);
                    out += render(locale, p.cases.find(([k]) => k === key)?.[1] ?? other, args, pound);
                    break;
                }
                const n = argNumber(a);
                if (n === undefined || !finite(n)) throw new LocaleError('BAD_ARGUMENT', p.name);
                const shown = decSub(n, p.offset);
                if (shown === undefined) throw new LocaleError('BAD_ARGUMENT', p.name);
                const exact = p.cases.find(([k]) => k.startsWith('=') && decEq(k.slice(1), n));
                let chosen: Part[];
                if (exact) chosen = exact[1];
                else {
                    const v = forPlural(shown);
                    const cat = p.kind === 'selectordinal' ? ordinalCategory(locale, v) : pluralCategory(locale, v);
                    chosen = p.cases.find(([k]) => k === cat)?.[1] ?? other;
                }
                out += render(locale, chosen, args, shown);
                break;
            }
        }
    }
    return out;
}

/** Formats an ICU MessageFormat `pattern` with `args` for `locale` (SPEC section 7). */
export function formatMessage(locale: string, pattern: string, args: Args = {}): string {
    const parser = new Parser(Array.from(pattern));
    return render(locale, parser.message(0, false, true), args, undefined);
}

// ---------------------------------------------------------------- catalogs

/** Message id to pattern. */
export type Catalog = Map<string, string>;

const MAX_JSON_DEPTH = 64;

class JsonReader {
    i = 0;
    constructor(private readonly s: string) {}

    bad(): LocaleError {
        return new LocaleError('BAD_JSON', String(this.i));
    }

    ws(): void {
        while (this.i < this.s.length && ' \t\n\r'.includes(this.s[this.i]!)) this.i++;
    }

    string(): string {
        if (this.s[this.i] !== '"') throw this.bad();
        this.i++;
        let out = '';
        for (;;) {
            if (this.i >= this.s.length) throw this.bad();
            const c = this.s[this.i++]!;
            if (c === '"') return out;
            if (c === '\\') {
                const e = this.s[this.i++];
                const simple: Record<string, string> = { '"': '"', '\\': '\\', '/': '/', b: '\b', f: '\f', n: '\n', r: '\r', t: '\t' };
                if (e !== undefined && Object.hasOwn(simple, e)) out += simple[e];
                else if (e === 'u') {
                    let cp = this.hex4();
                    if (cp >= 0xd800 && cp < 0xdc00) {
                        if (this.s[this.i] !== '\\' || this.s[this.i + 1] !== 'u') throw this.bad();
                        this.i += 2;
                        const lo = this.hex4();
                        if (lo < 0xdc00 || lo >= 0xe000) throw this.bad();
                        cp = 0x10000 + ((cp - 0xd800) << 10) + (lo - 0xdc00);
                    } else if (cp >= 0xdc00 && cp < 0xe000) throw this.bad();
                    out += String.fromCodePoint(cp);
                } else throw this.bad();
            } else if (c.charCodeAt(0) < 0x20) throw this.bad();
            else {
                const code = c.charCodeAt(0);
                if (code >= 0xd800 && code < 0xe000) {
                    // A raw surrogate is only valid as part of a pair.
                    const lo = this.s.charCodeAt(this.i);
                    if (code >= 0xdc00 || !(lo >= 0xdc00 && lo < 0xe000)) throw this.bad();
                    out += c + this.s[this.i++];
                } else out += c;
            }
        }
    }

    hex4(): number {
        const h = this.s.slice(this.i, this.i + 4);
        if (!/^[0-9A-Fa-f]{4}$/.test(h)) throw this.bad();
        this.i += 4;
        return parseInt(h, 16);
    }

    skipValue(depth: number): void {
        if (depth > MAX_JSON_DEPTH) throw this.bad();
        this.ws();
        const c = this.s[this.i];
        if (c === '"') {
            this.string();
            return;
        }
        if (c === '{' || c === '[') {
            const close = c === '{' ? '}' : ']';
            this.i++;
            this.ws();
            if (this.s[this.i] === close) {
                this.i++;
                return;
            }
            for (;;) {
                this.ws();
                if (close === '}') {
                    this.string();
                    this.ws();
                    if (this.s[this.i] !== ':') throw this.bad();
                    this.i++;
                }
                this.skipValue(depth + 1);
                this.ws();
                if (this.s[this.i] === ',') this.i++;
                else if (this.s[this.i] === close) {
                    this.i++;
                    return;
                } else throw this.bad();
            }
        }
        for (const lit of ['true', 'false', 'null']) {
            if (this.s.startsWith(lit, this.i)) {
                this.i += lit.length;
                return;
            }
        }
        const m = /^-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?/.exec(this.s.slice(this.i));
        if (!m || m[0] === '' || m[0] === '-') throw this.bad();
        this.i += m[0].length;
    }
}

/** Parses a catalog: a JSON object (RFC 8259) whose values are all strings. */
export function parseCatalog(json: string): Catalog {
    const check = new JsonReader(json);
    check.skipValue(0);
    check.ws();
    if (check.i !== json.length) throw check.bad();
    const r = new JsonReader(json);
    r.ws();
    if (json[r.i] !== '{') throw new LocaleError('NOT_OBJECT');
    r.i++;
    const cat: Catalog = new Map();
    r.ws();
    if (json[r.i] === '}') return cat;
    for (;;) {
        r.ws();
        const key = r.string();
        r.ws();
        r.i++;
        r.ws();
        if (json[r.i] !== '"') throw new LocaleError('NON_STRING_VALUE', key);
        const value = r.string();
        if (cat.has(key)) throw new LocaleError('DUPLICATE_KEY', key);
        cat.set(key, value);
        r.ws();
        if (json[r.i] === ',') r.i++;
        else return cat;
    }
}

/** A catalog-backed message. */
export interface LocalizedMessage {
    code: string;
    messageId: string;
    text: string;
    /** True when the id was missing or its pattern failed; `text` is then `!!id!!`. */
    fallback: boolean;
}

/** Looks up `messageId` and formats it; missing or failing patterns give `!!messageId!!`. */
export function resolve(locale: string, catalog: Catalog, code: string, messageId: string, args: Args = {}): LocalizedMessage {
    const pattern = catalog.get(messageId);
    if (pattern !== undefined) {
        try {
            return { code, messageId, text: formatMessage(locale, pattern, args), fallback: false };
        } catch {
            // fall through to the visible marker
        }
    }
    return { code, messageId, text: `!!${messageId}!!`, fallback: true };
}
