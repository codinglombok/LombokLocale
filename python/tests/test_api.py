"""API behaviour not covered by the shared vectors."""
from decimal import Decimal

import pytest

import lomboklocale as ll


def code(fn):
    with pytest.raises(ll.LocaleError) as e:
        fn()
    return e.value.code


def test_tag_parts():
    t = ll.parse_tag("sl-Latn-IT-rozaj-u-ca-gregory-x-priv")
    assert (t.language, t.script, t.region) == ("sl", "Latn", "IT")
    assert t.variants == ["rozaj"] and t.extensions == [["u", "ca", "gregory"]] and t.private_use == ["priv"]
    assert t.base() == "sl-Latn-IT-rozaj"
    assert t.lookup_chain() == ["sl-Latn-IT-rozaj", "sl-Latn-IT", "sl-Latn", "sl"]
    assert str(t) == "sl-Latn-IT-rozaj-u-ca-gregory-x-priv"
    assert ll.canonicalize("en-b-aa-a-bb") == "en-a-bb-b-aa"


def test_errors():
    e = ll.LocaleError("INVALID_SUBTAG", "$")
    assert str(e) == "INVALID_SUBTAG: $" and e.detail == "$"
    assert str(ll.LocaleError("EMPTY")) == "EMPTY"
    assert isinstance(e, ValueError)


def test_numeric_inputs():
    assert ll.to_decimal_string(1.5) == "1.5"
    assert ll.to_decimal_string(-0.0) == "-0.0"
    assert ll.to_decimal_string(float("nan")) == "NaN"
    assert ll.to_decimal_string(float("inf")) == "Infinity"
    assert ll.to_decimal_string(float("-inf")) == "-Infinity"
    assert ll.to_decimal_string(Decimal("NaN")) == "NaN"
    assert ll.to_decimal_string(Decimal("-Infinity")) == "-Infinity"
    assert ll.to_decimal_string(10 ** 25) == "1" + "0" * 25
    assert code(lambda: ll.to_decimal_string(True)) == "BAD_NUMBER"
    assert code(lambda: ll.to_decimal_string([1])) == "BAD_NUMBER"
    assert ll.format_number("en", 0.1 + 0.2) == "0.3"
    assert ll.format_number("en", 1e21) == "1,000,000,000,000,000,000,000"
    assert ll.format_number("en", -0.0) == "-0"
    assert ll.format_number("en", Decimal("1.50"), min_fraction=2) == "1.50"
    assert ll.plural_category("en", 1) == "one"
    assert ll.ordinal_category("en", 2) == "two"
    assert code(lambda: ll.plural_category("en", float("nan"))) == "BAD_NUMBER"
    assert code(lambda: ll.format_number("en", 1, style="scientific")) == "BAD_OPTION"
    assert code(lambda: ll.format_number("en", 1, min_fraction=True)) == "BAD_OPTION"


def test_dates():
    assert ll.parse_date("2026-10-02") == (2026, 10, 2)
    assert ll.weekday(2026, 10, 2) == 5
    assert ll.format_date("en", "2026-10-02") == "Oct 2, 2026"


def test_message_arguments():
    assert ll.format_message("en", "{a} {b} {c} {d}", {"a": 1234.5, "b": 7, "c": "x", "d": ll.Num("1.50")}) == "1,234.5 7 x 1.5"
    assert ll.format_message("en", "plain") == "plain"
    assert ll.format_message("en", "{n, select, 1{one} other{x}}", {"n": 1}) == "one"
    assert code(lambda: ll.format_message("en", "{n, plural, other{#}}", {"n": True})) == "BAD_ARGUMENT"
    assert code(lambda: ll.format_message("en", "{n, plural, other{#}", {"n": 1})) == "SYNTAX"
    assert code(lambda: ll.format_message("en", "{n, plural, other x}", {"n": 1})) == "SYNTAX"
    assert code(lambda: ll.format_message("en", "{n, plural,", {"n": 1})) == "SYNTAX"
    assert code(lambda: ll.format_message("en", "{n, plural, offset:x other{#}}", {"n": 1})) == "SYNTAX"
    assert ll.format_message("en", "{n, plural, offset: 1 other{#}}", {"n": 3}) == "2"
    assert code(lambda: ll.format_message("en", "{n, date, huge}", {"n": "x"})) == "SYNTAX"
    assert code(lambda: ll.format_message("en", "{n", {})) == "SYNTAX"
    assert ll.format_message("en", "it's '{' unterminated") == "it's { unterminated"
    assert ll.format_message("en", "'{a''b}'") == "{a'b}"


def test_catalogs():
    assert ll.parse_catalog('{"b": "2", "a": "1"}') == {"b": "2", "a": "1"}
    for bad in ['{"a": tru}', '{"a": -}', '{"a": 1.}', '{"a" "b"}', '{"a": "\\x"}', '{"a": "\\u12"}', '{"a": "\\udc00"}',
                '{"a": "\\ud800\\u0041"}', '{"a": [1 2]}', '{"a": "\ud800"}', "[" * 100 + "]" * 100, '{"a": "x"']:
        assert code(lambda: ll.parse_catalog(bad)) == "BAD_JSON", bad
    assert code(lambda: ll.parse_catalog('{"a": {}, "b": []}')) == "NON_STRING_VALUE"
    assert ll.parse_catalog('{"a": "\\b\\f\\r\\/\\"\\\\"}')["a"] == '\b\f\r/"\\'
    m = ll.resolve("id", {"c": "{n, plural, other{# hal}}"}, "C", "c", {"n": 2000})
    assert (m.code, m.message_id, m.text, m.fallback) == ("C", "c", "2.000 hal", False)
    assert ll.resolve("id", {}, "X", "y").text == "!!y!!"


def test_constants():
    assert ll.CLDR_VERSION == "47.0.0" and ll.MAX_DEPTH == 16 and ll.MAX_EXPONENT == 1000
    assert ll.__version__ == "0.2.0"
