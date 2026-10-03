"""Runs every case of vectors/lomboklocale-vectors-v1.json."""
import json
from pathlib import Path

import pytest

import lomboklocale as ll

DOC = json.loads((Path(__file__).resolve().parents[2] / "vectors" / "lomboklocale-vectors-v1.json").read_text(encoding="utf-8"))


def _args(a):
    return {k: (v["s"] if "s" in v else ll.Num(v["n"])) for k, v in a.items()}


def _run(kind, i):
    if kind == "parse":
        return ll.canonicalize(i["tag"])
    if kind == "negotiate":
        return ll.negotiate(i["requested"], i["available"], i["default"])
    if kind == "dataLocale":
        return ll.data_locale(i["tag"])
    if kind == "plural":
        f = ll.ordinal_category if i.get("ordinal") else ll.plural_category
        return f(i["locale"], i["value"])
    if kind == "number":
        return ll.format_number(i["locale"], i["value"], i.get("style", "decimal"), i.get("currency"),
                                i.get("minFraction"), i.get("maxFraction"))
    if kind == "date":
        return ll.format_date(i["locale"], i["date"], i["style"])
    if kind == "message":
        return ll.format_message(i["locale"], i["pattern"], _args(i["args"]))
    if kind == "catalog":
        return ll.parse_catalog(i["json"])
    if kind == "resolve":
        m = ll.resolve(i["locale"], ll.parse_catalog(i["catalog"]), i["code"], i["messageId"], _args(i["args"]))
        return {"code": m.code, "messageId": m.message_id, "text": m.text, "fallback": m.fallback}
    raise AssertionError(kind)


def test_enough_cases():
    assert len(DOC["cases"]) >= 100


@pytest.mark.parametrize("case", DOC["cases"], ids=[c["id"] for c in DOC["cases"]])
def test_vector(case):
    try:
        got = {"ok": _run(case["kind"], case["input"])}
    except ll.LocaleError as e:
        got = {"error": e.code}
    assert got == case["expected"]
