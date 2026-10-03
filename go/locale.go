// Package lomboklocale provides BCP 47 tags and negotiation, CLDR 47 plural
// rules, number/percent/currency and date formatting, an ICU MessageFormat
// subset, and JSON message catalogs, with the same results as the Rust,
// TypeScript, Python and PHP ports (docs/SPEC_LombokLocale_v0.2.0.md).
package lomboklocale

import (
	"encoding/json"
	"math"
	"math/big"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"unicode/utf8"
)

// MaxDepth is the maximum nesting of message sub-messages.
const MaxDepth = 16

// MaxExponent is the largest exponent magnitude accepted in decimal strings.
const MaxExponent = 1000

// Error is returned by every function that can fail. Code is one of the SPEC
// error codes (section 8); Detail names the offending input.
type Error struct {
	Code   string
	Detail string
}

func (e *Error) Error() string {
	if e.Detail == "" {
		return e.Code
	}
	return e.Code + ": " + e.Detail
}

func newErr(code, detail string) *Error { return &Error{Code: code, Detail: detail} }

type rel struct {
	Op     string
	Mod    int
	Neg    bool
	Ranges [][2]int
}

type rules []struct {
	Cat    string
	Groups [][]rel
}

type localeData struct {
	CurrencyPattern string                        `json:"currencyPattern"`
	CurrencySymbols map[string][3]json.RawMessage `json:"currencySymbols"`
	DateFormats     map[string]string             `json:"dateFormats"`
	Days            [2][7]string                  `json:"days"`
	Decimal         string                        `json:"decimal"`
	DecimalPattern  string                        `json:"decimalPattern"`
	Era             string                        `json:"era"`
	Group           string                        `json:"group"`
	Infinity        string                        `json:"infinity"`
	MinGrouping     int                           `json:"minGrouping"`
	Minus           string                        `json:"minus"`
	Months          [2][12]string                 `json:"months"`
	NaN             string                        `json:"nan"`
	Percent         string                        `json:"percent"`
	PercentPattern  string                        `json:"percentPattern"`
}

type currencySymbol struct {
	symbol          string
	firstSZ, lastSZ bool
}

var (
	cldrVersion    string
	currencyDigits map[string]int
	locales        map[string]*localeData
	symbols        map[string]map[string]currencySymbol
	plurals        map[string]rules
	ordinals       map[string]rules
)

// CLDRVersion is the CLDR release of the embedded data.
func CLDRVersion() string { return cldrVersion }

func decodeRules(raw map[string][][2]json.RawMessage) map[string]rules {
	out := make(map[string]rules, len(raw))
	for loc, entries := range raw {
		var rs rules
		for _, e := range entries {
			var cat string
			var groups [][][4]json.RawMessage
			must(json.Unmarshal(e[0], &cat))
			must(json.Unmarshal(e[1], &groups))
			var gs [][]rel
			for _, g := range groups {
				var rels []rel
				for _, r := range g {
					var x rel
					var neg int
					must(json.Unmarshal(r[0], &x.Op))
					must(json.Unmarshal(r[1], &x.Mod))
					must(json.Unmarshal(r[2], &neg))
					must(json.Unmarshal(r[3], &x.Ranges))
					x.Neg = neg == 1
					rels = append(rels, x)
				}
				gs = append(gs, rels)
			}
			rs = append(rs, struct {
				Cat    string
				Groups [][]rel
			}{cat, gs})
		}
		out[loc] = rs
	}
	return out
}

func must(err error) {
	if err != nil {
		panic(err)
	}
}

func init() {
	var raw struct {
		Cldr           string                          `json:"cldr"`
		CurrencyDigits map[string]int                  `json:"currencyDigits"`
		Locales        map[string]*localeData          `json:"locales"`
		Plurals        map[string][][2]json.RawMessage `json:"plurals"`
		Ordinals       map[string][][2]json.RawMessage `json:"ordinals"`
	}
	must(json.Unmarshal([]byte(cldrJSON), &raw))
	cldrVersion = raw.Cldr
	currencyDigits = raw.CurrencyDigits
	locales = raw.Locales
	symbols = map[string]map[string]currencySymbol{}
	for loc, d := range locales {
		m := map[string]currencySymbol{}
		for code, v := range d.CurrencySymbols {
			var s currencySymbol
			var a, b int
			must(json.Unmarshal(v[0], &s.symbol))
			must(json.Unmarshal(v[1], &a))
			must(json.Unmarshal(v[2], &b))
			s.firstSZ, s.lastSZ = a == 1, b == 1
			m[code] = s
		}
		symbols[loc] = m
	}
	plurals = decodeRules(raw.Plurals)
	ordinals = decodeRules(raw.Ordinals)
}

// ---------------------------------------------------------------- tags

// LanguageTag is a well-formed BCP 47 tag in canonical case.
type LanguageTag struct {
	Language   string
	Script     string
	Region     string
	Variants   []string
	Extensions [][]string // each: singleton followed by its subtags, sorted by singleton
	PrivateUse []string
}

var (
	subtagRe = regexp.MustCompile(`^[A-Za-z0-9]{1,8}$`)
	alphaRe  = regexp.MustCompile(`^[A-Za-z]+$`)
	digitsRe = regexp.MustCompile(`^[0-9]+$`)
)

// ParseTag parses a BCP 47 tag; "_" is accepted as a separator and outer
// spaces are ignored.
func ParseTag(tag string) (LanguageTag, error) {
	t := strings.TrimSpace(tag)
	if t == "" {
		return LanguageTag{}, newErr("EMPTY", "")
	}
	subs := strings.Split(strings.ReplaceAll(t, "_", "-"), "-")
	for _, s := range subs {
		if !subtagRe.MatchString(s) {
			return LanguageTag{}, newErr("INVALID_SUBTAG", s)
		}
	}
	lang := subs[0]
	if !alphaRe.MatchString(lang) || len(lang) == 1 || len(lang) == 4 {
		return LanguageTag{}, newErr("INVALID_SUBTAG", lang)
	}
	out := LanguageTag{Language: strings.ToLower(lang)}
	at := func(k int) string {
		if k < len(subs) {
			return subs[k]
		}
		return ""
	}
	i := 1
	if len(at(i)) == 4 && alphaRe.MatchString(at(i)) {
		s := strings.ToLower(at(i))
		out.Script = strings.ToUpper(s[:1]) + s[1:]
		i++
	}
	if (len(at(i)) == 2 && alphaRe.MatchString(at(i))) || (len(at(i)) == 3 && digitsRe.MatchString(at(i))) {
		out.Region = strings.ToUpper(at(i))
		i++
	}
	for i < len(subs) && (len(at(i)) >= 5 || (len(at(i)) == 4 && at(i)[0] >= '0' && at(i)[0] <= '9')) {
		v := strings.ToLower(at(i))
		for _, x := range out.Variants {
			if x == v {
				return LanguageTag{}, newErr("DUPLICATE_VARIANT", v)
			}
		}
		out.Variants = append(out.Variants, v)
		i++
	}
	for i < len(subs) && len(at(i)) == 1 && strings.ToLower(at(i)) != "x" {
		single := strings.ToLower(at(i))
		for _, e := range out.Extensions {
			if e[0] == single {
				return LanguageTag{}, newErr("DUPLICATE_EXTENSION", single)
			}
		}
		i++
		ext := []string{single}
		for i < len(subs) && len(at(i)) >= 2 {
			ext = append(ext, strings.ToLower(at(i)))
			i++
		}
		if len(ext) == 1 {
			return LanguageTag{}, newErr("INVALID_SUBTAG", single)
		}
		out.Extensions = append(out.Extensions, ext)
	}
	if i < len(subs) && strings.ToLower(at(i)) == "x" {
		i++
		if i == len(subs) {
			return LanguageTag{}, newErr("INVALID_SUBTAG", "x")
		}
		for ; i < len(subs); i++ {
			out.PrivateUse = append(out.PrivateUse, strings.ToLower(subs[i]))
		}
	}
	if i < len(subs) {
		return LanguageTag{}, newErr("INVALID_SUBTAG", at(i))
	}
	sort.Slice(out.Extensions, func(a, b int) bool {
		return strings.Join(out.Extensions[a], "-") < strings.Join(out.Extensions[b], "-")
	})
	return out, nil
}

// Base returns language, script, region and variants joined with "-".
func (t LanguageTag) Base() string {
	parts := []string{t.Language}
	for _, p := range []string{t.Script, t.Region} {
		if p != "" {
			parts = append(parts, p)
		}
	}
	return strings.Join(append(parts, t.Variants...), "-")
}

// LookupChain returns the RFC 4647 fallback list: the base tag, then shorter prefixes.
func (t LanguageTag) LookupChain() []string {
	parts := strings.Split(t.Base(), "-")
	out := make([]string, 0, len(parts))
	for k := len(parts); k > 0; k-- {
		out = append(out, strings.Join(parts[:k], "-"))
	}
	return out
}

// String returns the canonical form including extensions and private use.
func (t LanguageTag) String() string {
	parts := []string{t.Base()}
	for _, e := range t.Extensions {
		parts = append(parts, e...)
	}
	if len(t.PrivateUse) > 0 {
		parts = append(append(parts, "x"), t.PrivateUse...)
	}
	return strings.Join(parts, "-")
}

// Canonicalize returns the canonical form of tag, e.g. "EN_us" -> "en-US".
func Canonicalize(tag string) (string, error) {
	t, err := ParseTag(tag)
	if err != nil {
		return "", err
	}
	return t.String(), nil
}

// Negotiate performs an RFC 4647 lookup over requested (in order, case-insensitive)
// and returns the match as written in available, or def.
func Negotiate(requested, available []string, def string) string {
	type cand struct{ key, orig string }
	var avail []cand
	for _, a := range available {
		if t, err := ParseTag(a); err == nil {
			avail = append(avail, cand{strings.ToLower(t.Base()), a})
		}
	}
	for _, r := range requested {
		t, err := ParseTag(r)
		if err != nil {
			continue
		}
		for _, c := range t.LookupChain() {
			for _, a := range avail {
				if a.key == strings.ToLower(c) {
					return a.orig
				}
			}
		}
	}
	return def
}

// DataLocale returns the CLDR data locale used to format for tag (SPEC section 2.3).
func DataLocale(tag string) string {
	t, err := ParseTag(tag)
	if err != nil {
		return "en"
	}
	if t.Language == "zh" && t.Script == "" && (t.Region == "TW" || t.Region == "HK" || t.Region == "MO") {
		t.Script = "Hant"
	}
	for _, c := range t.LookupChain() {
		if _, ok := locales[c]; ok {
			return c
		}
	}
	return "en"
}

// ---------------------------------------------------------------- decimals

type dec struct {
	kind      byte // 'f' finite, 'n' NaN, 'i' infinity
	neg       bool
	int, frac string
}

var decRe = regexp.MustCompile(`^([+-]?)([0-9]+)(?:\.([0-9]+))?(?:[eE]([+-]?[0-9]{1,4}))?$`)

// FloatString returns the shortest decimal text that reads back as f
// ("NaN", "Infinity" and "-Infinity" for the special values).
func FloatString(f float64) string {
	switch {
	case math.IsNaN(f):
		return "NaN"
	case math.IsInf(f, 1):
		return "Infinity"
	case math.IsInf(f, -1):
		return "-Infinity"
	}
	return strconv.FormatFloat(f, 'g', -1, 64)
}

func parseDec(s string) (dec, error) {
	switch s {
	case "NaN":
		return dec{kind: 'n'}, nil
	case "Infinity", "+Infinity":
		return dec{kind: 'i'}, nil
	case "-Infinity":
		return dec{kind: 'i', neg: true}, nil
	}
	m := decRe.FindStringSubmatch(s)
	if m == nil {
		return dec{}, newErr("BAD_NUMBER", s)
	}
	exp := 0
	if m[4] != "" {
		exp, _ = strconv.Atoi(m[4])
		if exp > MaxExponent || exp < -MaxExponent {
			return dec{}, newErr("BAD_NUMBER", s)
		}
	}
	digits := m[2] + m[3]
	point := len(m[2]) + exp
	if point <= 0 {
		digits = strings.Repeat("0", 1-point) + digits
		point = 1
	}
	if point > len(digits) {
		digits += strings.Repeat("0", point-len(digits))
	}
	ip := strings.TrimLeft(digits[:point], "0")
	if ip == "" {
		ip = "0"
	}
	return dec{kind: 'f', neg: m[1] == "-", int: ip, frac: digits[point:]}, nil
}

func roundHalfExpand(ip, fp string, max int) (string, string) {
	if len(fp) <= max {
		return ip, fp
	}
	b := []byte(ip + fp[:max])
	if fp[max] >= '5' {
		k := len(b) - 1
		for {
			if k < 0 {
				b = append([]byte{'1'}, b...)
				break
			}
			if b[k] == '9' {
				b[k] = '0'
				k--
				continue
			}
			b[k]++
			break
		}
	}
	split := len(b) - max
	i := strings.TrimLeft(string(b[:split]), "0")
	if i == "" {
		i = "0"
	}
	return i, string(b[split:])
}

// ---------------------------------------------------------------- plural

func operand(ip, fp, op string, mod int) (bool, int64, bool) {
	trimmed := strings.TrimRight(fp, "0")
	num := func(s string) (int64, bool) {
		if mod > 0 {
			var r int64
			for i := 0; i < len(s); i++ {
				r = (r*10 + int64(s[i]-'0')) % int64(mod)
			}
			return r, true
		}
		if len(s) > 18 {
			return 0, false
		}
		v, _ := strconv.ParseInt(s, 10, 64)
		return v, true
	}
	small := func(n int) (int64, bool) {
		if mod > 0 {
			return int64(n % mod), true
		}
		return int64(n), true
	}
	or0 := func(s string) string {
		if s == "" {
			return "0"
		}
		return s
	}
	var v int64
	var ok bool
	isInt := true
	switch op {
	case "n":
		isInt = strings.Trim(fp, "0") == ""
		v, ok = num(ip)
	case "i":
		v, ok = num(ip)
	case "v":
		v, ok = small(len(fp))
	case "w":
		v, ok = small(len(trimmed))
	case "f":
		v, ok = num(or0(fp))
	case "t":
		v, ok = num(or0(trimmed))
	default:
		v, ok = 0, true
	}
	return isInt, v, ok
}

func selectCategory(table map[string]rules, locale, value string) (string, error) {
	d, err := parseDec(value)
	if err != nil {
		return "", err
	}
	if d.kind != 'f' {
		return "", newErr("BAD_NUMBER", value)
	}
	var rs rules
	found := false
	if t, err := ParseTag(locale); err == nil {
		for _, c := range t.LookupChain() {
			if r, ok := table[c]; ok {
				rs, found = r, true
				break
			}
		}
	}
	if !found {
		rs = table["root"]
	}
	for _, entry := range rs {
		for _, rels := range entry.Groups {
			all := true
			for _, r := range rels {
				isInt, v, ok := operand(d.int, d.frac, r.Op, r.Mod)
				inside := false
				if isInt && ok {
					for _, rg := range r.Ranges {
						if int64(rg[0]) <= v && v <= int64(rg[1]) {
							inside = true
							break
						}
					}
				}
				if inside == r.Neg {
					all = false
					break
				}
			}
			if all {
				return entry.Cat, nil
			}
		}
	}
	return "other", nil
}

// PluralCategory returns the cardinal plural category of value (a decimal
// string; trailing zeros count as visible fraction digits).
func PluralCategory(locale, value string) (string, error) {
	return selectCategory(plurals, locale, value)
}

// OrdinalCategory returns the ordinal plural category (1st, 2nd, 3rd, ...).
func OrdinalCategory(locale, value string) (string, error) {
	return selectCategory(ordinals, locale, value)
}

// ---------------------------------------------------------------- numbers

// NumberOptions configures FormatNumber. Style is "decimal" (default),
// "percent" or "currency"; nil fraction limits use the style defaults.
type NumberOptions struct {
	Style       string
	Currency    string
	MinFraction *int
	MaxFraction *int
}

// Digits returns a pointer to n, for NumberOptions fields.
func Digits(n int) *int { return &n }

func splitPattern(p string) (string, string, string) {
	quoted := false
	start, end := -1, 0
	for i := 0; i < len(p); i++ {
		c := p[i]
		if c == '\'' {
			quoted = !quoted
		} else if !quoted && strings.IndexByte("#0,.", c) >= 0 {
			if start < 0 {
				start = i
			}
			end = i + 1
		}
	}
	if start < 0 {
		start, end = len(p), len(p)
	}
	return p[:start], p[start:end], p[end:]
}

func groupInt(ip string, primary, secondary, minGrouping int, sep string) string {
	if primary == 0 || len(ip) < primary+minGrouping {
		return ip
	}
	head := ip[:len(ip)-primary]
	parts := []string{ip[len(ip)-primary:]}
	for len(head) > secondary {
		parts = append([]string{head[len(head)-secondary:]}, parts...)
		head = head[:len(head)-secondary]
	}
	if head != "" {
		parts = append([]string{head}, parts...)
	}
	return strings.Join(parts, sep)
}

func expandAffix(raw string, d *localeData, symbol string) string {
	var b strings.Builder
	quoted := false
	rs := []rune(raw)
	for i := 0; i < len(rs); i++ {
		c := rs[i]
		switch {
		case c == '\'':
			if i+1 < len(rs) && rs[i+1] == '\'' {
				b.WriteRune('\'')
				i++
				continue
			}
			quoted = !quoted
		case quoted:
			b.WriteRune(c)
		case c == '¤':
			b.WriteString(symbol)
		case c == '%':
			b.WriteString(d.Percent)
		case c == '-':
			b.WriteString(d.Minus)
		default:
			b.WriteRune(c)
		}
	}
	return b.String()
}

var currencyRe = regexp.MustCompile(`^[A-Za-z]{3}$`)

// FormatNumber formats value (a decimal string, "NaN", "Infinity" or
// "-Infinity") for locale: half away from zero, CLDR grouping and symbols.
func FormatNumber(locale, value string, opts NumberOptions) (string, error) {
	dl := DataLocale(locale)
	d := locales[dl]
	var dmin, dmax int
	var pattern, code string
	switch opts.Style {
	case "", "decimal":
		dmin, dmax, pattern = 0, 3, d.DecimalPattern
	case "percent":
		dmin, dmax, pattern = 0, 0, d.PercentPattern
	case "currency":
		if !currencyRe.MatchString(opts.Currency) {
			return "", newErr("BAD_OPTION", "currency")
		}
		code = strings.ToUpper(opts.Currency)
		dmin = 2
		if v, ok := currencyDigits[code]; ok {
			dmin = v
		}
		dmax, pattern = dmin, d.CurrencyPattern
	default:
		return "", newErr("BAD_OPTION", "style")
	}
	minf, maxf := dmin, dmax
	switch {
	case opts.MinFraction != nil && opts.MaxFraction != nil:
		minf, maxf = *opts.MinFraction, *opts.MaxFraction
	case opts.MinFraction != nil:
		minf = *opts.MinFraction
		if minf > dmax {
			maxf = minf
		}
	case opts.MaxFraction != nil:
		maxf = *opts.MaxFraction
		if maxf < dmin {
			minf = maxf
		}
	}
	if minf < 0 || maxf > 20 || minf > maxf {
		return "", newErr("BAD_OPTION", "fractionDigits")
	}
	pos, negp, hasNeg := strings.Cut(pattern, ";")
	prefix, numpart, suffix := splitPattern(pos)
	v, err := parseDec(value)
	if err != nil {
		return "", err
	}
	var neg bool
	var body string
	switch v.kind {
	case 'n':
		body = d.NaN
	case 'i':
		neg, body = v.neg, d.Infinity
	default:
		neg = v.neg
		ip, fp := v.int, v.frac
		if opts.Style == "percent" {
			fp += "00"
			ip = strings.TrimLeft(ip+fp[:2], "0")
			if ip == "" {
				ip = "0"
			}
			fp = fp[2:]
		}
		ip, fp = roundHalfExpand(ip, fp, maxf)
		for len(fp) < minf {
			fp += "0"
		}
		for len(fp) > minf && strings.HasSuffix(fp, "0") {
			fp = fp[:len(fp)-1]
		}
		groups := strings.Split(strings.Split(numpart, ".")[0], ",")
		primary, secondary := 0, 0
		if len(groups) > 1 {
			primary = len(groups[len(groups)-1])
		}
		secondary = primary
		if len(groups) > 2 {
			secondary = len(groups[len(groups)-2])
		}
		body = groupInt(ip, primary, secondary, d.MinGrouping, d.Group)
		if fp != "" {
			body += d.Decimal + fp
		}
	}
	np, ns := prefix, suffix
	if neg && hasNeg {
		np, _, ns = splitPattern(negp)
	} else if neg {
		np = "-" + prefix
	}
	if opts.Style == "currency" {
		sym, ok := symbols[dl][code]
		if !ok {
			sym = currencySymbol{symbol: code}
		}
		pre, suf := expandAffix(np, d, sym.symbol), expandAffix(ns, d, sym.symbol)
		if strings.HasSuffix(np, "¤") && !sym.lastSZ && body != "" && body[0] >= '0' && body[0] <= '9' {
			pre += " "
		}
		if strings.HasPrefix(ns, "¤") && !sym.firstSZ && body != "" && body[len(body)-1] >= '0' && body[len(body)-1] <= '9' {
			suf = " " + suf
		}
		return pre + body + suf, nil
	}
	return expandAffix(np, d, "") + body + expandAffix(ns, d, ""), nil
}

// ---------------------------------------------------------------- dates

var dateRe = regexp.MustCompile(`^([0-9]{4})-([0-9]{2})-([0-9]{2})$`)

func isLeap(y int) bool { return y%4 == 0 && (y%100 != 0 || y%400 == 0) }

func daysIn(y, m int) int {
	switch m {
	case 2:
		if isLeap(y) {
			return 29
		}
		return 28
	case 4, 6, 9, 11:
		return 30
	}
	return 31
}

// ParseDate parses YYYY-MM-DD (0001-01-01 to 9999-12-31).
func ParseDate(s string) (year, month, day int, err error) {
	m := dateRe.FindStringSubmatch(s)
	if m == nil {
		return 0, 0, 0, newErr("BAD_DATE", s)
	}
	year, _ = strconv.Atoi(m[1])
	month, _ = strconv.Atoi(m[2])
	day, _ = strconv.Atoi(m[3])
	if year < 1 || month < 1 || month > 12 || day < 1 || day > daysIn(year, month) {
		return 0, 0, 0, newErr("BAD_DATE", s)
	}
	return year, month, day, nil
}

// Weekday returns the day of the week of a valid date, 0 = Sunday.
func Weekday(year, month, day int) int {
	p := year - 1
	days := p*365 + p/4 - p/100 + p/400
	for m := 1; m < month; m++ {
		days += daysIn(year, m)
	}
	days += day - 1
	return (days + 1) % 7
}

func pad(v, n int) string {
	s := strconv.Itoa(v)
	for len(s) < n {
		s = "0" + s
	}
	return s
}

// FormatDate formats YYYY-MM-DD with the CLDR pattern of style
// ("full", "long", "medium" or "short"; Gregorian calendar).
func FormatDate(locale, date, style string) (string, error) {
	if style != "full" && style != "long" && style != "medium" && style != "short" {
		return "", newErr("BAD_OPTION", "style")
	}
	y, mo, dd, err := ParseDate(date)
	if err != nil {
		return "", err
	}
	d := locales[DataLocale(locale)]
	p := []rune(d.DateFormats[style])
	var b strings.Builder
	for i := 0; i < len(p); {
		c := p[i]
		if c == '\'' {
			j := i + 1
			for j < len(p) {
				if p[j] == '\'' {
					if j+1 < len(p) && p[j+1] == '\'' {
						b.WriteRune('\'')
						j += 2
						continue
					}
					break
				}
				b.WriteRune(p[j])
				j++
			}
			if j == i+1 {
				b.WriteRune('\'')
			}
			i = j + 1
			continue
		}
		if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') {
			j := i
			for j < len(p) && p[j] == c {
				j++
			}
			n := j - i
			switch c {
			case 'G':
				b.WriteString(d.Era)
			case 'y':
				if n == 2 {
					b.WriteString(pad(y%100, 2))
				} else {
					b.WriteString(pad(y, n))
				}
			case 'M', 'L':
				if n <= 2 {
					b.WriteString(pad(mo, n))
				} else if n == 3 {
					b.WriteString(d.Months[0][mo-1])
				} else {
					b.WriteString(d.Months[1][mo-1])
				}
			case 'd':
				b.WriteString(pad(dd, n))
			case 'E':
				w := 0
				if n == 4 {
					w = 1
				}
				b.WriteString(d.Days[w][Weekday(y, mo, dd)])
			default:
				panic("pattern letter not used by the CLDR 47 subset: " + string(c))
			}
			i = j
			continue
		}
		b.WriteRune(c)
		i++
	}
	return b.String(), nil
}

// ---------------------------------------------------------------- messages

// Arg is a message argument: text, or a number given as decimal text.
type Arg struct {
	Text  string
	Num   string
	IsNum bool
}

// Str returns a text argument.
func Str(s string) Arg { return Arg{Text: s} }

// Num returns a number argument from decimal text such as "1.50".
func Num(s string) Arg { return Arg{Num: s, IsNum: true} }

// Int returns a number argument.
func Int(v int64) Arg { return Num(strconv.FormatInt(v, 10)) }

// Float returns a number argument (shortest round-trip text).
func Float(v float64) Arg { return Num(FloatString(v)) }

type part struct {
	kind   string // text, pound, simple, number, date, plural, selectordinal, select
	text   string
	name   string
	style  string
	offset int64
	keys   []string
	bodies [][]part
}

type msgParser struct {
	s []rune
	i int
}

var nameRe = regexp.MustCompile(`^[A-Za-z0-9_]+$`)
var exactRe = regexp.MustCompile(`^=[0-9]+(\.[0-9]+)?$`)

func (p *msgParser) syntax() error { return newErr("SYNTAX", strconv.Itoa(p.i)) }

func (p *msgParser) at(k int) rune {
	if k < len(p.s) {
		return p.s[k]
	}
	return 0
}

func (p *msgParser) message(depth int, inPlural, top bool) ([]part, error) {
	if depth > MaxDepth {
		return nil, newErr("TOO_DEEP", "")
	}
	var parts []part
	var buf strings.Builder
	flush := func() {
		if buf.Len() > 0 {
			parts = append(parts, part{kind: "text", text: buf.String()})
			buf.Reset()
		}
	}
	for p.i < len(p.s) {
		c := p.s[p.i]
		switch {
		case c == '\'':
			next := p.at(p.i + 1)
			if next == '\'' {
				buf.WriteRune('\'')
				p.i += 2
			} else if next == '{' || next == '}' || next == '|' || (next == '#' && inPlural) {
				p.i++
				for p.i < len(p.s) {
					if p.s[p.i] == '\'' {
						if p.at(p.i+1) == '\'' {
							buf.WriteRune('\'')
							p.i += 2
							continue
						}
						p.i++
						break
					}
					buf.WriteRune(p.s[p.i])
					p.i++
				}
			} else {
				buf.WriteRune('\'')
				p.i++
			}
		case c == '{':
			flush()
			p.i++
			a, err := p.argument(depth, inPlural)
			if err != nil {
				return nil, err
			}
			parts = append(parts, a)
		case c == '}':
			if top {
				return nil, p.syntax()
			}
			flush()
			return parts, nil
		case c == '#' && inPlural:
			flush()
			parts = append(parts, part{kind: "pound"})
			p.i++
		default:
			buf.WriteRune(c)
			p.i++
		}
	}
	flush()
	return parts, nil
}

func (p *msgParser) ws() {
	for p.i < len(p.s) && strings.ContainsRune(" \t\r\n", p.s[p.i]) {
		p.i++
	}
}

func (p *msgParser) word() string {
	p.ws()
	start := p.i
	for p.i < len(p.s) && !strings.ContainsRune(" \t\r\n,{}", p.s[p.i]) {
		p.i++
	}
	return string(p.s[start:p.i])
}

func (p *msgParser) expect(c rune) error {
	p.ws()
	if p.at(p.i) != c || p.i >= len(p.s) {
		return p.syntax()
	}
	p.i++
	return nil
}

var keywords = map[string]bool{"zero": true, "one": true, "two": true, "few": true, "many": true, "other": true}

func (p *msgParser) argument(depth int, inPlural bool) (part, error) {
	name := p.word()
	if !nameRe.MatchString(name) {
		return part{}, p.syntax()
	}
	p.ws()
	if p.i >= len(p.s) {
		return part{}, p.syntax()
	}
	if p.s[p.i] == '}' {
		p.i++
		return part{kind: "simple", name: name}, nil
	}
	if err := p.expect(','); err != nil {
		return part{}, err
	}
	kind := p.word()
	if kind == "number" || kind == "date" {
		p.ws()
		style := ""
		if p.at(p.i) == ',' && p.i < len(p.s) {
			p.i++
			style = p.word()
			ok := false
			if kind == "number" {
				ok = style == "integer" || style == "percent"
			} else {
				ok = style == "full" || style == "long" || style == "medium" || style == "short"
			}
			if !ok {
				return part{}, p.syntax()
			}
		}
		if err := p.expect('}'); err != nil {
			return part{}, err
		}
		if kind == "date" && style == "" {
			style = "medium"
		}
		return part{kind: kind, name: name, style: style}, nil
	}
	if kind != "plural" && kind != "selectordinal" && kind != "select" {
		return part{}, p.syntax()
	}
	if err := p.expect(','); err != nil {
		return part{}, err
	}
	out := part{kind: kind, name: name}
	seenOffset := false
	for {
		p.ws()
		if p.i >= len(p.s) {
			return part{}, p.syntax()
		}
		if p.s[p.i] == '}' {
			p.i++
			break
		}
		key := p.word()
		if strings.HasPrefix(key, "offset:") && kind == "plural" && len(out.keys) == 0 && !seenOffset {
			rest := key[7:]
			if rest == "" {
				rest = p.word()
			}
			if !digitsRe.MatchString(rest) {
				return part{}, p.syntax()
			}
			v, err := strconv.ParseInt(rest, 10, 64)
			if err != nil {
				return part{}, p.syntax()
			}
			out.offset, seenOffset = v, true
			continue
		}
		if key == "" {
			return part{}, p.syntax()
		}
		for _, k := range out.keys {
			if k == key {
				return part{}, p.syntax()
			}
		}
		var valid bool
		switch {
		case kind == "select":
			valid = nameRe.MatchString(key)
		case strings.HasPrefix(key, "="):
			valid = exactRe.MatchString(key)
		default:
			valid = keywords[key]
		}
		if !valid {
			return part{}, p.syntax()
		}
		if err := p.expect('{'); err != nil {
			return part{}, err
		}
		body, err := p.message(depth+1, kind != "select" || inPlural, false)
		if err != nil {
			return part{}, err
		}
		if p.i >= len(p.s) || p.s[p.i] != '}' {
			return part{}, p.syntax()
		}
		p.i++
		out.keys = append(out.keys, key)
		out.bodies = append(out.bodies, body)
	}
	for _, k := range out.keys {
		if k == "other" {
			return out, nil
		}
	}
	return part{}, newErr("MISSING_OTHER", name)
}

func finiteDec(v string) (dec, bool) {
	d, err := parseDec(v)
	return d, err == nil && d.kind == 'f'
}

func decEq(a, b string) bool {
	x, ok1 := finiteDec(a)
	y, ok2 := finiteDec(b)
	if !ok1 || !ok2 {
		return false
	}
	zero := func(d dec) bool { return d.int == "0" && strings.Trim(d.frac, "0") == "" }
	if zero(x) && zero(y) {
		return true
	}
	return x.neg == y.neg && x.int == y.int && strings.TrimRight(x.frac, "0") == strings.TrimRight(y.frac, "0")
}

func decSub(value string, offset int64) (string, bool) {
	if offset == 0 {
		return value, true
	}
	d, _ := finiteDec(value)
	if len(d.int+d.frac) > 36 {
		return "", false
	}
	scale := len(d.frac)
	n, _ := new(big.Int).SetString(d.int+d.frac, 10)
	if d.neg {
		n.Neg(n)
	}
	n.Sub(n, new(big.Int).Mul(big.NewInt(offset), new(big.Int).Exp(big.NewInt(10), big.NewInt(int64(scale)), nil)))
	s := new(big.Int).Abs(n).String()
	for len(s) < scale+1 {
		s = "0" + s
	}
	out := s[:len(s)-scale]
	if scale > 0 {
		out += "." + s[len(s)-scale:]
	}
	if n.Sign() < 0 {
		out = "-" + out
	}
	return out, true
}

func forPlural(value string) string {
	d, _ := finiteDec(value)
	ip, fp := roundHalfExpand(d.int, d.frac, 3)
	fp = strings.TrimRight(fp, "0")
	if fp == "" {
		return ip
	}
	return ip + "." + fp
}

func render(locale string, parts []part, args map[string]Arg, pound *string, b *strings.Builder) error {
	look := func(name string) (Arg, error) {
		a, ok := args[name]
		if !ok {
			return Arg{}, newErr("MISSING_ARGUMENT", name)
		}
		return a, nil
	}
	num := func(name, v string, opts NumberOptions) error {
		s, err := FormatNumber(locale, v, opts)
		if err != nil {
			return newErr("BAD_ARGUMENT", name)
		}
		b.WriteString(s)
		return nil
	}
	for _, p := range parts {
		switch p.kind {
		case "text":
			b.WriteString(p.text)
		case "pound":
			if pound == nil {
				b.WriteByte('#')
			} else {
				s, _ := FormatNumber(locale, *pound, NumberOptions{})
				b.WriteString(s)
			}
		case "simple":
			a, err := look(p.name)
			if err != nil {
				return err
			}
			if !a.IsNum {
				b.WriteString(a.Text)
			} else if err := num(p.name, a.Num, NumberOptions{}); err != nil {
				return err
			}
		case "number":
			a, err := look(p.name)
			if err != nil {
				return err
			}
			if !a.IsNum {
				return newErr("BAD_ARGUMENT", p.name)
			}
			opts := NumberOptions{}
			if p.style == "integer" {
				opts.MaxFraction = Digits(0)
			} else if p.style == "percent" {
				opts.Style = "percent"
			}
			if err := num(p.name, a.Num, opts); err != nil {
				return err
			}
		case "date":
			a, err := look(p.name)
			if err != nil {
				return err
			}
			if a.IsNum {
				return newErr("BAD_ARGUMENT", p.name)
			}
			s, err := FormatDate(locale, a.Text, p.style)
			if err != nil {
				return newErr("BAD_ARGUMENT", p.name)
			}
			b.WriteString(s)
		default:
			a, err := look(p.name)
			if err != nil {
				return err
			}
			var other []part
			for k, key := range p.keys {
				if key == "other" {
					other = p.bodies[k]
				}
			}
			if p.kind == "select" {
				key := a.Text
				if a.IsNum {
					key = a.Num
				}
				chosen := other
				for k, kk := range p.keys {
					if kk == key {
						chosen = p.bodies[k]
						break
					}
				}
				if err := render(locale, chosen, args, pound, b); err != nil {
					return err
				}
				continue
			}
			if !a.IsNum {
				return newErr("BAD_ARGUMENT", p.name)
			}
			if _, ok := finiteDec(a.Num); !ok {
				return newErr("BAD_ARGUMENT", p.name)
			}
			shown, ok := decSub(a.Num, p.offset)
			if !ok {
				return newErr("BAD_ARGUMENT", p.name)
			}
			var chosen []part
			for k, key := range p.keys {
				if strings.HasPrefix(key, "=") && decEq(key[1:], a.Num) {
					chosen = p.bodies[k]
					break
				}
			}
			if chosen == nil {
				v := forPlural(shown)
				var cat string
				if p.kind == "selectordinal" {
					cat, _ = OrdinalCategory(locale, v)
				} else {
					cat, _ = PluralCategory(locale, v)
				}
				chosen = other
				for k, key := range p.keys {
					if key == cat {
						chosen = p.bodies[k]
						break
					}
				}
			}
			if err := render(locale, chosen, args, &shown, b); err != nil {
				return err
			}
		}
	}
	return nil
}

// FormatMessage formats an ICU MessageFormat pattern with args for locale (SPEC section 7).
func FormatMessage(locale, pattern string, args map[string]Arg) (string, error) {
	p := &msgParser{s: []rune(pattern)}
	parts, err := p.message(0, false, true)
	if err != nil {
		return "", err
	}
	var b strings.Builder
	if err := render(locale, parts, args, nil, &b); err != nil {
		return "", err
	}
	return b.String(), nil
}

// ---------------------------------------------------------------- catalogs

const maxJSONDepth = 64

type jsonReader struct {
	s string
	i int
}

func (r *jsonReader) bad() error { return newErr("BAD_JSON", strconv.Itoa(r.i)) }

func (r *jsonReader) ws() {
	for r.i < len(r.s) && strings.IndexByte(" \t\n\r", r.s[r.i]) >= 0 {
		r.i++
	}
}

func (r *jsonReader) hex4() (rune, error) {
	if r.i+4 > len(r.s) {
		return 0, r.bad()
	}
	v, err := strconv.ParseUint(r.s[r.i:r.i+4], 16, 32)
	if err != nil || strings.ContainsAny(r.s[r.i:r.i+4], "+-") {
		return 0, r.bad()
	}
	r.i += 4
	return rune(v), nil
}

func (r *jsonReader) str() (string, error) {
	if r.i >= len(r.s) || r.s[r.i] != '"' {
		return "", r.bad()
	}
	r.i++
	var b strings.Builder
	esc := map[byte]string{'"': "\"", '\\': "\\", '/': "/", 'b': "\b", 'f': "\f", 'n': "\n", 'r': "\r", 't': "\t"}
	for {
		if r.i >= len(r.s) {
			return "", r.bad()
		}
		c := r.s[r.i]
		r.i++
		switch {
		case c == '"':
			return b.String(), nil
		case c == '\\':
			if r.i >= len(r.s) {
				return "", r.bad()
			}
			e := r.s[r.i]
			r.i++
			if v, ok := esc[e]; ok {
				b.WriteString(v)
			} else if e == 'u' {
				cp, err := r.hex4()
				if err != nil {
					return "", err
				}
				if cp >= 0xD800 && cp < 0xDC00 {
					if r.i+2 > len(r.s) || r.s[r.i:r.i+2] != `\u` {
						return "", r.bad()
					}
					r.i += 2
					lo, err := r.hex4()
					if err != nil || lo < 0xDC00 || lo >= 0xE000 {
						return "", r.bad()
					}
					cp = 0x10000 + (cp-0xD800)<<10 + (lo - 0xDC00)
				} else if cp >= 0xDC00 && cp < 0xE000 {
					return "", r.bad()
				}
				b.WriteRune(cp)
			} else {
				return "", r.bad()
			}
		case c < 0x20:
			return "", r.bad()
		default:
			b.WriteByte(c)
		}
	}
}

var jsonNumRe = regexp.MustCompile(`^-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][+-]?[0-9]+)?`)

func (r *jsonReader) skip(depth int) error {
	if depth > maxJSONDepth {
		return r.bad()
	}
	r.ws()
	if r.i >= len(r.s) {
		return r.bad()
	}
	switch c := r.s[r.i]; c {
	case '"':
		_, err := r.str()
		return err
	case '{', '[':
		closeB := byte('}')
		if c == '[' {
			closeB = ']'
		}
		r.i++
		r.ws()
		if r.i < len(r.s) && r.s[r.i] == closeB {
			r.i++
			return nil
		}
		for {
			r.ws()
			if closeB == '}' {
				if _, err := r.str(); err != nil {
					return err
				}
				r.ws()
				if r.i >= len(r.s) || r.s[r.i] != ':' {
					return r.bad()
				}
				r.i++
			}
			if err := r.skip(depth + 1); err != nil {
				return err
			}
			r.ws()
			if r.i >= len(r.s) {
				return r.bad()
			}
			n := r.s[r.i]
			r.i++
			if n == closeB {
				return nil
			}
			if n != ',' {
				return r.bad()
			}
		}
	}
	for _, lit := range []string{"true", "false", "null"} {
		if strings.HasPrefix(r.s[r.i:], lit) {
			r.i += len(lit)
			return nil
		}
	}
	m := jsonNumRe.FindString(r.s[r.i:])
	if m == "" {
		return r.bad()
	}
	r.i += len(m)
	return nil
}

// ParseCatalog parses a catalog: a JSON object (RFC 8259) whose values are all strings.
func ParseCatalog(text string) (map[string]string, error) {
	if !utf8Valid(text) {
		return nil, newErr("BAD_JSON", "0")
	}
	check := &jsonReader{s: text}
	if err := check.skip(0); err != nil {
		return nil, err
	}
	check.ws()
	if check.i != len(text) {
		return nil, check.bad()
	}
	r := &jsonReader{s: text}
	r.ws()
	if r.s[r.i] != '{' {
		return nil, newErr("NOT_OBJECT", "")
	}
	r.i++
	out := map[string]string{}
	r.ws()
	if r.s[r.i] == '}' {
		return out, nil
	}
	for {
		r.ws()
		key, _ := r.str()
		r.ws()
		r.i++
		r.ws()
		if r.s[r.i] != '"' {
			return nil, newErr("NON_STRING_VALUE", key)
		}
		value, _ := r.str()
		if _, dup := out[key]; dup {
			return nil, newErr("DUPLICATE_KEY", key)
		}
		out[key] = value
		r.ws()
		if r.s[r.i] != ',' {
			return out, nil
		}
		r.i++
	}
}

// LocalizedMessage is a catalog-backed message; Fallback is true when Text is "!!id!!".
type LocalizedMessage struct {
	Code      string
	MessageID string
	Text      string
	Fallback  bool
}

// Resolve looks up messageID and formats it; a missing id or a failing
// pattern gives "!!messageID!!".
func Resolve(locale string, catalog map[string]string, code, messageID string, args map[string]Arg) LocalizedMessage {
	if pattern, ok := catalog[messageID]; ok {
		if text, err := FormatMessage(locale, pattern, args); err == nil {
			return LocalizedMessage{code, messageID, text, false}
		}
	}
	return LocalizedMessage{code, messageID, "!!" + messageID + "!!", true}
}

func utf8Valid(s string) bool { return utf8.ValidString(s) }
