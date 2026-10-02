#!/usr/bin/env python3
"""Builds vectors/lomboklocale-vectors-v1.json from an independent reference model.

The model below is written directly from the SPEC (docs/SPEC_LombokLocale_v0.2.0.md)
and reads the CLDR 47 subset in data/cldr47.json. It shares no code with the five
ports. Hand-written expectations (``want``) are checked against the model and the
build fails on any mismatch. Number, currency, percent, date and plural results are
additionally cross-checked against ICU by vectors/check_icu.mjs.

    python3 vectors/build_vectors.py   # writes the JSON and vectors/SHA256SUMS
"""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CLDR = json.loads((ROOT / "data" / "cldr47.json").read_text(encoding="utf-8"))
DATA = CLDR["data"]


class Err(Exception):
    def __init__(self, code, detail=""):
        super().__init__(code)
        self.code = code
        self.detail = detail


# ---------------------------------------------------------------- BCP 47

ALPHA = re.compile(r"^[A-Za-z]+$")
ALNUM = re.compile(r"^[A-Za-z0-9]+$")


def parse_tag(tag):
    """Returns a dict with canonical parts or raises Err."""
    if tag.strip() == "":
        raise Err("EMPTY")
    subs = re.split(r"[-_]", tag.strip())
    for s in subs:
        if s == "" or not ALNUM.match(s) or len(s) > 8:
            raise Err("INVALID_SUBTAG", s)
    i = 0
    lang = subs[0]
    if not (ALPHA.match(lang) and (2 <= len(lang) <= 3 or 5 <= len(lang) <= 8)):
        raise Err("INVALID_SUBTAG", lang)
    out = {"language": lang.lower(), "script": None, "region": None, "variants": [], "extensions": [], "private": []}
    i = 1
    if i < len(subs) and len(subs[i]) == 4 and ALPHA.match(subs[i]):
        out["script"] = subs[i][0].upper() + subs[i][1:].lower()
        i += 1
    if i < len(subs) and ((len(subs[i]) == 2 and ALPHA.match(subs[i])) or (len(subs[i]) == 3 and subs[i].isdigit())):
        out["region"] = subs[i].upper()
        i += 1
    while i < len(subs) and (5 <= len(subs[i]) <= 8 or (len(subs[i]) == 4 and subs[i][0].isdigit())):
        v = subs[i].lower()
        if v in out["variants"]:
            raise Err("DUPLICATE_VARIANT", v)
        out["variants"].append(v)
        i += 1
    seen = set()
    while i < len(subs) and len(subs[i]) == 1 and subs[i].lower() != "x":
        single = subs[i].lower()
        if single in seen:
            raise Err("DUPLICATE_EXTENSION", single)
        seen.add(single)
        i += 1
        body = []
        while i < len(subs) and 2 <= len(subs[i]) <= 8:
            body.append(subs[i].lower())
            i += 1
        if not body:
            raise Err("INVALID_SUBTAG", single)
        out["extensions"].append([single] + body)
    if i < len(subs) and subs[i].lower() == "x":
        i += 1
        if i == len(subs):
            raise Err("INVALID_SUBTAG", "x")
        while i < len(subs):
            out["private"].append(subs[i].lower())
            i += 1
    if i < len(subs):
        raise Err("INVALID_SUBTAG", subs[i])
    return out


def tag_string(p, with_ext=True):
    parts = [p["language"]]
    if p["script"]:
        parts.append(p["script"])
    if p["region"]:
        parts.append(p["region"])
    parts += p["variants"]
    if with_ext:
        for e in sorted(p["extensions"]):
            parts += e
        if p["private"]:
            parts += ["x"] + p["private"]
    return "-".join(parts)


def lookup_chain(p):
    parts = tag_string(p, with_ext=False).split("-")
    return ["-".join(parts[:k]) for k in range(len(parts), 0, -1)]


def negotiate(requested, available, default):
    canon = []
    for a in available:
        try:
            canon.append((tag_string(parse_tag(a), with_ext=False).lower(), a))
        except Err:
            pass
    for r in requested:
        try:
            p = parse_tag(r)
        except Err:
            continue
        for cand in lookup_chain(p):
            for c, orig in canon:
                if c == cand.lower():
                    return orig
    return default


def data_locale(tag):
    try:
        p = parse_tag(tag)
    except Err:
        return "en"
    if p["language"] == "zh" and p["script"] is None and p["region"] in ("TW", "HK", "MO"):
        p = dict(p, script="Hant")
    for cand in lookup_chain(p):
        if cand in DATA:
            return cand
    return "en"


def rules_locale(tag, table):
    try:
        p = parse_tag(tag)
    except Err:
        return None
    for cand in lookup_chain(p):
        if cand in table:
            return cand
    return None


# ---------------------------------------------------------------- decimals

DEC = re.compile(r"^([+-]?)(\d+)(?:\.(\d+))?(?:[eE]([+-]?\d+))?$")


def parse_decimal(s):
    """-> (neg, int_digits, frac_digits) exact, or ('nan'|'inf', neg)."""
    if s in ("NaN",):
        return ("nan", False)
    if s in ("Infinity", "+Infinity", "-Infinity"):
        return ("inf", s.startswith("-"))
    m = DEC.match(s)
    if not m:
        raise Err("BAD_NUMBER", s)
    if m.group(4) is not None and abs(int(m.group(4))) > 1000:
        raise Err("BAD_NUMBER", s)
    neg = m.group(1) == "-"
    digits = m.group(2) + (m.group(3) or "")
    point = len(m.group(2)) + int(m.group(4) or 0)
    if point <= 0:
        digits = "0" * (1 - point) + digits
        point = 1
    if point > len(digits):
        digits += "0" * (point - len(digits))
    ip, fp = digits[:point].lstrip("0") or "0", digits[point:]
    return (neg, ip, fp)


def round_half_expand(ip, fp, maxf):
    if len(fp) <= maxf:
        return ip, fp
    keep, rest = fp[:maxf], fp[maxf:]
    if rest[0] >= "5":
        n = int(ip + keep) + 1
        s = str(n).rjust(len(ip + keep), "0")
        return (s[:len(s) - maxf] if maxf else s).lstrip("0") or "0", s[len(s) - maxf:] if maxf else ""
    return ip, keep


# ---------------------------------------------------------------- plural

def operands(s):
    p = parse_decimal(s)
    if p[0] in ("nan", "inf"):
        raise Err("BAD_NUMBER", s)
    neg, ip, fp = p
    t = fp.rstrip("0")
    return {"n": (ip, fp), "i": ip, "v": len(fp), "w": len(t), "f": fp or "0", "t": t or "0", "c": "0", "e": "0"}


def big_mod(digits, m):
    r = 0
    for ch in digits:
        r = (r * 10 + int(ch)) % m
    return r


def relation_value(ops, operand, mod):
    """Returns (is_integer, int_value_or_None) for the operand after modulus."""
    if operand == "n":
        ip, fp = ops["n"]
        is_int = fp.strip("0") == ""
        val = big_mod(ip, mod) if mod else (int(ip) if len(ip) <= 18 else None)
        return is_int, val
    v = ops[operand]
    if isinstance(v, int):
        return True, (v % mod if mod else v)
    return True, (big_mod(v, mod) if mod else (int(v) if len(v) <= 18 else None))


def rule_matches(groups, ops):
    for rels in groups:
        ok = True
        for operand, mod, neg, ranges in rels:
            is_int, val = relation_value(ops, operand, mod)
            inside = is_int and val is not None and any(lo <= val <= hi for lo, hi in ranges)
            if inside == bool(neg):
                ok = False
                break
        if ok:
            return True
    return False


def plural(tag, value, ordinal=False):
    table = CLDR["ordinals" if ordinal else "plurals"]
    loc = rules_locale(tag, table) or "root"
    ops = operands(value)
    for cat, groups in table.get(loc, []):
        if rule_matches(groups, ops):
            return cat
    return "other"


# ---------------------------------------------------------------- numbers

def split_pattern(pat):
    """-> (prefix, number_part, suffix) of one subpattern, affixes raw (quotes kept)."""
    i, n = 0, len(pat)
    start = end = None
    q = False
    for i, ch in enumerate(pat):
        if ch == "'":
            q = not q
        elif not q and ch in "#0,.":
            if start is None:
                start = i
            end = i + 1
    return pat[:start], pat[start:end], pat[end:]


def grouping(numpart):
    ip = numpart.split(".")[0]
    groups = ip.split(",")
    primary = len(groups[-1]) if len(groups) > 1 else 0
    secondary = len(groups[-2]) if len(groups) > 2 else primary
    return primary, secondary


def expand_affix(raw, d, symbol=None):
    out, q, i = [], False, 0
    while i < len(raw):
        ch = raw[i]
        if ch == "'":
            if i + 1 < len(raw) and raw[i + 1] == "'":
                out.append("'")
                i += 2
                continue
            q = not q
        elif q:
            out.append(ch)
        elif ch == "¤":
            out.append(symbol)
        elif ch == "%":
            out.append(d["percent"])
        elif ch == "-":
            out.append(d["minus"])
        else:
            out.append(ch)
        i += 1
    return "".join(out)


def group_int(ip, primary, secondary, min_grouping, sep):
    if primary == 0 or len(ip) < primary + min_grouping:
        return ip
    head, tail = ip[:-primary], ip[-primary:]
    parts = [tail]
    while len(head) > secondary:
        parts.insert(0, head[-secondary:])
        head = head[:-secondary]
    if head:
        parts.insert(0, head)
    return sep.join(parts)


def format_number(tag, value, style="decimal", currency=None, minf=None, maxf=None):
    d = DATA[data_locale(tag)]
    if style == "currency":
        if currency is None or not re.fullmatch(r"[A-Za-z]{3}", currency):
            raise Err("BAD_OPTION", "currency")
        currency = currency.upper()
        digits = CLDR["currencyDigits"].get(currency, 2)
        dmin, dmax = digits, digits
        pattern = d["currencyPattern"]
    elif style == "percent":
        dmin, dmax, pattern = 0, 0, d["percentPattern"]
    elif style == "decimal":
        dmin, dmax, pattern = 0, 3, d["decimalPattern"]
    else:
        raise Err("BAD_OPTION", "style")
    if minf is not None and maxf is None:
        maxf = max(minf, dmax)
    if maxf is not None and minf is None:
        minf = min(dmin, maxf)
    minf = dmin if minf is None else minf
    maxf = dmax if maxf is None else maxf
    if not (0 <= minf <= 20 and 0 <= maxf <= 20 and minf <= maxf):
        raise Err("BAD_OPTION", "fractionDigits")
    pos, _, negp = pattern.partition(";")
    prefix, numpart, suffix = split_pattern(pos)
    p = parse_decimal(value)
    if p[0] in ("nan", "inf"):
        neg = p[1]
        body = d["nan"] if p[0] == "nan" else d["infinity"]
    else:
        neg, ip, fp = p
        if style == "percent":
            fp = fp + "00"
            ip, fp = (ip + fp[:2]).lstrip("0") or "0", fp[2:]
        ip, fp = round_half_expand(ip, fp, maxf)
        fp = fp.ljust(minf, "0")
        while len(fp) > minf and fp.endswith("0"):
            fp = fp[:-1]
        primary, secondary = grouping(numpart)
        body = group_int(ip, primary, secondary, d["minGrouping"], d["group"])
        if fp:
            body += d["decimal"] + fp
    if neg and negp:
        nprefix, _, nsuffix = split_pattern(negp)
    elif neg:
        nprefix, nsuffix = "-" + prefix, suffix
    else:
        nprefix, nsuffix = prefix, suffix
    sym = None
    if style == "currency":
        entry = d["currencySymbols"].get(currency, [currency, 0, 0])
        sym = entry[0]
        pre = expand_affix(nprefix, d, sym)
        suf = expand_affix(nsuffix, d, sym)
        if nprefix.endswith("¤") and not entry[2] and body[:1].isdigit():
            pre += "\xa0"
        if nsuffix.startswith("¤") and not entry[1] and body[-1:].isdigit():
            suf = "\xa0" + suf
        return pre + body + suf
    return expand_affix(nprefix, d) + body + expand_affix(nsuffix, d)


# ---------------------------------------------------------------- dates

DATE = re.compile(r"^(\d{4})-(\d{2})-(\d{2})$")


def parse_date(s):
    m = DATE.match(s)
    if not m:
        raise Err("BAD_DATE", s)
    y, mo, da = int(m.group(1)), int(m.group(2)), int(m.group(3))
    leap = y % 4 == 0 and (y % 100 != 0 or y % 400 == 0)
    dim = [31, 29 if leap else 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    if y < 1 or not 1 <= mo <= 12 or not 1 <= da <= dim[mo - 1]:
        raise Err("BAD_DATE", s)
    return y, mo, da


def weekday(y, m, d):
    """0 = Sunday. Computed from days since 0001-01-01 (a Monday)."""
    days = 0
    for yy in range(1, y):
        days += 366 if (yy % 4 == 0 and (yy % 100 != 0 or yy % 400 == 0)) else 365
    leap = y % 4 == 0 and (y % 100 != 0 or y % 400 == 0)
    dim = [31, 29 if leap else 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    days += sum(dim[:m - 1]) + d - 1
    return (days + 1) % 7


def format_date(tag, date, style="medium"):
    if style not in ("full", "long", "medium", "short"):
        raise Err("BAD_OPTION", "style")
    y, m, dd = parse_date(date)
    d = DATA[data_locale(tag)]
    pat = d["dateFormats"][style]
    out, i = [], 0
    while i < len(pat):
        ch = pat[i]
        if ch == "'":
            j = i + 1
            while j < len(pat):
                if pat[j] == "'":
                    if j + 1 < len(pat) and pat[j + 1] == "'":
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
        if ("a" <= ch <= "z") or ("A" <= ch <= "Z"):
            j = i
            while j < len(pat) and pat[j] == ch:
                j += 1
            n = j - i
            if ch == "G":
                out.append(d["era"])
            elif ch == "y":
                out.append(str(y % 100).zfill(2) if n == 2 else str(y).zfill(n))
            elif ch in "ML":
                out.append(str(m).zfill(n) if n <= 2 else d["months"][0 if n == 3 else 1][m - 1])
            elif ch == "d":
                out.append(str(dd).zfill(n))
            elif ch == "E":
                out.append(d["days"][1 if n == 4 else 0][weekday(y, m, dd)])
            else:
                raise Err("INTERNAL", ch)
            i = j
            continue
        out.append(ch)
        i += 1
    return "".join(out)


# ---------------------------------------------------------------- messages

MAX_DEPTH = 16
NAME = re.compile(r"^[A-Za-z0-9_]+$")


class Parser:
    """ICU MessageFormat subset parser producing a small tree."""

    def __init__(self, s):
        self.s = s
        self.i = 0

    def error(self, code="SYNTAX"):
        raise Err(code, str(self.i))

    def message(self, depth, in_plural, top):
        if depth > MAX_DEPTH:
            raise Err("TOO_DEEP")
        parts, buf = [], []
        s = self.s
        while self.i < len(s):
            ch = s[self.i]
            if ch == "'":
                nxt = s[self.i + 1] if self.i + 1 < len(s) else ""
                if nxt == "'":
                    buf.append("'")
                    self.i += 2
                elif nxt in ("{", "}", "|") or (nxt == "#" and in_plural):
                    self.i += 1
                    while True:
                        if self.i >= len(s):
                            self.i = len(s)
                            break
                        if s[self.i] == "'":
                            if self.i + 1 < len(s) and s[self.i + 1] == "'":
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
            elif ch == "{":
                if buf:
                    parts.append(("text", "".join(buf)))
                    buf = []
                self.i += 1
                parts.append(self.argument(depth, in_plural))
            elif ch == "}":
                if top:
                    self.error()
                break
            elif ch == "#" and in_plural:
                if buf:
                    parts.append(("text", "".join(buf)))
                    buf = []
                parts.append(("pound",))
                self.i += 1
            else:
                buf.append(ch)
                self.i += 1
        if buf:
            parts.append(("text", "".join(buf)))
        return parts

    def ws(self):
        while self.i < len(self.s) and self.s[self.i] in " \t\r\n":
            self.i += 1

    def word(self):
        self.ws()
        j = self.i
        while self.i < len(self.s) and self.s[self.i] not in " \t\r\n,{}":
            self.i += 1
        return self.s[j:self.i]

    def expect(self, ch):
        self.ws()
        if self.i >= len(self.s) or self.s[self.i] != ch:
            self.error()
        self.i += 1

    def argument(self, depth, in_plural):
        name = self.word()
        if not NAME.match(name):
            self.error()
        self.ws()
        if self.i >= len(self.s):
            self.error()
        if self.s[self.i] == "}":
            self.i += 1
            return ("simple", name)
        self.expect(",")
        kind = self.word()
        if kind in ("number", "date"):
            self.ws()
            style = None
            if self.i < len(self.s) and self.s[self.i] == ",":
                self.i += 1
                style = self.word()
                allowed = ("integer", "percent") if kind == "number" else ("short", "medium", "long", "full")
                if style not in allowed:
                    self.error()
            self.expect("}")
            return (kind, name, style)
        if kind not in ("plural", "selectordinal", "select"):
            self.error()
        self.expect(",")
        offset = 0
        cases = {}
        while True:
            self.ws()
            if self.i >= len(self.s):
                self.error()
            if self.s[self.i] == "}":
                self.i += 1
                break
            key = self.word()
            if key.startswith("offset:") and kind == "plural" and not cases and offset == 0:
                rest = key[len("offset:"):]
                if rest == "":
                    rest = self.word()
                if not rest.isdigit():
                    self.error()
                offset = int(rest)
                continue
            if key == "" or key in cases:
                self.error()
            if kind != "select" and key.startswith("="):
                if not re.fullmatch(r"=\d+(\.\d+)?", key):
                    self.error()
            elif kind != "select" and key not in ("zero", "one", "two", "few", "many", "other"):
                self.error()
            elif kind == "select" and not NAME.match(key):
                self.error()
            self.expect("{")
            body = self.message(depth + 1, True if kind != "select" else in_plural, False)
            if self.i >= len(self.s) or self.s[self.i] != "}":
                self.error()
            self.i += 1
            cases[key] = body
        if "other" not in cases:
            raise Err("MISSING_OTHER")
        return (kind, name, offset, cases)


def as_number(v):
    if isinstance(v, dict) and "n" in v:
        return v["n"]
    raise Err("BAD_ARGUMENT")


def dec_sub(value, offset):
    neg, ip, fp = parse_decimal(value)
    if offset == 0:
        return value
    scale = len(fp)
    if len(ip + fp) > 36:
        raise Err("BAD_ARGUMENT")
    n = int(ip + fp) * (-1 if neg else 1) - offset * 10 ** scale
    sign = "-" if n < 0 else ""
    s = str(abs(n)).rjust(scale + 1, "0")
    return sign + (s[:len(s) - scale] + ("." + s[len(s) - scale:] if scale else ""))


def dec_eq(a, b):
    pa, pb = parse_decimal(a), parse_decimal(b)
    za = pa[1] == "0" and pa[2].strip("0") == ""
    zb = pb[1] == "0" and pb[2].strip("0") == ""
    if za and zb:
        return True
    return pa[0] == pb[0] and pa[1] == pb[1] and pa[2].rstrip("0") == pb[2].rstrip("0")


def rounded_for_plural(value):
    neg, ip, fp = parse_decimal(value)
    ip, fp = round_half_expand(ip, fp, 3)
    fp = fp.rstrip("0")
    return ip + ("." + fp if fp else "")


def fmt_arg(tag, name, value, **kw):
    try:
        return format_number(tag, value, **kw)
    except Err:
        raise Err("BAD_ARGUMENT", name)


def render(tag, parts, args, pound):
    out = []
    for part in parts:
        kind = part[0]
        if kind == "text":
            out.append(part[1])
        elif kind == "pound":
            out.append(format_number(tag, pound) if pound is not None else "#")
        elif kind == "simple":
            if part[1] not in args:
                raise Err("MISSING_ARGUMENT", part[1])
            v = args[part[1]]
            out.append(v["s"] if "s" in v else fmt_arg(tag, part[1], v["n"]))
        elif kind == "number":
            if part[1] not in args:
                raise Err("MISSING_ARGUMENT", part[1])
            n = as_number(args[part[1]])
            if part[2] == "integer":
                out.append(fmt_arg(tag, part[1], n, maxf=0))
            elif part[2] == "percent":
                out.append(fmt_arg(tag, part[1], n, style="percent"))
            else:
                out.append(fmt_arg(tag, part[1], n))
        elif kind == "date":
            if part[1] not in args:
                raise Err("MISSING_ARGUMENT", part[1])
            v = args[part[1]]
            if "s" not in v:
                raise Err("BAD_ARGUMENT", part[1])
            try:
                out.append(format_date(tag, v["s"], part[2] or "medium"))
            except Err:
                raise Err("BAD_ARGUMENT", part[1])
        else:
            _, name, offset, cases = part
            if name not in args:
                raise Err("MISSING_ARGUMENT", name)
            v = args[name]
            if kind == "select":
                key = v["s"] if "s" in v else v["n"]
                out.append(render(tag, cases.get(key, cases["other"]), args, pound))
                continue
            n = as_number(v)
            try:
                parse_decimal(n)
            except Err:
                raise Err("BAD_ARGUMENT", name)
            if parse_decimal(n)[0] in ("nan", "inf"):
                raise Err("BAD_ARGUMENT", name)
            exact = None
            for key in cases:
                if key.startswith("=") and dec_eq(key[1:], n):
                    exact = key
                    break
            shown = dec_sub(n, offset)
            if exact is not None:
                chosen = cases[exact]
            else:
                cat = plural(tag, rounded_for_plural(shown).lstrip("-"), ordinal=(kind == "selectordinal"))
                chosen = cases.get(cat, cases["other"])
            out.append(render(tag, chosen, args, shown))
    return "".join(out)


def format_message(tag, pattern, args):
    p = Parser(pattern)
    parts = p.message(0, False, True)
    return render(tag, parts, args, None)


# ---------------------------------------------------------------- catalogs

def parse_catalog(text):
    pairs = []
    def no_constant(name):
        raise ValueError(name)

    try:
        obj = json.loads(text, object_pairs_hook=lambda kv: ("obj", kv), parse_constant=no_constant)
    except (ValueError, RecursionError):
        raise Err("BAD_JSON")
    if any(0xD800 <= ord(ch) <= 0xDFFF for ch in json.dumps(obj, ensure_ascii=False)):
        raise Err("BAD_JSON")
    if not (isinstance(obj, tuple) and obj[0] == "obj"):
        raise Err("NOT_OBJECT")
    seen = set()
    for k, v in obj[1]:
        if not isinstance(v, str):
            raise Err("NON_STRING_VALUE", k)
        if k in seen:
            raise Err("DUPLICATE_KEY", k)
        seen.add(k)
        pairs.append((k, v))
    return dict(pairs)


def resolve(tag, catalog_text, code, message_id, args):
    cat = parse_catalog(catalog_text)
    pattern = cat.get(message_id)
    if pattern is None:
        return {"code": code, "messageId": message_id, "text": "!!%s!!" % message_id, "fallback": True}
    try:
        text = format_message(tag, pattern, args)
        return {"code": code, "messageId": message_id, "text": text, "fallback": False}
    except Err:
        return {"code": code, "messageId": message_id, "text": "!!%s!!" % message_id, "fallback": True}


# ---------------------------------------------------------------- cases

CASES = []
COUNT = {}


def run(kind, inp):
    try:
        if kind == "parse":
            return {"ok": tag_string(parse_tag(inp["tag"]))}
        if kind == "negotiate":
            return {"ok": negotiate(inp["requested"], inp["available"], inp["default"])}
        if kind == "plural":
            return {"ok": plural(inp["locale"], inp["value"], inp.get("ordinal", False))}
        if kind == "number":
            return {"ok": format_number(inp["locale"], inp["value"], inp.get("style", "decimal"), inp.get("currency"),
                                        inp.get("minFraction"), inp.get("maxFraction"))}
        if kind == "date":
            return {"ok": format_date(inp["locale"], inp["date"], inp.get("style", "medium"))}
        if kind == "message":
            return {"ok": format_message(inp["locale"], inp["pattern"], inp["args"])}
        if kind == "catalog":
            return {"ok": parse_catalog(inp["json"])}
        if kind == "resolve":
            return {"ok": resolve(inp["locale"], inp["catalog"], inp["code"], inp["messageId"], inp["args"])}
        if kind == "dataLocale":
            return {"ok": data_locale(inp["tag"])}
    except Err as e:
        return {"error": e.code}
    raise ValueError(kind)


def case(kind, inp, want=None, note=""):
    got = run(kind, inp)
    if want is not None:
        w = want if isinstance(want, dict) and set(want) <= {"ok", "error"} and want else {"ok": want}
        if w != got:
            raise SystemExit("model disagrees with hand-written expectation in %s %r: got %r want %r" % (kind, inp, got, w))
    COUNT[kind] = COUNT.get(kind, 0) + 1
    CASES.append({"id": "%s-%03d" % (kind, COUNT[kind]), "kind": kind, "note": note, "input": inp, "expected": got})


E = lambda code: {"error": code}  # noqa: E731

# BCP 47 parsing
for tag, want in [("id", "id"), ("en-US", "en-US"), ("EN_us", "en-US"), ("zh-hant-tw", "zh-Hant-TW"),
                  ("sr-Latn-RS", "sr-Latn-RS"), ("es-419", "es-419"), ("de-CH-1996", "de-CH-1996"),
                  ("sl-rozaj-biske", "sl-rozaj-biske"), ("en-US-u-ca-gregory", "en-US-u-ca-gregory"),
                  ("en-u-nu-latn-a-bbb", "en-a-bbb-u-nu-latn"), ("de-x-private", "de-x-private"),
                  ("fil", "fil"), ("  id-ID  ", "id-ID"), ("haw-US", "haw-US"), ("zh-Hans", "zh-Hans"),
                  ("", E("EMPTY")), ("   ", E("EMPTY")), ("e", E("INVALID_SUBTAG")), ("abcd", E("INVALID_SUBTAG")),
                  ("en--US", E("INVALID_SUBTAG")), ("en-US-", E("INVALID_SUBTAG")), ("en-ÜS", E("INVALID_SUBTAG")),
                  ("de-1996-1996", E("DUPLICATE_VARIANT")), ("en-u-ca-u-nu", E("DUPLICATE_EXTENSION")),
                  ("en-u", E("INVALID_SUBTAG")), ("en-x", E("INVALID_SUBTAG")), ("x-private", E("INVALID_SUBTAG")),
                  ("en-toolongsubtag", E("INVALID_SUBTAG")), ("123", E("INVALID_SUBTAG")), ("en-US-US", E("INVALID_SUBTAG"))]:
    case("parse", {"tag": tag}, want)

# Negotiation (RFC 4647 lookup)
for req, av, dflt, want in [
        (["id-ID", "en"], ["en", "id"], "en", "id"),
        (["en-GB"], ["en", "id"], "id", "en"),
        (["zh-Hant-TW"], ["zh", "zh-Hant"], "en", "zh-Hant"),
        (["zh-Hant-TW"], ["zh", "zh-TW"], "en", "zh"),
        (["zh-Hant-TW"], ["zh-Hant-TW", "zh-Hant"], "en", "zh-Hant-TW"),
        (["de-CH-1996"], ["de-CH", "de"], "en", "de-CH"),
        (["fr-FR"], ["en", "id"], "en", "en"),
        (["fr", "ja-JP"], ["id", "ja"], "en", "ja"),
        (["EN-us"], ["en-US"], "id", "en-US"),
        (["en-US"], ["EN-us"], "id", "EN-us"),
        (["en-US-u-ca-buddhist"], ["en-US"], "id", "en-US"),
        (["not a tag", "id"], ["id"], "en", "id"),
        ([], ["id"], "en", "en"),
        (["id"], [], "en", "en"),
        (["pt-BR"], ["pt-PT", "pt"], "en", "pt"),
        (["sr-Latn"], ["sr", "sr-Cyrl"], "en", "sr")]:
    case("negotiate", {"requested": req, "available": av, "default": dflt}, want)

# Data locale resolution
for tag, want in [("id-ID", "id"), ("en-US", "en"), ("en-GB", "en-GB"), ("zh-TW", "zh-Hant"), ("zh-HK", "zh-Hant"),
                  ("zh-CN", "zh"), ("zh-Hans-SG", "zh"), ("pt-BR", "pt"), ("pt-PT", "pt-PT"), ("xx", "en"),
                  ("not a tag", "en"), ("jv-Latn-ID", "jv"), ("ar-EG-u-nu-arab", "ar")]:
    case("dataLocale", {"tag": tag}, want)

# Plural rules (cardinal)
for loc, val, want in [("en", "1", "one"), ("en", "0", "other"), ("en", "2", "other"), ("en", "1.0", "other"),
                       ("en", "1.5", "other"), ("id", "1", "other"), ("ja", "1", "other"), ("fr", "0", "one"),
                       ("fr", "1.5", "one"), ("fr", "2", "other"), ("fr", "1000000", "many"), ("fr", "2000000", "many"),
                       ("fr", "1000001", "other"), ("pt", "0", "one"), ("pt", "1.5", "one"), ("pt-PT", "0", "other"),
                       ("pt-PT", "1", "one"), ("es", "1", "one"), ("es", "1000000", "many"), ("ru", "1", "one"),
                       ("ru", "21", "one"), ("ru", "11", "many"), ("ru", "2", "few"), ("ru", "22", "few"), ("ru", "12", "many"),
                       ("ru", "5", "many"), ("ru", "1.5", "other"), ("pl", "1", "one"), ("pl", "21", "many"),
                       ("pl", "22", "few"), ("pl", "5", "many"), ("pl", "0.5", "other"), ("uk", "101", "one"),
                       ("hr", "21", "one"), ("hr", "5", "other"), ("hr", "0.1", "one"), ("ar", "0", "zero"), ("ar", "1", "one"),
                       ("ar", "2", "two"), ("ar", "3", "few"), ("ar", "103", "few"), ("ar", "11", "many"), ("ar", "100", "other"),
                       ("ar", "1.5", "other"), ("cy", "0", "zero"), ("cy", "3", "few"), ("cy", "6", "many"), ("cy", "7", "other"),
                       ("he", "2", "two"), ("lt", "11", "other"), ("lt", "0.5", "many"), ("en-US", "1", "one"),
                       ("xx", "1", "other"), ("ru", "123456789012345678901", "one"), ("en", "-1", "one"), ("hi", "0", "one"),
                       ("si", "0.1", "one"), ("lv", "0.1", "one"), ("lv", "0.10", "other"), ("mk", "1.1", "one"), ("br", "1000000", "many"),
                       ("gv", "20", "few"), ("ksh", "0", "zero"), ("shi", "1", "one"), ("dsb", "103", "few"), ("mt", "11", "many"), ("is", "21", "one"), ("lv", "0", "zero"), ("ro", "19", "few"), ("ro", "20", "other")]:
    case("plural", {"locale": loc, "value": val}, want)

# Plural rules (ordinal)
for loc, val, want in [("en", "1", "one"), ("en", "2", "two"), ("en", "3", "few"), ("en", "4", "other"), ("en", "11", "other"),
                       ("en", "12", "other"), ("en", "13", "other"), ("en", "21", "one"), ("en", "103", "few"),
                       ("en", "111", "other"), ("fr", "1", "one"), ("fr", "2", "other"), ("it", "8", "many"),
                       ("it", "11", "many"), ("it", "80", "many"), ("it", "9", "other"), ("sv", "22", "one"),
                       ("sv", "12", "other"), ("hu", "5", "one"), ("id", "1", "other"), ("cy", "0", "zero"), ("cy", "3", "few")]:
    case("plural", {"locale": loc, "value": val, "ordinal": True}, want)

# Decimal numbers
for loc, val, want, extra in [
        ("en", "1234567.891", "1,234,567.891", {}), ("id", "1234567.891", "1.234.567,891", {}),
        ("de", "1234567.891", "1.234.567,891", {}), ("fr", "1234567.891", "1 234 567,891", {}),
        ("es", "1234", "1234", {}), ("es", "12345", "12.345", {}), ("pt-PT", "1234", "1234", {}),
        ("pt-PT", "12345", "12\xa0345", {}), ("pl", "1234", "1234", {}), ("hi", "1234567.5", "12,34,567.5", {}),
        ("en-IN", "123456789", "12,34,56,789", {}), ("en", "0.12345", "0.123", {}), ("en", "0.0005", "0.001", {}),
        ("en", "0.0004", "0", {}), ("en", "-0.0004", "-0", {}), ("en", "2.5", "2.5", {}), ("en", "-1234.5", "-1,234.5", {}),
        ("ar", "-1234.5", "‎-1,234.5", {}), ("en", "1e21", "1,000,000,000,000,000,000,000", {}),
        ("en", "1.5e-3", "0.002", {}), ("en", "12345678901234567890123", "12,345,678,901,234,567,890,123", {}),
        ("en", "1234.5", "1,234.50", {"minFraction": 2}), ("en", "1.23456", "1.2346", {"maxFraction": 4}),
        ("en", "1.5", "2", {"maxFraction": 0}), ("en", "2.5", "3", {"maxFraction": 0}), ("en", "-2.5", "-3", {"maxFraction": 0}),
        ("id", "0.1", "0,10", {"minFraction": 2, "maxFraction": 2}), ("en", "1", "x", {"minFraction": 3, "maxFraction": 1}),
        ("en", "NaN", "NaN", {}), ("en", "Infinity", "∞", {}), ("de", "-Infinity", "-∞", {}),
        ("ja", "1234567", "1,234,567", {}), ("th", "1234.5", "1,234.5", {}), ("ru", "1234567", "1\xa0234\xa0567", {}),
        ("sv", "-1234.5", "−1\xa0234,5", {}), ("tr", "1234.5", "1.234,5", {}), ("vi", "1234.5", "1.234,5", {}),
        ("en", "abc", "x", {}), ("en", "1.2.3", "x", {}), ("en", "1e1001", "x", {}), ("en", "1.", "x", {}), ("en", ".5", "x", {}),
        ("en", "1e1000", None, {}), ("en", "1E-3", "0.001", {}), ("en", "1", "x", {"maxFraction": 21})]:
    if want == "x":
        want = E("BAD_NUMBER") if extra == {} else E("BAD_OPTION")
    elif want is None:
        pass
    inp = {"locale": loc, "value": val}
    inp.update(extra)
    case("number", inp, want)

# Percent
for loc, val, want in [("en", "0.256", "26%"), ("fr", "0.256", "26\xa0%"), ("de", "0.5", "50\xa0%"), ("tr", "0.5", "%50"),
                       ("ar", "0.5", "50‎%‎"), ("id", "1.5", "150%"), ("en", "-0.05", "-5%"), ("es", "12.34", "1234\xa0%")]:
    case("number", {"locale": loc, "value": val, "style": "percent"}, want)

# Currency
for loc, val, cur, want in [
        ("en", "1999.99", "USD", "$1,999.99"), ("id", "15000", "IDR", "Rp\xa015.000,00"), ("en", "15000", "IDR", "IDR\xa015,000.00"),
        ("de", "15000", "EUR", "15.000,00\xa0€"), ("fr", "1234.5", "EUR", "1 234,50\xa0€"), ("ja", "15000", "JPY", "￥15,000"),
        ("en", "15000.5", "JPY", "¥15,001"), ("ko", "15000", "KRW", "₩15,000"), ("en", "-5", "USD", "-$5.00"),
        ("id", "-5", "IDR", "-Rp\xa05,00"), ("ar", "1234.5", "SAR", None), ("ar", "-1234.5", "SAR", None),
        ("hi", "1234567", "INR", "₹12,34,567.00"), ("ms", "10", "MYR", "RM\xa010.00"), ("th", "10", "THB", "฿10.00"),
        ("vi", "15000", "VND", "15.000\xa0₫"), ("en", "10", "XYZ", "XYZ\xa010.00"), ("de", "10", "XYZ", "10,00\xa0XYZ"),
        ("en", "10", "usd", "$10.00"), ("en", "10", "US", E("BAD_OPTION")), ("zh", "10", "CNY", "¥10.00"),
        ("zh-Hant", "10", "USD", "US$10.00"), ("en-GB", "10", "GBP", "£10.00"), ("es", "1234.5", "EUR", "1234,50\xa0€"),
        ("pt", "10", "USD", "US$\xa010,00"), ("en", "Infinity", "USD", "$∞"), ("en", "10", "USD", "$10", ),
        ("en", "10.005", "USD", "$10.01")]:
    inp = {"locale": loc, "value": val, "style": "currency", "currency": cur}
    if loc == "en" and val == "10" and cur == "USD" and want == "$10":
        inp["minFraction"] = 0
        inp["maxFraction"] = 0
    case("number", inp, want)
case("number", {"locale": "en", "value": "10", "style": "currency"}, E("BAD_OPTION"))
case("number", {"locale": "en", "value": "10", "style": "scientific"}, E("BAD_OPTION"))

# Dates
for loc, dt, style, want in [
        ("en", "2026-10-02", "full", "Friday, October 2, 2026"), ("en", "2026-10-02", "long", "October 2, 2026"),
        ("en", "2026-10-02", "medium", "Oct 2, 2026"), ("en", "2026-10-02", "short", "10/2/26"),
        ("en-GB", "2026-10-02", "short", "02/10/2026"), ("id", "2026-10-02", "full", "Jumat, 02 Oktober 2026"),
        ("id", "2026-10-02", "long", "2 Oktober 2026"), ("id", "2026-10-02", "short", "02/10/26"),
        ("de", "2026-10-02", "full", None), ("fr", "2026-10-02", "long", "2 octobre 2026"),
        ("es", "2026-10-02", "long", "2 de octubre de 2026"), ("pt", "2026-10-02", "medium", None),
        ("ru", "2026-10-02", "long", None), ("ja", "2026-10-02", "full", "2026年10月2日金曜日"),
        ("zh", "2026-10-02", "long", "2026年10月2日"), ("ko", "2026-10-02", "full", None), ("th", "2026-10-02", "long", None),
        ("ar", "2026-10-02", "long", None), ("hi", "2026-10-02", "medium", None), ("vi", "2026-10-02", "full", None),
        ("jv", "2026-10-02", "long", None), ("su", "2026-10-02", "long", None), ("cy", "2026-10-02", "full", None),
        ("en", "2024-02-29", "medium", "Feb 29, 2024"), ("en", "0001-01-01", "full", "Monday, January 1, 1"),
        ("en", "9999-12-31", "full", "Friday, December 31, 9999"), ("en", "2000-01-01", "short", "1/1/00"),
        ("en", "2023-02-29", "medium", E("BAD_DATE")), ("en", "2026-13-01", "medium", E("BAD_DATE")),
        ("en", "0000-01-01", "medium", E("BAD_DATE")), ("en", "2026-1-1", "medium", E("BAD_DATE")),
        ("en", "2026-10-02", "huge", E("BAD_OPTION")), ("uk", "2026-10-02", "medium", None), ("pl", "2026-10-02", "full", None)]:
    case("date", {"locale": loc, "date": dt, "style": style}, want)

# Messages
S = lambda v: {"s": v}  # noqa: E731
N = lambda v: {"n": v}  # noqa: E731
for loc, pat, args, want in [
        ("en", "Hello, {name}!", {"name": S("Lombok")}, "Hello, Lombok!"),
        ("id", "Total {n}", {"n": N("1234567.5")}, "Total 1.234.567,5"),
        ("en", "{n, plural, one{# file} other{# files}}", {"n": N("1")}, "1 file"),
        ("en", "{n, plural, one{# file} other{# files}}", {"n": N("1234")}, "1,234 files"),
        ("en", "{n, plural, one{# file} other{# files}}", {"n": N("1.0")}, "1 file"),
        ("en", "{n, plural, one{# file} other{# files}}", {"n": N("1.5")}, "1.5 files"),
        ("en", "{n, plural, =0{no files} one{# file} other{# files}}", {"n": N("0")}, "no files"),
        ("en", "{n, plural, offset:1 =0{nobody} =1{{who}} one{{who} and # other} other{{who} and # others}}",
         {"n": N("2"), "who": S("Ana")}, "Ana and 1 other"),
        ("en", "{n, plural, offset:1 =0{nobody} =1{{who}} one{{who} and # other} other{{who} and # others}}",
         {"n": N("1"), "who": S("Ana")}, "Ana"),
        ("en", "{n, plural, offset:1 =0{nobody} =1{{who}} one{{who} and # other} other{{who} and # others}}",
         {"n": N("5"), "who": S("Ana")}, "Ana and 4 others"),
        ("ru", "{n, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}", {"n": N("22")}, "22 файла"),
        ("ar", "{n, plural, zero{لا شيء} one{واحد} two{اثنان} few{# قليل} many{# كثير} other{#}}", {"n": N("11")}, "11 كثير"),
        ("en", "{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}", {"n": N("23")}, "23rd"),
        ("en", "{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}", {"n": N("111")}, "111th"),
        ("id", "{n, selectordinal, other{ke-#}}", {"n": N("3")}, "ke-3"),
        ("id", "{g, select, male{Dia (L)} female{Dia (P)} other{Dia}}", {"g": S("female")}, "Dia (P)"),
        ("id", "{g, select, male{Dia (L)} female{Dia (P)} other{Dia}}", {"g": S("x")}, "Dia"),
        ("en", "{g, select, female{{n, plural, one{She has # cat} other{She has # cats}}} other{{n, plural, one{They have # cat} other{They have # cats}}}}",
         {"g": S("female"), "n": N("3")}, "She has 3 cats"),
        ("en", "{n, plural, other{{g, select, other{# items}}}}", {"n": N("4"), "g": S("x")}, "4 items"),
        ("en", "It''s {n, number, percent}", {"n": N("0.25")}, "It's 25%"),
        ("en", "'{literal}' and {x}", {"x": S("y")}, "{literal} and y"),
        ("en", "Don't touch", {}, "Don't touch"),
        ("en", "{n, plural, other{'#' is #}}", {"n": N("5")}, "# is 5"),
        ("en", "# outside", {}, "# outside"),
        ("de", "{n, number, integer} Stück", {"n": N("1234.6")}, "1.235 Stück"),
        ("en", "Due {d, date, long}", {"d": S("2026-10-02")}, "Due October 2, 2026"),
        ("id", "Jatuh tempo {d, date}", {"d": S("2026-10-02")}, "Jatuh tempo 2 Okt 2026"),
        ("en", "{ name }", {"name": S("x")}, "x"),
        ("en", "Hi {name}", {}, E("MISSING_ARGUMENT")),
        ("en", "Hi {name", {"name": S("x")}, E("SYNTAX")),
        ("en", "Hi name}", {}, E("SYNTAX")),
        ("en", "{n, plural, one{x}}", {"n": N("1")}, E("MISSING_OTHER")),
        ("en", "{n, select, a{x}}", {"n": S("a")}, E("MISSING_OTHER")),
        ("en", "{n, plural, one{x} one{y} other{z}}", {"n": N("1")}, E("SYNTAX")),
        ("en", "{n, plural, single{x} other{y}}", {"n": N("1")}, E("SYNTAX")),
        ("en", "{n, spellout}", {"n": N("1")}, E("SYNTAX")),
        ("en", "{n, plural, other{#}}", {"n": S("many")}, E("BAD_ARGUMENT")),
        ("en", "{d, date, short}", {"d": S("2026-02-30")}, E("BAD_ARGUMENT")),
        ("en", "{bad-name}", {}, E("SYNTAX")),
        ("en", "{n, number, currency}", {"n": N("1")}, E("SYNTAX")),
        ("en", "{n,plural,offset:2 other{#}}", {"n": N("5")}, "3"),
        ("en", "{n, plural, =1.5{exact} other{#}}", {"n": N("1.5")}, "exact"),
        ("en", "{n, plural, offset:1 other{#}}", {"n": N("1" + "0" * 40)}, E("BAD_ARGUMENT")),
        ("en", "{n, plural, other{#}}", {"n": N("NaN")}, E("BAD_ARGUMENT")),
        ("en", "{n}", {"n": N("bad")}, E("BAD_ARGUMENT")),
        ("en", "{n, number}", {"n": S("x")}, E("BAD_ARGUMENT")),
        ("en", "{d, date}", {"d": N("1")}, E("BAD_ARGUMENT")),
        ("en", "{n, plural, =0{none} other{#}}", {"n": N("-0.00")}, "none"),
        ("en", "{n, select, 5{five} other{x}}", {"n": N("5")}, "five")]:
    case("message", {"locale": loc, "pattern": pat, "args": args}, want)
deep = "x"
for _ in range(17):
    deep = "{v, select, other{%s}}" % deep
case("message", {"locale": "en", "pattern": deep, "args": {"v": S("a")}}, E("TOO_DEEP"))
deep16 = "x"
for _ in range(16):
    deep16 = "{v, select, other{%s}}" % deep16
case("message", {"locale": "en", "pattern": deep16, "args": {"v": S("a")}}, "x")

# Catalogs
for text, want in [('{"a": "Halo", "b.c": "Hai {name}"}', {"a": "Halo", "b.c": "Hai {name}"}), ("{}", {}),
                   (' { "x" : "\\u0041\\n" } ', {"x": "A\n"}), ('["a"]', E("NOT_OBJECT")), ('"a"', E("NOT_OBJECT")),
                   ('{"a": 1}', E("NON_STRING_VALUE")), ('{"a": {"b": "c"}}', E("NON_STRING_VALUE")),
                   ('{"a": "x", "a": "y"}', E("DUPLICATE_KEY")), ('{"a": "x",}', E("BAD_JSON")), ("{", E("BAD_JSON")),
                   ('{"a": "x"} extra', E("BAD_JSON")), ("", E("BAD_JSON")), ('{"a": "\\ud800"}', E("BAD_JSON")),
                   ('{"a": NaN}', E("BAD_JSON")), ('{"a": [1, 2.5e3, true, null]}', E("NON_STRING_VALUE")),
                   ('{"a": "\\ud83d\\ude00"}', {"a": "\U0001F600"}), ('{"a": "x\ty"}', E("BAD_JSON")), ('{"a": 01}', E("BAD_JSON"))]:
    case("catalog", {"json": text}, want)

CAT = '{"validator.too_short": "Minimal {min} karakter", "files": "{n, plural, other{# berkas}}", "broken": "{x"}'
for loc, code, mid, args, want in [
        ("id", "TOO_SHORT", "validator.too_short", {"min": N("8")},
         {"code": "TOO_SHORT", "messageId": "validator.too_short", "text": "Minimal 8 karakter", "fallback": False}),
        ("id", "FILES", "files", {"n": N("1200")}, {"code": "FILES", "messageId": "files", "text": "1.200 berkas", "fallback": False}),
        ("id", "X", "missing.id", {}, {"code": "X", "messageId": "missing.id", "text": "!!missing.id!!", "fallback": True}),
        ("id", "B", "broken", {}, {"code": "B", "messageId": "broken", "text": "!!broken!!", "fallback": True}),
        ("id", "TOO_SHORT", "validator.too_short", {}, {"code": "TOO_SHORT", "messageId": "validator.too_short",
                                                        "text": "!!validator.too_short!!", "fallback": True})]:
    case("resolve", {"locale": loc, "catalog": CAT, "code": code, "messageId": mid, "args": args}, want)

doc = {
    "name": "lomboklocale-vectors",
    "version": 1,
    "spec": "docs/SPEC_LombokLocale_v0.2.0.md",
    "cldr": CLDR["cldr"],
    "cases": CASES,
}
out = ROOT / "vectors" / "lomboklocale-vectors-v1.json"
out.write_text(json.dumps(doc, ensure_ascii=False, indent=1) + "\n", encoding="utf-8")
digest = hashlib.sha256(out.read_bytes()).hexdigest()
(ROOT / "vectors" / "SHA256SUMS").write_text("%s  vectors/lomboklocale-vectors-v1.json\n" % digest, encoding="utf-8")
print(len(CASES), "cases", COUNT, digest)
