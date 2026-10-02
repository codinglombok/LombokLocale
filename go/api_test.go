package lomboklocale

import (
	"errors"
	"math"
	"reflect"
	"strings"
	"testing"
)

func code(err error) string {
	var e *Error
	if errors.As(err, &e) {
		return e.Code
	}
	return "none"
}

func TestTagParts(t *testing.T) {
	tag, err := ParseTag("sl-Latn-IT-rozaj-u-ca-gregory-x-priv")
	if err != nil {
		t.Fatal(err)
	}
	if tag.Language != "sl" || tag.Script != "Latn" || tag.Region != "IT" || tag.Base() != "sl-Latn-IT-rozaj" {
		t.Fatalf("%+v", tag)
	}
	if !reflect.DeepEqual(tag.LookupChain(), []string{"sl-Latn-IT-rozaj", "sl-Latn-IT", "sl-Latn", "sl"}) {
		t.Fatal(tag.LookupChain())
	}
	if tag.String() != "sl-Latn-IT-rozaj-u-ca-gregory-x-priv" {
		t.Fatal(tag.String())
	}
	if s, _ := Canonicalize("en-b-aa-a-bb"); s != "en-a-bb-b-aa" {
		t.Fatal(s)
	}
}

func TestErrors(t *testing.T) {
	e := &Error{Code: "INVALID_SUBTAG", Detail: "$"}
	if e.Error() != "INVALID_SUBTAG: $" || (&Error{Code: "EMPTY"}).Error() != "EMPTY" {
		t.Fatal(e.Error())
	}
	if CLDRVersion() != "47.0.0" || MaxDepth != 16 || MaxExponent != 1000 {
		t.Fatal("constants")
	}
}

func TestNumbersAndArgs(t *testing.T) {
	cases := map[float64]string{1.5: "1.5", math.NaN(): "NaN", math.Inf(1): "Infinity", math.Inf(-1): "-Infinity", 1e21: "1e+21"}
	for f, want := range cases {
		if got := FloatString(f); got != want {
			t.Errorf("%v: %s", f, got)
		}
	}
	if s, _ := FormatNumber("en", FloatString(0.1+0.2), NumberOptions{}); s != "0.3" {
		t.Fatal(s)
	}
	if _, err := FormatNumber("en", "1", NumberOptions{Style: "scientific"}); code(err) != "BAD_OPTION" {
		t.Fatal(err)
	}
	if _, err := FormatNumber("en", "1", NumberOptions{MinFraction: Digits(-1)}); code(err) != "BAD_OPTION" {
		t.Fatal(err)
	}
	if _, err := PluralCategory("en", "NaN"); code(err) != "BAD_NUMBER" {
		t.Fatal(err)
	}
	if _, err := PluralCategory("en", "x"); code(err) != "BAD_NUMBER" {
		t.Fatal(err)
	}
	s, err := FormatMessage("en", "{a} {b} {c}", map[string]Arg{"a": Int(-3), "b": Float(1234.5), "c": Str("x")})
	if err != nil || s != "-3 1,234.5 x" {
		t.Fatal(s, err)
	}
	if s, _ := FormatMessage("en", "{n, select, 1{one} other{x}}", map[string]Arg{"n": Int(1)}); s != "one" {
		t.Fatal(s)
	}
	if s, _ := FormatMessage("en", "{n, plural, offset: 1 other{#}}", map[string]Arg{"n": Int(3)}); s != "2" {
		t.Fatal(s)
	}
	for _, p := range []string{"{n, plural, other{#}", "{n, plural, other x}", "{n, plural,", "{n, plural, offset:x other{#}}", "{n, date, huge}", "{n", "{n,", "{n, number,"} {
		if _, err := FormatMessage("en", p, map[string]Arg{"n": Int(1)}); code(err) != "SYNTAX" {
			t.Errorf("%s: %v", p, err)
		}
	}
	if s, _ := FormatMessage("en", "it's '{' unterminated", nil); s != "it's { unterminated" {
		t.Fatal(s)
	}
	if s, _ := FormatMessage("en", "'{a''b}'", nil); s != "{a'b}" {
		t.Fatal(s)
	}
}

func TestDates(t *testing.T) {
	y, m, d, err := ParseDate("2026-10-02")
	if err != nil || y != 2026 || m != 10 || d != 2 || Weekday(y, m, d) != 5 {
		t.Fatal(y, m, d, err)
	}
	if _, err := FormatDate("en", "2026-10-02", "x"); code(err) != "BAD_OPTION" {
		t.Fatal(err)
	}
}

func TestCatalogs(t *testing.T) {
	bad := []string{`{"a": tru}`, `{"a": -}`, `{"a": 1.}`, `{"a" "b"}`, `{"a": "\x"}`, `{"a": "\u12"}`, `{"a": "\udc00"}`,
		`{"a": "\ud800A"}`, `{"a": [1 2]}`, strings.Repeat("[", 100) + strings.Repeat("]", 100), `{"a": "x"`, "{\"a\": \"\xff\"}", `{"a": "\u+123"}`, `{"a": "\`, `{"a`, `{"a":`, `[`}
	for _, b := range bad {
		if _, err := ParseCatalog(b); code(err) != "BAD_JSON" {
			t.Errorf("%q: %v", b, err)
		}
	}
	if _, err := ParseCatalog(`{"a": {}, "b": []}`); code(err) != "NON_STRING_VALUE" {
		t.Fatal(err)
	}
	c, _ := ParseCatalog(`{"a": "\b\f\r\/\"\\"}`)
	if c["a"] != "\b\f\r/\"\\" {
		t.Fatal(c)
	}
	m := Resolve("id", map[string]string{"c": "{n, plural, other{# hal}}"}, "C", "c", map[string]Arg{"n": Int(2000)})
	if m.Text != "2.000 hal" || m.Fallback {
		t.Fatal(m)
	}
	if Resolve("id", nil, "X", "y", nil).Text != "!!y!!" {
		t.Fatal("fallback")
	}
}
