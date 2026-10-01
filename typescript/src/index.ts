// LombokLocale — TypeScript port (zero dependencies). Behaviour is defined by
// SPEC_LombokLocale and verified against ../vectors/lomboklocale-vectors-v1.json.

import { asciiLower, asciiUpper, chars, cmpCodePoints, eqIgnoreAsciiCase, isAsciiAlnum, isAsciiAlpha, isRustWhitespace, rustTrim } from "./compat.js";

// ───────────────────────────── BCP-47 ─────────────────────────────

export interface LocaleTag { language: string; script?: string; region?: string; variants: string[] }
export type LocaleError = "Empty" | "InvalidSubtag";
export type Result<T, E> = { ok: true; value: T } | { ok: false; error: E };
const ok = <T>(value: T): { ok: true; value: T } => ({ ok: true, value });
const err = <E>(error: E): { ok: false; error: E } => ({ ok: false, error });

export function localeToString(t: LocaleTag): string {
  let out = t.language;
  if (t.script) out += "-" + t.script;
  if (t.region) out += "-" + t.region;
  for (const v of t.variants) out += "-" + v;
  return out;
}

export function parseBcp47(tag: string): Result<LocaleTag, LocaleError> {
  const trimmed = rustTrim(tag);
  if (trimmed === "") return err("Empty");
  const parts = trimmed.split(/[-_]/);
  let idx = 1;
  const langRaw = parts[0] ?? "";
  if (!isAsciiAlpha(langRaw) || langRaw.length < 2 || langRaw.length > 8) return err("InvalidSubtag");
  const language = asciiLower(langRaw);
  let script: string | undefined;
  let p = parts[idx];
  if (p !== undefined && p.length === 4 && isAsciiAlpha(p)) {
    const l = asciiLower(p);
    script = asciiUpper(l[0]!) + l.slice(1);
    idx++;
  }
  let region: string | undefined;
  p = parts[idx];
  if (p !== undefined && ((p.length === 2 && isAsciiAlpha(p)) || (p.length === 3 && /^[0-9]+$/.test(p)))) {
    region = asciiUpper(p);
    idx++;
  }
  const variants: string[] = [];
  while ((p = parts[idx]) !== undefined) {
    if (isAsciiAlnum(p) && p.length >= 4) { variants.push(asciiLower(p)); idx++; } else return err("InvalidSubtag");
  }
  return ok({ language, script, region, variants });
}

export function negotiate(requested: string[], available: string[], dflt: string): string {
  for (const req of requested) {
    const r = parseBcp47(req);
    if (!r.ok) continue;
    const t = r.value;
    let chain: string[] = [localeToString(t)];
    let base = t.language;
    if (t.script) base += "-" + t.script;
    if (t.region) chain.push(base + "-" + t.region);
    chain.push(base);
    if (t.script && t.region) chain.push(`${t.language}-${t.region}`);
    chain.push(t.language);
    chain = chain.filter((c, i) => i === 0 || c !== chain[i - 1]); // Vec::dedup (consecutive only)
    for (const cand of chain) for (const av of available) if (eqIgnoreAsciiCase(av, cand)) return av;
  }
  return dflt;
}

// ───────────────────────────── plural ─────────────────────────────

export type PluralCategory = "zero" | "one" | "two" | "few" | "many" | "other";
const lang = (locale: string): string => locale.split(/[-_]/)[0] ?? locale;
const big = (n: number | bigint): bigint => (typeof n === "bigint" ? n : BigInt(n));

export function pluralCategory(locale: string, nIn: number | bigint): PluralCategory {
  const n = big(nIn), m10 = n % 10n, m100 = n % 100n;
  switch (lang(locale)) {
    case "en": case "de": case "nl": case "sv": case "da": case "no": case "it": case "el": case "fi": case "hu": case "tr": case "et":
      return n === 1n ? "one" : "other";
    case "fr": case "pt":
      return n === 0n || n === 1n ? "one" : "other";
    case "ru": case "uk": case "pl": case "sr": case "hr": case "bs":
      if (m10 === 1n && m100 !== 11n) return "one";
      if (m10 >= 2n && m10 <= 4n && !(m100 >= 12n && m100 <= 14n)) return "few";
      return "many";
    case "ar":
      if (n === 0n) return "zero";
      if (n === 1n) return "one";
      if (n === 2n) return "two";
      if (m100 >= 3n && m100 <= 10n) return "few";
      if (m100 >= 11n && m100 <= 99n) return "many";
      return "other";
    case "id": case "jv": case "su": case "ms": case "ja": case "zh": case "ko": case "vi": case "th": case "km": case "lo": case "my":
      return "other";
    case "cy":
      return n === 0n ? "zero" : n === 1n ? "one" : n === 2n ? "two" : n === 3n ? "few" : n === 6n ? "many" : "other";
    default:
      return n === 1n ? "one" : "other";
  }
}

export function ordinalCategory(locale: string, nIn: number | bigint): PluralCategory {
  const n = big(nIn), m10 = n % 10n, m100 = n % 100n;
  switch (lang(locale)) {
    case "en":
      if (m10 === 1n && m100 !== 11n) return "one";
      if (m10 === 2n && m100 !== 12n) return "two";
      if (m10 === 3n && m100 !== 13n) return "few";
      return "other";
    case "fr": return n === 1n ? "one" : "other";
    case "it": return n === 11n || n === 8n || n === 80n || n === 800n ? "many" : "other";
    case "sv": return (m10 === 1n || m10 === 2n) && m100 !== 11n && m100 !== 12n ? "one" : "other";
    case "hu": return n === 1n || n === 5n ? "one" : "other";
    default: return "other";
  }
}

// ───────────────────────────── message format ─────────────────────────────

/** string → Str; bigint/integer number → Int/UInt by sign; non-integer number → Float. */
export type ArgValue = string | number | bigint;
export type Args = Record<string, ArgValue> | Map<string, ArgValue>;
export type MessageError = "UnbalancedBraces" | "UnknownVariable" | "MalformedSelector" | "TooDeep";
export const MAX_DEPTH = 16;
const U64_MAX = 2n ** 64n - 1n;

function display(v: ArgValue): string {
  return typeof v === "string" ? v : String(v);
}
function asU64(v: ArgValue): bigint | null {
  if (typeof v === "string") return null;
  if (typeof v === "bigint") return v >= 0n ? v : null;
  if (Number.isNaN(v) || v < 0) return null;
  if (!Number.isFinite(v)) return U64_MAX;
  const t = BigInt(Math.trunc(v));
  return t > U64_MAX ? U64_MAX : t;
}
function lookup(args: Args, name: string): ArgValue | undefined {
  if (args instanceof Map) return args.get(name);
  return Object.prototype.hasOwnProperty.call(args, name) ? args[name] : undefined;
}

class MsgErr extends Error { constructor(public kind: MessageError) { super(kind); } }

function readBlock(cs: string[], start: number): [string, number] {
  let depth = 0, i = start, inner = "";
  for (;;) {
    if (i >= cs.length) throw new MsgErr("UnbalancedBraces");
    const c = cs[i]!;
    if (c === "{") { depth++; if (depth > 1) inner += c; }
    else if (c === "}") { depth--; if (depth === 0) return [inner, i + 1]; inner += c; }
    else inner += c;
    i++;
  }
}
function parseCases(s: string): Array<[string, string]> {
  const cs = chars(s), cases: Array<[string, string]> = [];
  let i = 0;
  while (i < cs.length) {
    while (i < cs.length && isRustWhitespace(cs[i]!)) i++;
    if (i >= cs.length) break;
    const ks = i;
    while (i < cs.length && cs[i] !== "{") i++;
    const key = rustTrim(cs.slice(ks, i).join(""));
    if (i >= cs.length) throw new MsgErr("UnbalancedBraces");
    const [body, next] = readBlock(cs, i);
    cases.push([key, body]);
    i = next;
  }
  return cases;
}
function pick(cases: Array<[string, string]>, ...keys: string[]): [string, string] {
  for (const k of keys) { const f = cases.find(([ck]) => ck === k); if (f) return f; }
  throw new MsgErr("MalformedSelector");
}
function renderBlock(locale: string, inner: string, args: Args, depth: number): string {
  const trimmed = rustTrim(inner);
  if (!trimmed.includes(",")) {
    const v = lookup(args, trimmed);
    if (v === undefined) throw new MsgErr("UnknownVariable");
    return display(v);
  }
  const first = trimmed.indexOf(","), second = trimmed.indexOf(",", first + 1);
  const varName = rustTrim(trimmed.slice(0, first));
  const kind = rustTrim(second === -1 ? trimmed.slice(first + 1) : trimmed.slice(first + 1, second));
  const rest = second === -1 ? "" : trimmed.slice(second + 1);
  const val = lookup(args, varName);
  if (val === undefined) throw new MsgErr("UnknownVariable");
  const cases = parseCases(rest);
  if (kind === "plural" || kind === "selectordinal") {
    const n = asU64(val);
    if (n === null) throw new MsgErr("MalformedSelector");
    const cat = kind === "plural" ? pluralCategory(locale, n) : ordinalCategory(locale, n);
    const chosen = pick(cases, cat, "other", "*");
    return formatDepth(locale, chosen[1], args, depth + 1).split("#").join(n.toString());
  }
  if (kind === "select") {
    const chosen = pick(cases, display(val), "*", "other");
    return formatDepth(locale, chosen[1], args, depth + 1);
  }
  throw new MsgErr("MalformedSelector");
}
function formatDepth(locale: string, pattern: string, args: Args, depth: number): string {
  if (depth > MAX_DEPTH) throw new MsgErr("TooDeep");
  const cs = chars(pattern);
  let out = "", i = 0;
  while (i < cs.length) {
    const c = cs[i]!;
    if (c === "{") { const [block, next] = readBlock(cs, i); out += renderBlock(locale, block, args, depth); i = next; }
    else if (c === "}") throw new MsgErr("UnbalancedBraces");
    else { out += c; i++; }
  }
  return out;
}
export function formatMessage(locale: string, pattern: string, args: Args = {}): Result<string, MessageError> {
  try { return ok(formatDepth(locale, pattern, args, 0)); }
  catch (e) { if (e instanceof MsgErr) return err(e.kind); throw e; }
}

// ───────────────────────────── numbers ─────────────────────────────

const NBSP = "\u00a0";
function numFmt(locale: string): { group: string; decimal: string } {
  switch (lang(locale)) {
    case "en": case "zh": case "ja": case "ko": return { group: ",", decimal: "." };
    case "id": case "de": case "nl": case "it": case "es": case "pt": case "ru": case "tr": case "vi": return { group: ".", decimal: "," };
    case "fr": case "sv": case "pl": case "fi": return { group: NBSP, decimal: "," };
    default: return { group: ",", decimal: "." };
  }
}
function groupDigits(digits: string, sep: string): string {
  const cs = chars(digits);
  let out = "";
  cs.forEach((c, i) => { if (i > 0 && (cs.length - i) % 3 === 0) out += sep; out += c; });
  return out;
}
export function formatInteger(value: bigint | number, locale: string): string {
  const v = BigInt(value), f = numFmt(locale);
  const g = groupDigits((v < 0n ? -v : v).toString(), f.group);
  return v < 0n ? "-" + g : g;
}
export const MAX_DECIMALS = 15;
const FIXED_LIMIT = 1e20;
export function formatFloat(value: number, decimalsIn: number, locale: string): string {
  if (Number.isNaN(value)) return "NaN";
  if (!Number.isFinite(value)) return value < 0 ? "-\u221e" : "\u221e";
  const f = numFmt(locale), decimals = Math.min(decimalsIn, MAX_DECIMALS);
  const abs = value < 0 ? -value : value;
  if (abs >= FIXED_LIMIT) return rustExp(value);
  let scale = 1n;
  for (let i = 0; i < decimals; i++) scale *= 10n;
  const scaled = BigInt(Math.floor(abs * Number(scale) + 0.5));
  const intPart = scaled / scale, frac = scaled - intPart * scale;
  const neg = value < 0 && scaled !== 0n;
  let out = neg ? "-" : "";
  out += groupDigits(intPart.toString(), f.group);
  if (decimals > 0) out += f.decimal + frac.toString().padStart(decimals, "0");
  return out;
}
/** Rust `{:e}` for f64: shortest round-trip digits, no '+' in the exponent. */
function rustExp(v: number): string { return v.toExponential().replace("e+", "e"); }

function currencySymbol(code: string): string {
  switch (code) { case "USD": return "$"; case "IDR": return "Rp"; case "EUR": return "\u20ac"; case "GBP": return "\u00a3"; case "JPY": case "CNY": return "\u00a5"; default: return ""; }
}
export function formatCurrency(value: number, code: string, locale: string): string {
  const amount = formatFloat(value, 2, locale), sym = currencySymbol(code);
  switch (lang(locale)) {
    case "en": case "id": return sym + amount;
    case "fr": case "sv": case "pl": case "fi": return amount + NBSP + sym;
    default: return sym + " " + amount;
  }
}

// ───────────────────────────── dates ─────────────────────────────

export interface SimpleDate { year: number; month: number; day: number }
export type DateStyle = "iso" | "short" | "long";
const MONTHS_ID = ["Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus", "September", "Oktober", "November", "Desember"];
const MONTHS_EN = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];
/** Rust `{:0Nd}`: sign counts toward the width. */
function pad(n: number, width: number): string {
  const s = String(Math.abs(n));
  return n < 0 ? "-" + s.padStart(Math.max(width - 1, 0), "0") : s.padStart(width, "0");
}
export function formatDate(d: SimpleDate, locale: string, style: DateStyle): string {
  const l = lang(locale);
  if (style === "iso") return `${pad(d.year, 4)}-${pad(d.month, 2)}-${pad(d.day, 2)}`;
  if (style === "short") return l === "en" ? `${pad(d.month, 2)}/${pad(d.day, 2)}/${pad(d.year, 4)}` : `${pad(d.day, 2)}/${pad(d.month, 2)}/${pad(d.year, 4)}`;
  const names = l === "id" ? MONTHS_ID : MONTHS_EN;
  const name = names[Math.max(d.month - 1, 0)] ?? "?";
  return l === "id" ? `${d.day} ${name} ${d.year}` : `${name} ${d.day}, ${d.year}`;
}
function daysInMonth(year: number, month: number): number {
  if ([1, 3, 5, 7, 8, 10, 12].includes(month)) return 31;
  if ([4, 6, 9, 11].includes(month)) return 30;
  if (month === 2) return (year % 4 === 0 && year % 100 !== 0) || year % 400 === 0 ? 29 : 28;
  return 0;
}
export function parseIsoDate(s: string): SimpleDate | null {
  if (!/^[0-9]{4}-[0-9]{2}-[0-9]{2}$/.test(s)) return null;
  const year = Number(s.slice(0, 4)), month = Number(s.slice(5, 7)), day = Number(s.slice(8, 10));
  if (month === 0 || month > 12 || day === 0 || day > daysInMonth(year, month)) return null;
  return { year, month, day };
}

// ───────────────────────────── catalogs ─────────────────────────────

export type CatalogError = "UnexpectedEnd" | "UnexpectedChar" | "ExpectedObject";
export class Catalog {
  private entries = new Map<string, string>();
  get(id: string): string | undefined { return this.entries.get(id); }
  get size(): number { return this.entries.size; }
  insert(id: string, text: string): void { this.entries.set(id, text); }
  /** (messageId, text) in code-point order, like Rust's BTreeMap. */
  *iter(): IterableIterator<[string, string]> {
    for (const k of [...this.entries.keys()].sort(cmpCodePoints)) yield [k, this.entries.get(k)!];
  }
}
class CatErr extends Error { constructor(public kind: CatalogError) { super(kind); } }

export function parseCatalog(json: string): Result<Catalog, CatalogError> {
  try { return ok(parseCatalogInner(json)); }
  catch (e) { if (e instanceof CatErr) return err(e.kind); throw e; }
}
function parseCatalogInner(json: string): Catalog {
  const cs = chars(json);
  let i = 0;
  const ws = () => { while (i < cs.length && isRustWhitespace(cs[i]!)) i++; };
  const expect = (c: string) => {
    if (i >= cs.length) throw new CatErr("UnexpectedEnd");
    if (cs[i] !== c) throw new CatErr("UnexpectedChar");
    i++;
  };
  ws();
  if (cs[i] !== "{") throw new CatErr("ExpectedObject");
  i++;
  ws();
  const cat = new Catalog();
  const trailing = () => { ws(); if (i < cs.length) throw new CatErr("UnexpectedChar"); };
  if (cs[i] === "}") { i++; trailing(); return cat; }
  for (;;) {
    ws();
    const key = parseString(cs, () => i, (n) => { i = n; });
    ws(); expect(":"); ws();
    const value = parseString(cs, () => i, (n) => { i = n; });
    cat.insert(key, value);
    ws();
    const c = cs[i];
    if (c === ",") i++;
    else if (c === "}") { i++; break; }
    else if (c === undefined) throw new CatErr("UnexpectedEnd");
    else throw new CatErr("UnexpectedChar");
  }
  trailing();
  return cat;
}
function parseString(cs: string[], get: () => number, set: (n: number) => void): string {
  let i = get();
  if (i >= cs.length) throw new CatErr("UnexpectedEnd");
  if (cs[i] !== '"') throw new CatErr("UnexpectedChar");
  i++;
  let out = "";
  for (;;) {
    if (i >= cs.length) throw new CatErr("UnexpectedEnd");
    const c = cs[i++]!;
    if (c === '"') { set(i); return out; }
    if (c !== "\\") { out += c; continue; }
    if (i >= cs.length) throw new CatErr("UnexpectedEnd");
    const esc = cs[i++]!;
    switch (esc) {
      case "n": out += "\n"; break;
      case "t": out += "\t"; break;
      case "r": out += "\r"; break;
      case '"': out += '"'; break;
      case "\\": out += "\\"; break;
      case "/": out += "/"; break;
      case "u": {
        let hex = "";
        for (let k = 0; k < 4; k++) { hex += cs[i] ?? "0"; i++; }   // Rust pads with '0' past EOF but still advances
        // JSON: exactly four hex digits; anything else is dropped.
        if (/^[0-9a-fA-F]{4}$/.test(hex)) {
          const code = parseInt(hex, 16);
          if (code <= 0x10ffff && !(code >= 0xd800 && code <= 0xdfff)) out += String.fromCodePoint(code);
        }
        break;
      }
      default: throw new CatErr("UnexpectedChar");
    }
  }
}

export interface LocalizedMessage { code: string; messageId: string; text: string }
export function resolve(locale: string, catalog: Catalog, code: string, messageId: string, args: Args = {}): LocalizedMessage {
  const pattern = catalog.get(messageId);
  let text = `!!${messageId}!!`;
  if (pattern !== undefined) {
    const r = formatMessage(locale, pattern, args);
    if (r.ok) text = r.value;
  }
  return { code, messageId, text };
}
