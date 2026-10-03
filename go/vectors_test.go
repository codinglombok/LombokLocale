package lomboklocale

import (
	"encoding/json"
	"errors"
	"os"
	"reflect"
	"testing"
)

type vcase struct {
	ID       string          `json:"id"`
	Kind     string          `json:"kind"`
	Input    map[string]any  `json:"input"`
	Expected json.RawMessage `json:"expected"`
}

func vargs(m map[string]any) map[string]Arg {
	out := map[string]Arg{}
	for k, v := range m {
		a := v.(map[string]any)
		if s, ok := a["s"]; ok {
			out[k] = Str(s.(string))
		} else {
			out[k] = Num(a["n"].(string))
		}
	}
	return out
}

func strs(v any) []string {
	var out []string
	for _, x := range v.([]any) {
		out = append(out, x.(string))
	}
	return out
}

func runCase(kind string, i map[string]any) (any, error) {
	s := func(k string) string { v, _ := i[k].(string); return v }
	switch kind {
	case "parse":
		return Canonicalize(s("tag"))
	case "negotiate":
		return Negotiate(strs(i["requested"]), strs(i["available"]), s("default")), nil
	case "dataLocale":
		return DataLocale(s("tag")), nil
	case "plural":
		if o, _ := i["ordinal"].(bool); o {
			return OrdinalCategory(s("locale"), s("value"))
		}
		return PluralCategory(s("locale"), s("value"))
	case "number":
		opts := NumberOptions{Style: s("style"), Currency: s("currency")}
		if v, ok := i["minFraction"].(float64); ok {
			opts.MinFraction = Digits(int(v))
		}
		if v, ok := i["maxFraction"].(float64); ok {
			opts.MaxFraction = Digits(int(v))
		}
		return FormatNumber(s("locale"), s("value"), opts)
	case "date":
		return FormatDate(s("locale"), s("date"), s("style"))
	case "message":
		return FormatMessage(s("locale"), s("pattern"), vargs(i["args"].(map[string]any)))
	case "catalog":
		return ParseCatalog(s("json"))
	case "resolve":
		cat, err := ParseCatalog(s("catalog"))
		if err != nil {
			return nil, err
		}
		m := Resolve(s("locale"), cat, s("code"), s("messageId"), vargs(i["args"].(map[string]any)))
		return map[string]any{"code": m.Code, "messageId": m.MessageID, "text": m.Text, "fallback": m.Fallback}, nil
	}
	panic(kind)
}

func TestVectors(t *testing.T) {
	raw, err := os.ReadFile("../vectors/lomboklocale-vectors-v1.json")
	if err != nil {
		t.Fatal(err)
	}
	var doc struct{ Cases []vcase }
	if err := json.Unmarshal(raw, &doc); err != nil {
		t.Fatal(err)
	}
	if len(doc.Cases) < 100 {
		t.Fatal("too few cases")
	}
	for _, c := range doc.Cases {
		got, err := runCase(c.Kind, c.Input)
		var res map[string]any
		var le *Error
		if errors.As(err, &le) {
			res = map[string]any{"error": le.Code}
		} else if err != nil {
			t.Fatalf("%s: %v", c.ID, err)
		} else {
			res = map[string]any{"ok": got}
		}
		b, _ := json.Marshal(res)
		var gotN, wantN any
		_ = json.Unmarshal(b, &gotN)
		_ = json.Unmarshal(c.Expected, &wantN)
		if !reflect.DeepEqual(gotN, wantN) {
			t.Errorf("%s: got %s want %s", c.ID, b, c.Expected)
		}
	}
}
