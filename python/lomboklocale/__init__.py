"""LombokLocale: BCP 47 tags and negotiation, CLDR 47 plural rules,
number/percent/currency and date formatting, an ICU MessageFormat subset, and
JSON message catalogs. Same results as the Rust, TypeScript, Go and PHP ports
(docs/SPEC_LombokLocale_v0.2.0.md). No dependencies.
"""
from __future__ import annotations

import re
from dataclasses import dataclass, field
from decimal import Decimal
from typing import Dict, List, Mapping, Optional, Sequence, Tuple, Union

from ._data import DATA

__version__ = "0.2.0"
__all__ = [
    "CLDR_VERSION", "MAX_DEPTH", "MAX_EXPONENT", "LocaleError", "LanguageTag", "LocalizedMessage", "Num",
    "parse_tag", "canonicalize", "negotiate", "data_locale", "plural_category", "ordinal_category",
    "format_number", "format_date", "parse_date", "weekday", "format_message", "parse_catalog", "resolve",
    "to_decimal_string",
]

CLDR_VERSION: str = DATA["cldr"]
MAX_DEPTH = 16
MAX_EXPONENT = 1000

Numeric = Union[int, float, Decimal, str]


class LocaleError(ValueError):
    """Raised by every function that can fail. ``code`` is one of the SPEC
    error codes (section 8); ``detail`` names the offending input."""

    def __init__(self, code: str, detail: str = "") -> None:
        super().__init__(code if detail == "" else "%s: %s" % (code, detail))
        self.code = code
        self.detail = detail


# ---------------------------------------------------------------- tags

@dataclass
class LanguageTag:
    """A well-formed BCP 47 tag in canonical case."""

    language: str
    script: Optional[str] = None
    region: Optional[str] = None
    variants: List[str] = field(default_factory=list)
    extensions: List[List[str]] = field(default_factory=list)
    private_use: List[str] = field(default_factory=list)

    def base(self) -> str:
        """language, script, region and variants joined with ``-``."""
        return "-".join([p for p in (self.language, self.script, self.region) if p] + self.variants)

    def lookup_chain(self) -> List[str]:
        """RFC 4647 fallback list: the base tag, then shorter prefixes."""
        parts = self.base().split("-")
        return ["-".join(parts[:k]) for k in range(len(parts), 0, -1)]

    def __str__(self) -> str:
        parts = [self.base()]
        for e in self.extensions:
            parts += e
        if self.private_use:
            parts += ["x"] + self.private_use
        return "-".join(parts)


_SUBTAG = re.compile(r"^[A-Za-z0-9]{1,8}$")


def _alpha(s: str) -> bool:
    return s.isascii() and s.isalpha()


def parse_tag(tag: str) -> LanguageTag:
    """Parses a tag; ``_`` is accepted as a separator, outer spaces are ignored."""
    t = tag.strip()
    if t == "":
        raise LocaleError("EMPTY")
    subs = re.split(r"[-_]", t)
    for s in subs:
        if not _SUBTAG.match(s):
            raise LocaleError("INVALID_SUBTAG", s)
    lang = subs[0]
    if not (_alpha(lang) and len(lang) not in (1, 4)):
        raise LocaleError("INVALID_SUBTAG", lang)
    out = LanguageTag(language=lang.lower())
    i = 1
    at = lambda k: subs[k] if k < len(subs) else ""  # noqa: E731
    if len(at(i)) == 4 and _alpha(at(i)):
        out.script = at(i)[0].upper() + at(i)[1:].lower()
        i += 1
    if (len(at(i)) == 2 and _alpha(at(i))) or (len(at(i)) == 3 and at(i).isdigit()):
        out.region = at(i).upper()
        i += 1
    while i < len(subs) and (len(at(i)) >= 5 or (len(at(i)) == 4 and at(i)[0].isdigit())):
        v = at(i).lower()
        if v in out.variants:
            raise LocaleError("DUPLICATE_VARIANT", v)
        out.variants.append(v)
        i += 1
    while i < len(subs) and len(at(i)) == 1 and at(i).lower() != "x":
        single = at(i).lower()
        if any(e[0] == single for e in out.extensions):
            raise LocaleError("DUPLICATE_EXTENSION", single)
        i += 1
        ext = [single]
        while i < len(subs) and len(at(i)) >= 2:
            ext.append(at(i).lower())
            i += 1
        if len(ext) == 1:
            raise LocaleError("INVALID_SUBTAG", single)
        out.extensions.append(ext)
    if i < len(subs) and at(i).lower() == "x":
        i += 1
        if i == len(subs):
            raise LocaleError("INVALID_SUBTAG", "x")
        out.private_use = [s.lower() for s in subs[i:]]
        i = len(subs)
    if i < len(subs):
        raise LocaleError("INVALID_SUBTAG", at(i))
    out.extensions.sort()
    return out


def canonicalize(tag: str) -> str:
    """Canonical form of ``tag``, e.g. ``"EN_us"`` -> ``"en-US"``."""
    return str(parse_tag(tag))


def _try_parse(tag: str) -> Optional[LanguageTag]:
    try:
        return parse_tag(tag)
    except LocaleError:
        return None


def negotiate(requested: Sequence[str], available: Sequence[str], default: str) -> str:
    """RFC 4647 lookup over ``requested`` in order (case-insensitive); returns
    the match as written in ``available``, or ``default``."""
    avail = []
    for a in available:
        t = _try_parse(a)
        if t is not None:
            avail.append((t.base().lower(), a))
    for r in requested:
        t = _try_parse(r)
        if t is None:
            continue
        for cand in t.lookup_chain():
            for c, orig in avail:
                if c == cand.lower():
                    return orig
    return default


def data_locale(tag: str) -> str:
    """The CLDR data locale used to format for ``tag`` (SPEC section 2.3)."""
    t = _try_parse(tag)
    if t is None:
        return "en"
    if t.language == "zh" and t.script is None and t.region in ("TW", "HK", "MO"):
        t.script = "Hant"
    for cand in t.lookup_chain():
        if cand in DATA["locales"]:
            return cand
    return "en"


# ---------------------------------------------------------------- decimals

_DEC = re.compile(r"^([+-]?)([0-9]+)(?:\.([0-9]+))?(?:[eE]([+-]?[0-9]{1,4}))?$")


def to_decimal_string(v: Numeric) -> str:
    """Text form of a number: shortest round-trip for floats, exact for Decimal."""
    if isinstance(v, bool):
        raise LocaleError("BAD_NUMBER", str(v))
    if isinstance(v, str):
        return v
    if isinstance(v, int):
        return str(v)
    if isinstance(v, float):
        if v != v:
            return "NaN"
        if v in (float("inf"), float("-inf")):
            return "Infinity" if v > 0 else "-Infinity"
        return repr(v)
    if isinstance(v, Decimal):
        if v.is_nan():
            return "NaN"
        if v.is_infinite():
            return "-Infinity" if v < 0 else "Infinity"
        return str(v)
    raise LocaleError("BAD_NUMBER", repr(v))


def _parse_dec(v: Numeric):
    s = to_decimal_string(v)
    if s == "NaN":
        return ("nan",)
    if s in ("Infinity", "+Infinity", "-Infinity"):
        return ("inf", s.startswith("-"))
    m = _DEC.match(s)
    if not m or abs(int(m.group(4) or "0")) > MAX_EXPONENT:
        raise LocaleError("BAD_NUMBER", s)
    digits = m.group(2) + (m.group(3) or "")
    point = len(m.group(2)) + int(m.group(4) or "0")
    if point <= 0:
        digits = "0" * (1 - point) + digits
        point = 1
    digits = digits.ljust(point, "0")
    return ("finite", m.group(1) == "-", digits[:point].lstrip("0") or "0", digits[point:])


def _round(int_part: str, frac: str, max_digits: int) -> Tuple[str, str]:
    if len(frac) <= max_digits:
        return int_part, frac
    keep = int_part + frac[:max_digits]
    if frac[max_digits] >= "5":
        keep = str(int(keep) + 1).rjust(len(keep), "0")
    split = len(keep) - max_digits
    return keep[:split].lstrip("0") or "0", keep[split:]


# ---------------------------------------------------------------- plural

def _operand(int_part: str, frac: str, op: str, mod: int):
    trimmed = frac.rstrip("0")

    def num(s: str):
        if mod:
            r = 0
            for ch in s:
                r = (r * 10 + ord(ch) - 48) % mod
            return r
        return int(s) if len(s) <= 18 else None

    if op == "n":
        return frac.strip("0") == "", num(int_part)
    if op == "i":
        return True, num(int_part)
    if op == "v":
        return True, len(frac) % mod if mod else len(frac)
    if op == "w":
        return True, len(trimmed) % mod if mod else len(trimmed)
    if op == "f":
        return True, num(frac or "0")
    if op == "t":
        return True, num(trimmed or "0")
    return True, 0


def _select(table, locale: str, value: Numeric) -> str:
    d = _parse_dec(value)
    if d[0] != "finite":
        raise LocaleError("BAD_NUMBER", to_decimal_string(value))
    _, _, ip, fp = d
    t = _try_parse(locale)
    rules = None
    if t is not None:
        for cand in t.lookup_chain():
            if cand in table:
                rules = table[cand]
                break
    if rules is None:
        rules = table.get("root", [])
    for cat, groups in rules:
        for rels in groups:
            ok = True
            for op, mod, neg, ranges in rels:
                is_int, val = _operand(ip, fp, op, mod)
                inside = is_int and val is not None and any(lo <= val <= hi for lo, hi in ranges)
                if inside == bool(neg):
                    ok = False
                    break
            if ok:
                return cat
    return "other"


def plural_category(locale: str, value: Numeric) -> str:
    """Cardinal plural category (``zero``, ``one``, ``two``, ``few``, ``many``, ``other``)."""
    return _select(DATA["plurals"], locale, value)


def ordinal_category(locale: str, value: Numeric) -> str:
    """Ordinal plural category (1st, 2nd, 3rd, ...)."""
    return _select(DATA["ordinals"], locale, value)


# ---------------------------------------------------------------- numbers

def _split_pattern(p: str) -> Tuple[str, str, str]:
    quoted = False
    start, end = -1, 0
    for i, c in enumerate(p):
        if c == "'":
            quoted = not quoted
        elif not quoted and c in "#0,.":
            if start < 0:
                start = i
            end = i + 1
    if start < 0:
        start = end = len(p)
    return p[:start], p[start:end], p[end:]


def _expand(raw: str, d, symbol: str) -> str:
    out = []
    quoted = False
    i = 0
    while i < len(raw):
        c = raw[i]
        if c == "'":
            if raw[i + 1:i + 2] == "'":
                out.append("'")
                i += 2
                continue
            quoted = not quoted
        elif quoted:
            out.append(c)
        elif c == "¤":
            out.append(symbol)
        elif c == "%":
            out.append(d["percent"])
        elif c == "-":
            out.append(d["minus"])
        else:
            out.append(c)
        i += 1
    return "".join(out)


def _group(ip: str, primary: int, secondary: int, min_grouping: int, sep: str) -> str:
    if primary == 0 or len(ip) < primary + min_grouping:
        return ip
    head, parts = ip[:-primary], [ip[-primary:]]
    while len(head) > secondary:
        parts.insert(0, head[-secondary:])
        head = head[:-secondary]
    if head:
        parts.insert(0, head)
    return sep.join(parts)


def format_number(locale: str, value: Numeric, style: str = "decimal", currency: Optional[str] = None,
                  min_fraction: Optional[int] = None, max_fraction: Optional[int] = None) -> str:
    """Formats a number (half away from zero; CLDR grouping and symbols)."""
    d = DATA["locales"][data_locale(locale)]
    code = ""
    if style == "currency":
        if not isinstance(currency, str) or not re.fullmatch(r"[A-Za-z]{3}", currency):
            raise LocaleError("BAD_OPTION", "currency")
        code = currency.upper()
        dmin = dmax = DATA["currencyDigits"].get(code, 2)
        pattern = d["currencyPattern"]
    elif style == "percent":
        dmin, dmax, pattern = 0, 0, d["percentPattern"]
    elif style == "decimal":
        dmin, dmax, pattern = 0, 3, d["decimalPattern"]
    else:
        raise LocaleError("BAD_OPTION", "style")
    minf, maxf = min_fraction, max_fraction
    if minf is not None and maxf is None:
        maxf = max(minf, dmax)
    if maxf is not None and minf is None:
        minf = min(dmin, maxf)
    minf = dmin if minf is None else minf
    maxf = dmax if maxf is None else maxf
    if not all(isinstance(x, int) and not isinstance(x, bool) for x in (minf, maxf)) or not 0 <= minf <= maxf <= 20:
        raise LocaleError("BAD_OPTION", "fractionDigits")
    pos, sep, negp = pattern.partition(";")
    prefix, numpart, suffix = _split_pattern(pos)
    v = _parse_dec(value)
    if v[0] == "nan":
        neg, body = False, d["nan"]
    elif v[0] == "inf":
        neg, body = v[1], d["infinity"]
    else:
        _, neg, ip, fp = v
        if style == "percent":
            fp += "00"
            ip, fp = (ip + fp[:2]).lstrip("0") or "0", fp[2:]
        ip, fp = _round(ip, fp, maxf)
        fp = fp.ljust(minf, "0")
        while len(fp) > minf and fp.endswith("0"):
            fp = fp[:-1]
        groups = numpart.split(".")[0].split(",")
        primary = len(groups[-1]) if len(groups) > 1 else 0
        secondary = len(groups[-2]) if len(groups) > 2 else primary
        body = _group(ip, primary, secondary, d["minGrouping"], d["group"])
        if fp:
            body += d["decimal"] + fp
    if neg and sep:
        np_, _, ns = _split_pattern(negp)
    elif neg:
        np_, ns = "-" + prefix, suffix
    else:
        np_, ns = prefix, suffix
    if style == "currency":
        sym, first_sz, last_sz = d["currencySymbols"].get(code, [code, 0, 0])
        pre, suf = _expand(np_, d, sym), _expand(ns, d, sym)
        if np_.endswith("¤") and not last_sz and body[:1].isdigit():
            pre += "\xa0"
        if ns.startswith("¤") and not first_sz and body[-1:].isdigit():
            suf = "\xa0" + suf
        return pre + body + suf
    return _expand(np_, d, "") + body + _expand(ns, d, "")


# ---------------------------------------------------------------- dates

_DATE = re.compile(r"^([0-9]{4})-([0-9]{2})-([0-9]{2})$")


def _leap(y: int) -> bool:
    return y % 4 == 0 and (y % 100 != 0 or y % 400 == 0)


def _dim(y: int, m: int) -> int:
    return (29 if _leap(y) else 28) if m == 2 else 30 if m in (4, 6, 9, 11) else 31


def parse_date(s: str) -> Tuple[int, int, int]:
    """Parses ``YYYY-MM-DD`` (0001-01-01 to 9999-12-31)."""
    m = _DATE.match(s)
    if not m:
        raise LocaleError("BAD_DATE", s)
    y, mo, d = int(m.group(1)), int(m.group(2)), int(m.group(3))
    if y < 1 or not 1 <= mo <= 12 or not 1 <= d <= _dim(y, mo):
        raise LocaleError("BAD_DATE", s)
    return y, mo, d


def weekday(y: int, m: int, d: int) -> int:
    """Day of the week, 0 = Sunday."""
    p = y - 1
    days = p * 365 + p // 4 - p // 100 + p // 400 + sum(_dim(y, k) for k in range(1, m)) + d - 1
    return (days + 1) % 7


def format_date(locale: str, date: str, style: str = "medium") -> str:
    """Formats ``YYYY-MM-DD`` with the CLDR pattern of ``style`` (Gregorian)."""
    if style not in ("full", "long", "medium", "short"):
        raise LocaleError("BAD_OPTION", "style")
    y, mo, dd = parse_date(date)
    d = DATA["locales"][data_locale(locale)]
    pat = d["dateFormats"][style]
    out = []
    i = 0
    while i < len(pat):
        c = pat[i]
        if c == "'":
            j = i + 1
            while j < len(pat):
                if pat[j] == "'":
                    if pat[j + 1:j + 2] == "'":
                        out.append("'")
                        j += 2
                        continue
                    break
                out.append(pat[j])
                j += 1
            if j == i + 1:
                out.append("'")
            i = j + 1
            continue
        if c.isascii() and c.isalpha():
            j = i
            while j < len(pat) and pat[j] == c:
                j += 1
            n = j - i
            if c == "G":
                out.append(d["era"])
            elif c == "y":
                out.append(str(y % 100).zfill(2) if n == 2 else str(y).zfill(n))
            elif c in "ML":
                out.append(str(mo).zfill(n) if n <= 2 else d["months"][0 if n == 3 else 1][mo - 1])
            elif c == "d":
                out.append(str(dd).zfill(n))
            elif c == "E":
                out.append(d["days"][1 if n == 4 else 0][weekday(y, mo, dd)])
            else:  # pragma: no cover - not used by the CLDR 47 subset
                raise AssertionError("pattern letter %s" % c)
            i = j
            continue
        out.append(c)
        i += 1
    return "".join(out)


# ---------------------------------------------------------------- messages

@dataclass(frozen=True)
class Num:
    """A number argument given as exact decimal text, e.g. ``Num("1.50")``."""

    value: str


Arg = Union[str, int, float, Decimal, Num]
_NAME = re.compile(r"^[A-Za-z0-9_]+$")
_KEYWORDS = ("zero", "one", "two", "few", "many", "other")


class _Parser:
    def __init__(self, s: str) -> None:
        self.s = s
        self.i = 0

    def syntax(self) -> LocaleError:
        return LocaleError("SYNTAX", str(self.i))

    def message(self, depth: int, in_plural: bool, top: bool) -> list:
        if depth > MAX_DEPTH:
            raise LocaleError("TOO_DEEP")
        s = self.s
        parts: list = []
        buf: List[str] = []
        while self.i < len(s):
            c = s[self.i]
            if c == "'":
                nxt = s[self.i + 1:self.i + 2]
                if nxt == "'":
                    buf.append("'")
                    self.i += 2
                elif nxt in ("{", "}", "|") or (nxt == "#" and in_plural):
                    self.i += 1
                    while self.i < len(s):
                        if s[self.i] == "'":
                            if s[self.i + 1:self.i + 2] == "'":
                                buf.append("'")
                                self.i += 2
                                continue
                            self.i += 1
                            break
                        buf.append(s[self.i])
                        self.i += 1
                else:
                    buf.append("'")
                    self.i += 1
            elif c == "{":
                if buf:
                    parts.append(("text", "".join(buf)))
                    buf = []
                self.i += 1
                parts.append(self.argument(depth, in_plural))
            elif c == "}":
                if top:
                    raise self.syntax()
                break
            elif c == "#" and in_plural:
                if buf:
                    parts.append(("text", "".join(buf)))
                    buf = []
                parts.append(("pound",))
                self.i += 1
            else:
                buf.append(c)
                self.i += 1
        if buf:
            parts.append(("text", "".join(buf)))
        return parts

    def ws(self) -> None:
        while self.i < len(self.s) and self.s[self.i] in " \t\r\n":
            self.i += 1

    def word(self) -> str:
        self.ws()
        j = self.i
        while self.i < len(self.s) and self.s[self.i] not in " \t\r\n,{}":
            self.i += 1
        return self.s[j:self.i]

    def expect(self, c: str) -> None:
        self.ws()
        if self.s[self.i:self.i + 1] != c:
            raise self.syntax()
        self.i += 1

    def argument(self, depth: int, in_plural: bool):
        name = self.word()
        if not _NAME.match(name):
            raise self.syntax()
        self.ws()
        if self.i >= len(self.s):
            raise self.syntax()
        if self.s[self.i] == "}":
            self.i += 1
            return ("simple", name)
        self.expect(",")
        kind = self.word()
        if kind in ("number", "date"):
            self.ws()
            style = None
            if self.s[self.i:self.i + 1] == ",":
                self.i += 1
                style = self.word()
                allowed = ("integer", "percent") if kind == "number" else ("full", "long", "medium", "short")
                if style not in allowed:
                    raise self.syntax()
            self.expect("}")
            return (kind, name, style if kind == "number" else (style or "medium"))
        if kind not in ("plural", "selectordinal", "select"):
            raise self.syntax()
        self.expect(",")
        offset = 0
        seen_offset = False
        cases: Dict[str, list] = {}
        while True:
            self.ws()
            if self.i >= len(self.s):
                raise self.syntax()
            if self.s[self.i] == "}":
                self.i += 1
                break
            key = self.word()
            if key.startswith("offset:") and kind == "plural" and not cases and not seen_offset:
                rest = key[7:] or self.word()
                if not re.fullmatch(r"[0-9]+", rest):
                    raise self.syntax()
                offset, seen_offset = int(rest), True
                continue
            if key == "" or key in cases:
                raise self.syntax()
            if kind == "select":
                valid = bool(_NAME.match(key))
            elif key.startswith("="):
                valid = bool(re.fullmatch(r"=[0-9]+(\.[0-9]+)?", key))
            else:
                valid = key in _KEYWORDS
            if not valid:
                raise self.syntax()
            self.expect("{")
            body = self.message(depth + 1, kind != "select" or in_plural, False)
            if self.s[self.i:self.i + 1] != "}":
                raise self.syntax()
            self.i += 1
            cases[key] = body
        if "other" not in cases:
            raise LocaleError("MISSING_OTHER", name)
        return ("choice", kind, name, offset, cases)


def _arg_number(a: Arg) -> Optional[str]:
    if isinstance(a, str):
        return None
    if isinstance(a, Num):
        return a.value
    try:
        return to_decimal_string(a)
    except LocaleError:
        return "?"


def _finite(v: str):
    try:
        d = _parse_dec(v)
    except LocaleError:
        return None
    return d if d[0] == "finite" else None


def _dec_eq(a: str, b: str) -> bool:
    x, y = _finite(a), _finite(b)
    if x is None or y is None:
        return False
    if x[2] == "0" and x[3].strip("0") == "" and y[2] == "0" and y[3].strip("0") == "":
        return True
    return x[1] == y[1] and x[2] == y[2] and x[3].rstrip("0") == y[3].rstrip("0")


def _dec_sub(value: str, offset: int) -> Optional[str]:
    if offset == 0:
        return value
    _, neg, ip, fp = _finite(value)
    if len(ip + fp) > 36:
        return None
    scale = len(fp)
    n = int(ip + fp) * (-1 if neg else 1) - offset * 10 ** scale
    s = str(abs(n)).rjust(scale + 1, "0")
    return ("-" if n < 0 else "") + s[:len(s) - scale] + ("." + s[len(s) - scale:] if scale else "")


def _for_plural(value: str) -> str:
    _, _, ip, fp = _finite(value)
    ip, fp = _round(ip, fp, 3)
    fp = fp.rstrip("0")
    return ip + ("." + fp if fp else "")


def _lookup(args: Mapping[str, Arg], name: str) -> Arg:
    if name not in args:
        raise LocaleError("MISSING_ARGUMENT", name)
    return args[name]


def _fmt_arg(locale: str, name: str, value: str, **kw) -> str:
    try:
        return format_number(locale, value, **kw)
    except LocaleError:
        raise LocaleError("BAD_ARGUMENT", name) from None


def _render(locale: str, parts: list, args: Mapping[str, Arg], pound: Optional[str]) -> str:
    out: List[str] = []
    for p in parts:
        kind = p[0]
        if kind == "text":
            out.append(p[1])
        elif kind == "pound":
            out.append("#" if pound is None else format_number(locale, pound))
        elif kind == "simple":
            a = _lookup(args, p[1])
            n = _arg_number(a)
            out.append(a if n is None else _fmt_arg(locale, p[1], n))
        elif kind == "number":
            n = _arg_number(_lookup(args, p[1]))
            if n is None:
                raise LocaleError("BAD_ARGUMENT", p[1])
            kw = {"max_fraction": 0} if p[2] == "integer" else {"style": "percent"} if p[2] == "percent" else {}
            out.append(_fmt_arg(locale, p[1], n, **kw))
        elif kind == "date":
            a = _lookup(args, p[1])
            if not isinstance(a, str):
                raise LocaleError("BAD_ARGUMENT", p[1])
            try:
                out.append(format_date(locale, a, p[2]))
            except LocaleError:
                raise LocaleError("BAD_ARGUMENT", p[1]) from None
        else:
            _, ckind, name, offset, cases = p
            a = _lookup(args, name)
            if ckind == "select":
                key = a if isinstance(a, str) else _arg_number(a)
                out.append(_render(locale, cases.get(key, cases["other"]), args, pound))
                continue
            n = _arg_number(a)
            if n is None or _finite(n) is None:
                raise LocaleError("BAD_ARGUMENT", name)
            shown = _dec_sub(n, offset)
            if shown is None:
                raise LocaleError("BAD_ARGUMENT", name)
            chosen = None
            for key, body in cases.items():
                if key.startswith("=") and _dec_eq(key[1:], n):
                    chosen = body
                    break
            if chosen is None:
                v = _for_plural(shown)
                cat = ordinal_category(locale, v) if ckind == "selectordinal" else plural_category(locale, v)
                chosen = cases.get(cat, cases["other"])
            out.append(_render(locale, chosen, args, shown))
    return "".join(out)


def format_message(locale: str, pattern: str, args: Optional[Mapping[str, Arg]] = None) -> str:
    """Formats an ICU MessageFormat ``pattern``. ``str`` arguments are text;
    ``int``, ``float``, ``Decimal`` and :class:`Num` are numbers (use
    ``Num("1.50")`` or ``Decimal("1.50")`` for exact decimals)."""
    parser = _Parser(pattern)
    return _render(locale, parser.message(0, False, True), args or {}, None)


# ---------------------------------------------------------------- catalogs

_MAX_JSON_DEPTH = 64
_ESCAPES = {'"': '"', "\\": "\\", "/": "/", "b": "\b", "f": "\f", "n": "\n", "r": "\r", "t": "\t"}
_NUMBER = re.compile(r"-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?")


class _Json:
    def __init__(self, s: str) -> None:
        self.s = s
        self.i = 0

    def bad(self) -> LocaleError:
        return LocaleError("BAD_JSON", str(self.i))

    def ws(self) -> None:
        while self.i < len(self.s) and self.s[self.i] in " \t\n\r":
            self.i += 1

    def hex4(self) -> int:
        h = self.s[self.i:self.i + 4]
        if not re.fullmatch(r"[0-9A-Fa-f]{4}", h):
            raise self.bad()
        self.i += 4
        return int(h, 16)

    def string(self) -> str:
        if self.s[self.i:self.i + 1] != '"':
            raise self.bad()
        self.i += 1
        out: List[str] = []
        while True:
            if self.i >= len(self.s):
                raise self.bad()
            c = self.s[self.i]
            self.i += 1
            if c == '"':
                return "".join(out)
            if c == "\\":
                e = self.s[self.i:self.i + 1]
                self.i += 1
                if e in _ESCAPES:
                    out.append(_ESCAPES[e])
                elif e == "u":
                    cp = self.hex4()
                    if 0xD800 <= cp < 0xDC00:
                        if self.s[self.i:self.i + 2] != "\\u":
                            raise self.bad()
                        self.i += 2
                        lo = self.hex4()
                        if not 0xDC00 <= lo < 0xE000:
                            raise self.bad()
                        cp = 0x10000 + ((cp - 0xD800) << 10) + (lo - 0xDC00)
                    elif 0xDC00 <= cp < 0xE000:
                        raise self.bad()
                    out.append(chr(cp))
                else:
                    raise self.bad()
            elif ord(c) < 0x20 or 0xD800 <= ord(c) < 0xE000:
                raise self.bad()
            else:
                out.append(c)

    def skip(self, depth: int) -> None:
        if depth > _MAX_JSON_DEPTH:
            raise self.bad()
        self.ws()
        c = self.s[self.i:self.i + 1]
        if c == '"':
            self.string()
            return
        if c in ("{", "["):
            close = "}" if c == "{" else "]"
            self.i += 1
            self.ws()
            if self.s[self.i:self.i + 1] == close:
                self.i += 1
                return
            while True:
                self.ws()
                if close == "}":
                    self.string()
                    self.ws()
                    if self.s[self.i:self.i + 1] != ":":
                        raise self.bad()
                    self.i += 1
                self.skip(depth + 1)
                self.ws()
                nxt = self.s[self.i:self.i + 1]
                self.i += 1
                if nxt == close:
                    return
                if nxt != ",":
                    raise self.bad()
        for lit in ("true", "false", "null"):
            if self.s.startswith(lit, self.i):
                self.i += len(lit)
                return
        m = _NUMBER.match(self.s, self.i)
        if not m:
            raise self.bad()
        self.i = m.end()


def parse_catalog(text: str) -> Dict[str, str]:
    """Parses a catalog: a JSON object (RFC 8259) whose values are all strings."""
    check = _Json(text)
    check.skip(0)
    check.ws()
    if check.i != len(text):
        raise check.bad()
    r = _Json(text)
    r.ws()
    if text[r.i:r.i + 1] != "{":
        raise LocaleError("NOT_OBJECT")
    r.i += 1
    out: Dict[str, str] = {}
    r.ws()
    if text[r.i:r.i + 1] == "}":
        return out
    while True:
        r.ws()
        key = r.string()
        r.ws()
        r.i += 1
        r.ws()
        if text[r.i:r.i + 1] != '"':
            raise LocaleError("NON_STRING_VALUE", key)
        value = r.string()
        if key in out:
            raise LocaleError("DUPLICATE_KEY", key)
        out[key] = value
        r.ws()
        if text[r.i:r.i + 1] == ",":
            r.i += 1
        else:
            return out


@dataclass
class LocalizedMessage:
    """A catalog-backed message; ``fallback`` is true when ``text`` is ``!!id!!``."""

    code: str
    message_id: str
    text: str
    fallback: bool


def resolve(locale: str, catalog: Mapping[str, str], code: str, message_id: str,
            args: Optional[Mapping[str, Arg]] = None) -> LocalizedMessage:
    """Looks up ``message_id`` and formats it; a missing id or a failing
    pattern gives ``!!message_id!!``."""
    pattern = catalog.get(message_id)
    if pattern is not None:
        try:
            return LocalizedMessage(code, message_id, format_message(locale, pattern, args), False)
        except LocaleError:
            pass
    return LocalizedMessage(code, message_id, "!!%s!!" % message_id, True)
