#!/usr/bin/env python3
"""Author vector INPUTS for LombokLocale. Expected outputs ("out"/"err") are filled by the
Rust reference: `LOMBOK_REGEN=1 cargo test --test vectors` (in rust/). Deterministic (fixed seed)."""
import json, os, random
C = []
def add(fn, inp): C.append({"fn": fn, "in": inp})

# --- BCP-47 / negotiate
for t in ["id","en-US","zh-Hant-TW","EN_us","sr-Latn-RS","es-419","de-CH-1996","","  ","1","abcdefghi","en--US","en-","-en","x","en-US-x","zh-hans","ZH-HANT-tw","i-klingon","en-US-posix"]:
    add("parse_bcp47", t)
NEG = [
 (["id-ID","en"],["en","id"],"en"),(["en-GB"],["en","id"],"id"),(["fr-FR"],["en","id"],"en"),
 (["fr","ja-JP"],["id","ja"],"en"),(["zh-Hant-TW"],["zh","zh-TW"],"en"),(["zh-Hant-TW"],["zh-Hant","zh"],"en"),
 (["EN-us"],["en-US"],"id"),([],["en"],"en"),(["!!"],["en"],"en"),(["pt-BR"],["pt","pt-BR"],"en"),
]
for r,a,d in NEG: add("negotiate", {"requested": r, "available": a, "default": d})

# --- plural / ordinal
LOC = ["en","en-US","id","ja","zh","ko","vi","fr","pt","de","ru","uk","pl","ar","cy","it","sv","hu","tr","xx"]
NS = [0,1,2,3,4,5,6,7,8,10,11,12,13,14,20,21,22,23,25,100,101,102,111,112,113,1000,1001]
for l in LOC:
    for n in NS: add("plural_category", {"locale": l, "n": n})
for l in ["en","fr","it","sv","hu","id","de","xx"]:
    for n in [0,1,2,3,4,5,8,11,12,13,21,22,23,80,101,111,800]: add("ordinal_category", {"locale": l, "n": n})

# --- numbers
for l in ["en","id","fr","de","ja","xx","en-US","pl"]:
    for v in [0,1,12,123,1234,12345,123456,1234567,-1,-1234,-1234567,9007199254740991,"-9223372036854775808","9223372036854775807"]:  # >2^53 given as strings (JSON numbers lose precision in JS)
        add("format_integer", {"value": v, "locale": l})
FL = [0,0.5,1.005,1234.5,-1234.5,0.004,-0.004,-0.005,999.995,1e15,1e19,1e20,1e30,-1.5e25,123456789.123456789,0.1,0.2,2.5,-0.0,"NaN","Infinity","-Infinity"]
for l in ["en","id","fr"]:
    for v in FL:
        for d in [0,2,15,40]: add("format_float", {"value": v, "decimals": d, "locale": l})
for cur in ["IDR","USD","EUR","JPY","XXX"]:
    for l in ["id","en","fr","de","ja"]:
        for v in [0,1,15000,1234.5,-99.99]: add("format_currency", {"value": v, "code": cur, "locale": l})

# --- dates
for l in ["id","en","fr"]:
    for st in ["iso","short","long"]:
        for (y,m,d) in [(2026,9,27),(2026,1,5),(1999,12,31),(5,3,4),(2026,13,1),(2026,0,1),(2026,2,30)]:
            add("format_date", {"year": y, "month": m, "day": d, "locale": l, "style": st})
for s in ["2026-09-27","2026-9-7","2026/09/27","2026-13-01","2026-00-10","abc","","2026-09","2026-09-27-01","-2026-09-27","2026-09-32","0001-01-01","2026-09-27 "]:
    add("parse_iso_date", s)

# --- messages
P1 = "{count, plural, one{# file} other{# files}}"
MSG = [
 ("en","Halo, {name}!",{"name":"Lombok"}),("en","Hi {name",{}),("en","Hi }",{}),("en","Hi {who}",{"name":"x"}),
 ("en",P1,{"count":1}),("en",P1,{"count":5}),("en",P1,{"count":0}),("id",P1,{"count":1}),("ru","{n, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}",{"n":21}),
 ("ru","{n, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}",{"n":3}),("ru","{n, plural, one{# файл} few{# файла} many{# файлов} other{# файла}}",{"n":11}),
 ("ar","{n, plural, zero{z} one{o} two{t} few{f} many{m} other{x}}",{"n":0}),("ar","{n, plural, zero{z} one{o} two{t} few{f} many{m} other{x}}",{"n":2}),("ar","{n, plural, zero{z} one{o} two{t} few{f} many{m} other{x}}",{"n":7}),
 ("en","{g, select, male{He} female{She} *{They}}",{"g":"male"}),("en","{g, select, male{He} female{She} *{They}}",{"g":"zz"}),("en","{g, select, male{He} other{Other}}",{"g":"zz"}),
 ("en","{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}",{"n":1}),("en","{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}",{"n":12}),("en","{n, selectordinal, one{#st} two{#nd} few{#rd} other{#th}}",{"n":23}),
 ("en","{n, plural, one{# item} other{# items}}",{"n":"x"}),("en","{n, plural, one{# item}}",{"n":2}),("en","{n, bogus, a{b}}",{"n":2}),
 ("en","{n}",{"n":-5}),("en","{n}",{"n":1.5}),("en","{n, plural, one{# a} other{# b}}",{"n":-1}),
 ("en","You have {count, plural, one{# message from {name}} other{# messages from {name}}}.",{"count":3,"name":"Ann"}),
 ("en","{a}{b}{a}",{"a":"x","b":"y"}),("en","",{}),("en","no braces",{}),("en","{  name  }",{"name":"sp"}),
 ("en","{g, select, a{{h, select, x{AX} *{A?}}} *{other}}",{"g":"a","h":"x"}),("en","{g, select, a{{h, select, x{AX} *{A?}}} *{other}}",{"g":"a","h":"q"}),
 ("en","é日\u0000{x}",{"x":"é"}),
]
for l,p,a in MSG: add("format_message", {"locale": l, "pattern": p, "args": a})
deep = "x"
for _ in range(20): deep = "{n, select, *{" + deep + "}}"
add("format_message", {"locale":"en","pattern":deep,"args":{"n":1}})

# --- catalogs
CAT = ['{"a":"\\u+041"}','{"a":"\\u00zz"}','{"a":"\\u0041\\u00e9"}','{"a":"\\ud800"}','{"a":"\\u12"}','{"a": "b"}','{}','  {  "a" : "x\\ny\\"z\\\\" , "b":"\\u00e9\\u65e5" }','[1]','{"a": 1}','{"a": "b",}','{"a" "b"}','{"a": "b"','','{"a":"\\q"}','{"k":"{n, plural, one{#} other{#s}}"}','{"a":"b"} x',"{'a':'b'}"]
for c in CAT: add("parse_catalog", c)
for pat,args in [("{n, plural, one{# msg} other{# msgs}}",{"n":2}),("Hi {name}",{"name":"A"}),("Bad {",{}),("plain",{})]:
    add("resolve", {"locale":"en","catalog":{"k":pat},"code":"CODE","messageId":"k","args":args})
add("resolve", {"locale":"en","catalog":{"k":"v"},"code":"CODE","messageId":"missing","args":{}})

# --- deterministic soup for message/bcp47/catalog parity
rng = random.Random(20260928)
PIECES = ["{","}",","," ","plural","select","selectordinal","one","other","few","*","#","n","name","x","\\","\"","en","id","-","_","US","Hans","zh","é","日","1","0"]
def soup(k): return "".join(rng.choice(PIECES) for _ in range(rng.randint(0,k)))
for _ in range(250):
    add("format_message", {"locale": rng.choice(["en","id","ru","ar","xx"]), "pattern": soup(24), "args": {"n": rng.choice([0,1,2,3,11,21,100]), "name": "z", "x": 1.5}})
for _ in range(120): add("parse_bcp47", soup(8))
for _ in range(120): add("parse_catalog", soup(12))
json.dump({"suite": "lomboklocale", "version": 1, "cases": C}, open(os.path.join(os.path.dirname(os.path.abspath(__file__)),"lomboklocale-vectors-v1.json"),"w",encoding="utf-8"), ensure_ascii=False, indent=0)
print(len(C), "cases")
