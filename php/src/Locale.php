<?php

declare(strict_types=1);

namespace LombokLocale;

/**
 * BCP 47 tags and negotiation, CLDR 47 plural rules, number/percent/currency
 * and date formatting, an ICU MessageFormat subset, and JSON message
 * catalogs. Same results as the Rust, TypeScript, Python and Go ports
 * (docs/SPEC_LombokLocale_v0.2.0.md).
 */
final class Locale
{
    public const MAX_DEPTH = 16;
    public const MAX_EXPONENT = 1000;
    private const MAX_JSON_DEPTH = 64;

    /** @var array<string, mixed>|null */
    private static ?array $data = null;

    /** @return array<string, mixed> */
    private static function data(): array
    {
        return self::$data ??= json_decode(Data::JSON, true, 512, JSON_THROW_ON_ERROR);
    }

    /** CLDR release of the embedded data. */
    public static function cldrVersion(): string
    {
        return self::data()['cldr'];
    }

    // ------------------------------------------------------------ tags

    /** Parses a tag; "_" is accepted as a separator, outer spaces are ignored. */
    public static function parseTag(string $tag): LanguageTag
    {
        $t = trim($tag, " \t\n\r\0\x0B");
        if ($t === '') {
            throw new LocaleError('EMPTY');
        }
        $subs = preg_split('/[-_]/', $t);
        foreach ($subs as $s) {
            if (!preg_match('/^[A-Za-z0-9]{1,8}$/D', $s)) {
                throw new LocaleError('INVALID_SUBTAG', $s);
            }
        }
        $alpha = static fn (string $s): bool => (bool) preg_match('/^[A-Za-z]+$/D', $s);
        $lang = $subs[0];
        $len = strlen($lang);
        if (!$alpha($lang) || $len === 1 || $len === 4) {
            throw new LocaleError('INVALID_SUBTAG', $lang);
        }
        $out = new LanguageTag(strtolower($lang));
        $at = static fn (int $k): string => $subs[$k] ?? '';
        $n = count($subs);
        $i = 1;
        if (strlen($at($i)) === 4 && $alpha($at($i))) {
            $out->script = ucfirst(strtolower($at($i)));
            $i++;
        }
        if ((strlen($at($i)) === 2 && $alpha($at($i))) || (strlen($at($i)) === 3 && ctype_digit($at($i)))) {
            $out->region = strtoupper($at($i));
            $i++;
        }
        while ($i < $n && (strlen($at($i)) >= 5 || (strlen($at($i)) === 4 && ctype_digit($at($i)[0])))) {
            $v = strtolower($at($i));
            if (in_array($v, $out->variants, true)) {
                throw new LocaleError('DUPLICATE_VARIANT', $v);
            }
            $out->variants[] = $v;
            $i++;
        }
        while ($i < $n && strlen($at($i)) === 1 && strtolower($at($i)) !== 'x') {
            $single = strtolower($at($i));
            foreach ($out->extensions as $e) {
                if ($e[0] === $single) {
                    throw new LocaleError('DUPLICATE_EXTENSION', $single);
                }
            }
            $i++;
            $ext = [$single];
            while ($i < $n && strlen($at($i)) >= 2) {
                $ext[] = strtolower($at($i));
                $i++;
            }
            if (count($ext) === 1) {
                throw new LocaleError('INVALID_SUBTAG', $single);
            }
            $out->extensions[] = $ext;
        }
        if ($i < $n && strtolower($at($i)) === 'x') {
            $i++;
            if ($i === $n) {
                throw new LocaleError('INVALID_SUBTAG', 'x');
            }
            for (; $i < $n; $i++) {
                $out->privateUse[] = strtolower($subs[$i]);
            }
        }
        if ($i < $n) {
            throw new LocaleError('INVALID_SUBTAG', $at($i));
        }
        usort($out->extensions, static fn (array $a, array $b): int => strcmp(implode('-', $a), implode('-', $b)));
        return $out;
    }

    /** Canonical form of $tag, e.g. "EN_us" -> "en-US". */
    public static function canonicalize(string $tag): string
    {
        return (string) self::parseTag($tag);
    }

    private static function tryParse(string $tag): ?LanguageTag
    {
        try {
            return self::parseTag($tag);
        } catch (LocaleError) {
            return null;
        }
    }

    /**
     * RFC 4647 lookup over $requested (in order, case-insensitive); returns the
     * match as written in $available, or $default.
     *
     * @param list<string> $requested
     * @param list<string> $available
     */
    public static function negotiate(array $requested, array $available, string $default): string
    {
        $avail = [];
        foreach ($available as $a) {
            $t = self::tryParse($a);
            if ($t !== null) {
                $avail[] = [strtolower($t->base()), $a];
            }
        }
        foreach ($requested as $r) {
            $t = self::tryParse($r);
            if ($t === null) {
                continue;
            }
            foreach ($t->lookupChain() as $cand) {
                foreach ($avail as [$c, $orig]) {
                    if ($c === strtolower($cand)) {
                        return $orig;
                    }
                }
            }
        }
        return $default;
    }

    /** The CLDR data locale used to format for $tag (SPEC section 2.3). */
    public static function dataLocale(string $tag): string
    {
        $t = self::tryParse($tag);
        if ($t === null) {
            return 'en';
        }
        if ($t->language === 'zh' && $t->script === null && in_array($t->region, ['TW', 'HK', 'MO'], true)) {
            $t->script = 'Hant';
        }
        foreach ($t->lookupChain() as $cand) {
            if (isset(self::data()['locales'][$cand])) {
                return $cand;
            }
        }
        return 'en';
    }

    // ------------------------------------------------------------ decimals

    /** Text form of a number: shortest round-trip for floats. */
    public static function toDecimalString(int|float|string|Num $v): string
    {
        if ($v instanceof Num) {
            return $v->value;
        }
        if (is_string($v)) {
            return $v;
        }
        if (is_int($v)) {
            return (string) $v;
        }
        if (is_nan($v)) {
            return 'NaN';
        }
        if (is_infinite($v)) {
            return $v > 0 ? 'Infinity' : '-Infinity';
        }
        $s = var_export($v, true);
        return $s;
    }

    /** @return array{0: string, 1?: bool, 2?: string, 3?: string} */
    private static function parseDec(int|float|string|Num $value): array
    {
        $s = self::toDecimalString($value);
        if ($s === 'NaN') {
            return ['nan'];
        }
        if ($s === 'Infinity' || $s === '+Infinity' || $s === '-Infinity') {
            return ['inf', $s[0] === '-'];
        }
        if (!preg_match('/^([+-]?)([0-9]+)(?:\.([0-9]+))?(?:[eE]([+-]?[0-9]{1,4}))?$/D', $s, $m)) {
            throw new LocaleError('BAD_NUMBER', $s);
        }
        $exp = (int) ($m[4] ?? '0');
        if (abs($exp) > self::MAX_EXPONENT) {
            throw new LocaleError('BAD_NUMBER', $s);
        }
        $digits = $m[2] . ($m[3] ?? '');
        $point = strlen($m[2]) + $exp;
        if ($point <= 0) {
            $digits = str_repeat('0', 1 - $point) . $digits;
            $point = 1;
        }
        $digits = str_pad($digits, $point, '0');
        $int = ltrim(substr($digits, 0, $point), '0');
        return ['finite', $m[1] === '-', $int === '' ? '0' : $int, (string) substr($digits, $point)];
    }

    /** @return array{string, string} */
    private static function round(string $int, string $frac, int $max): array
    {
        if (strlen($frac) <= $max) {
            return [$int, $frac];
        }
        $keep = $int . substr($frac, 0, $max);
        if ($frac[$max] >= '5') {
            $k = strlen($keep) - 1;
            while (true) {
                if ($k < 0) {
                    $keep = '1' . $keep;
                    break;
                }
                if ($keep[$k] === '9') {
                    $keep[$k] = '0';
                    $k--;
                    continue;
                }
                $keep[$k] = (string) ((int) $keep[$k] + 1);
                break;
            }
        }
        $split = strlen($keep) - $max;
        $ip = ltrim(substr($keep, 0, $split), '0');
        return [$ip === '' ? '0' : $ip, (string) substr($keep, $split)];
    }

    // ------------------------------------------------------------ plural

    /** @return array{bool, ?int} */
    private static function operand(string $int, string $frac, string $op, int $mod): array
    {
        $trimmed = rtrim($frac, '0');
        $num = static function (string $s) use ($mod): ?int {
            if ($mod > 0) {
                $r = 0;
                for ($i = 0, $n = strlen($s); $i < $n; $i++) {
                    $r = ($r * 10 + ord($s[$i]) - 48) % $mod;
                }
                return $r;
            }
            return strlen($s) > 18 ? null : (int) $s;
        };
        $small = static fn (int $n): int => $mod > 0 ? $n % $mod : $n;
        return match ($op) {
            'n' => [trim($frac, '0') === '', $num($int)],
            'i' => [true, $num($int)],
            'v' => [true, $small(strlen($frac))],
            'w' => [true, $small(strlen($trimmed))],
            'f' => [true, $num($frac === '' ? '0' : $frac)],
            't' => [true, $num($trimmed === '' ? '0' : $trimmed)],
            default => [true, 0],
        };
    }

    /** @param array<string, mixed> $table */
    private static function select(array $table, string $locale, int|float|string|Num $value): string
    {
        $d = self::parseDec($value);
        if ($d[0] !== 'finite') {
            throw new LocaleError('BAD_NUMBER', self::toDecimalString($value));
        }
        $rules = null;
        $t = self::tryParse($locale);
        if ($t !== null) {
            foreach ($t->lookupChain() as $cand) {
                if (isset($table[$cand])) {
                    $rules = $table[$cand];
                    break;
                }
            }
        }
        $rules ??= $table['root'] ?? [];
        foreach ($rules as [$cat, $groups]) {
            foreach ($groups as $rels) {
                $ok = true;
                foreach ($rels as [$op, $mod, $neg, $ranges]) {
                    [$isInt, $val] = self::operand($d[2], $d[3], $op, $mod);
                    $inside = false;
                    if ($isInt && $val !== null) {
                        foreach ($ranges as [$lo, $hi]) {
                            if ($lo <= $val && $val <= $hi) {
                                $inside = true;
                                break;
                            }
                        }
                    }
                    if ($inside === ($neg === 1)) {
                        $ok = false;
                        break;
                    }
                }
                if ($ok) {
                    return $cat;
                }
            }
        }
        return 'other';
    }

    /** Cardinal plural category; decimal strings keep trailing zeros as visible digits. */
    public static function pluralCategory(string $locale, int|float|string|Num $value): string
    {
        return self::select(self::data()['plurals'], $locale, $value);
    }

    /** Ordinal plural category (1st, 2nd, 3rd, ...). */
    public static function ordinalCategory(string $locale, int|float|string|Num $value): string
    {
        return self::select(self::data()['ordinals'], $locale, $value);
    }

    // ------------------------------------------------------------ numbers

    /** @return array{string, string, string} */
    private static function splitPattern(string $p): array
    {
        $quoted = false;
        $start = -1;
        $end = 0;
        for ($i = 0, $n = strlen($p); $i < $n; $i++) {
            $c = $p[$i];
            if ($c === "'") {
                $quoted = !$quoted;
            } elseif (!$quoted && str_contains('#0,.', $c)) {
                if ($start < 0) {
                    $start = $i;
                }
                $end = $i + 1;
            }
        }
        if ($start < 0) {
            $start = $end = strlen($p);
        }
        return [substr($p, 0, $start), substr($p, $start, $end - $start), substr($p, $end)];
    }

    /** @param array<string, mixed> $d */
    private static function expandAffix(string $raw, array $d, string $symbol): string
    {
        $chars = preg_split('//u', $raw, -1, PREG_SPLIT_NO_EMPTY);
        $out = '';
        $quoted = false;
        for ($i = 0, $n = count($chars); $i < $n; $i++) {
            $c = $chars[$i];
            if ($c === "'") {
                if (($chars[$i + 1] ?? '') === "'") {
                    $out .= "'";
                    $i++;
                    continue;
                }
                $quoted = !$quoted;
            } elseif ($quoted) {
                $out .= $c;
            } elseif ($c === '¤') {
                $out .= $symbol;
            } elseif ($c === '%') {
                $out .= $d['percent'];
            } elseif ($c === '-') {
                $out .= $d['minus'];
            } else {
                $out .= $c;
            }
        }
        return $out;
    }

    private static function groupInt(string $ip, int $primary, int $secondary, int $minGrouping, string $sep): string
    {
        if ($primary === 0 || strlen($ip) < $primary + $minGrouping) {
            return $ip;
        }
        $head = substr($ip, 0, -$primary);
        $parts = [substr($ip, -$primary)];
        while (strlen($head) > $secondary) {
            array_unshift($parts, substr($head, -$secondary));
            $head = substr($head, 0, -$secondary);
        }
        if ($head !== '') {
            array_unshift($parts, $head);
        }
        return implode($sep, $parts);
    }

    /**
     * Formats a number (half away from zero; CLDR grouping and symbols).
     * $style is "decimal", "percent" or "currency".
     */
    public static function formatNumber(
        string $locale,
        int|float|string|Num $value,
        string $style = 'decimal',
        ?string $currency = null,
        ?int $minFraction = null,
        ?int $maxFraction = null,
    ): string {
        $data = self::data();
        $dl = self::dataLocale($locale);
        $d = $data['locales'][$dl];
        $code = '';
        if ($style === 'currency') {
            if ($currency === null || !preg_match('/^[A-Za-z]{3}$/D', $currency)) {
                throw new LocaleError('BAD_OPTION', 'currency');
            }
            $code = strtoupper($currency);
            $dmin = $dmax = $data['currencyDigits'][$code] ?? 2;
            $pattern = $d['currencyPattern'];
        } elseif ($style === 'percent') {
            [$dmin, $dmax, $pattern] = [0, 0, $d['percentPattern']];
        } elseif ($style === 'decimal') {
            [$dmin, $dmax, $pattern] = [0, 3, $d['decimalPattern']];
        } else {
            throw new LocaleError('BAD_OPTION', 'style');
        }
        $minf = $minFraction;
        $maxf = $maxFraction;
        if ($minf !== null && $maxf === null) {
            $maxf = max($minf, $dmax);
        }
        if ($maxf !== null && $minf === null) {
            $minf = min($dmin, $maxf);
        }
        $minf ??= $dmin;
        $maxf ??= $dmax;
        if ($minf < 0 || $maxf > 20 || $minf > $maxf) {
            throw new LocaleError('BAD_OPTION', 'fractionDigits');
        }
        $semi = strpos($pattern, ';');
        $pos = $semi === false ? $pattern : substr($pattern, 0, $semi);
        $negp = $semi === false ? null : substr($pattern, $semi + 1);
        [$prefix, $numpart, $suffix] = self::splitPattern($pos);
        $v = self::parseDec($value);
        if ($v[0] === 'nan') {
            [$neg, $body] = [false, $d['nan']];
        } elseif ($v[0] === 'inf') {
            [$neg, $body] = [$v[1], $d['infinity']];
        } else {
            [, $neg, $ip, $fp] = $v;
            if ($style === 'percent') {
                $fp .= '00';
                $ip = ltrim($ip . substr($fp, 0, 2), '0');
                $ip = $ip === '' ? '0' : $ip;
                $fp = (string) substr($fp, 2);
            }
            [$ip, $fp] = self::round($ip, $fp, $maxf);
            $fp = str_pad($fp, $minf, '0');
            while (strlen($fp) > $minf && str_ends_with($fp, '0')) {
                $fp = substr($fp, 0, -1);
            }
            $groups = explode(',', explode('.', $numpart)[0]);
            $g = count($groups);
            $primary = $g > 1 ? strlen($groups[$g - 1]) : 0;
            $secondary = $g > 2 ? strlen($groups[$g - 2]) : $primary;
            $body = self::groupInt($ip, $primary, $secondary, $d['minGrouping'], $d['group']);
            if ($fp !== '') {
                $body .= $d['decimal'] . $fp;
            }
        }
        if ($neg && $negp !== null) {
            [$np, , $ns] = self::splitPattern($negp);
        } elseif ($neg) {
            [$np, $ns] = ['-' . $prefix, $suffix];
        } else {
            [$np, $ns] = [$prefix, $suffix];
        }
        if ($style === 'currency') {
            [$sym, $firstSZ, $lastSZ] = $d['currencySymbols'][$code] ?? [$code, 0, 0];
            $pre = self::expandAffix($np, $d, $sym);
            $suf = self::expandAffix($ns, $d, $sym);
            if (str_ends_with($np, '¤') && !$lastSZ && $body !== '' && ctype_digit($body[0])) {
                $pre .= "\u{a0}";
            }
            if (str_starts_with($ns, '¤') && !$firstSZ && $body !== '' && ctype_digit($body[strlen($body) - 1])) {
                $suf = "\u{a0}" . $suf;
            }
            return $pre . $body . $suf;
        }
        return self::expandAffix($np, $d, '') . $body . self::expandAffix($ns, $d, '');
    }

    // ------------------------------------------------------------ dates

    private static function daysIn(int $y, int $m): int
    {
        if ($m === 2) {
            return ($y % 4 === 0 && ($y % 100 !== 0 || $y % 400 === 0)) ? 29 : 28;
        }
        return in_array($m, [4, 6, 9, 11], true) ? 30 : 31;
    }

    /** @return array{int, int, int} Parses YYYY-MM-DD (0001-01-01 to 9999-12-31). */
    public static function parseDate(string $s): array
    {
        if (!preg_match('/^([0-9]{4})-([0-9]{2})-([0-9]{2})$/D', $s, $m)) {
            throw new LocaleError('BAD_DATE', $s);
        }
        [$y, $mo, $d] = [(int) $m[1], (int) $m[2], (int) $m[3]];
        if ($y < 1 || $mo < 1 || $mo > 12 || $d < 1 || $d > self::daysIn($y, $mo)) {
            throw new LocaleError('BAD_DATE', $s);
        }
        return [$y, $mo, $d];
    }

    /** Day of the week of a valid date, 0 = Sunday. */
    public static function weekday(int $y, int $m, int $d): int
    {
        $p = $y - 1;
        $days = $p * 365 + intdiv($p, 4) - intdiv($p, 100) + intdiv($p, 400);
        for ($k = 1; $k < $m; $k++) {
            $days += self::daysIn($y, $k);
        }
        return ($days + $d - 1 + 1) % 7;
    }

    /** Formats YYYY-MM-DD with the CLDR pattern of $style (Gregorian calendar). */
    public static function formatDate(string $locale, string $date, string $style = 'medium'): string
    {
        if (!in_array($style, ['full', 'long', 'medium', 'short'], true)) {
            throw new LocaleError('BAD_OPTION', 'style');
        }
        [$y, $mo, $dd] = self::parseDate($date);
        $d = self::data()['locales'][self::dataLocale($locale)];
        $p = preg_split('//u', $d['dateFormats'][$style], -1, PREG_SPLIT_NO_EMPTY);
        $n = count($p);
        $out = '';
        $i = 0;
        while ($i < $n) {
            $c = $p[$i];
            if ($c === "'") {
                $j = $i + 1;
                while ($j < $n) {
                    if ($p[$j] === "'") {
                        if (($p[$j + 1] ?? '') === "'") {
                            $out .= "'";
                            $j += 2;
                            continue;
                        }
                        break;
                    }
                    $out .= $p[$j];
                    $j++;
                }
                if ($j === $i + 1) {
                    $out .= "'";
                }
                $i = $j + 1;
                continue;
            }
            if (preg_match('/^[A-Za-z]$/D', $c)) {
                $j = $i;
                while ($j < $n && $p[$j] === $c) {
                    $j++;
                }
                $len = $j - $i;
                $out .= match ($c) {
                    'G' => $d['era'],
                    'y' => $len === 2 ? str_pad((string) ($y % 100), 2, '0', STR_PAD_LEFT) : str_pad((string) $y, $len, '0', STR_PAD_LEFT),
                    'M', 'L' => $len <= 2 ? str_pad((string) $mo, $len, '0', STR_PAD_LEFT) : $d['months'][$len === 3 ? 0 : 1][$mo - 1],
                    'd' => str_pad((string) $dd, $len, '0', STR_PAD_LEFT),
                    'E' => $d['days'][$len === 4 ? 1 : 0][self::weekday($y, $mo, $dd)],
                };
                $i = $j;
                continue;
            }
            $out .= $c;
            $i++;
        }
        return $out;
    }

    // ------------------------------------------------------------ messages

    /**
     * Formats an ICU MessageFormat $pattern (SPEC section 7). String arguments
     * are text; int, float and Num are numbers (use `new Num('1.50')` for exact decimals).
     *
     * @param array<string, string|int|float|Num> $args
     */
    public static function formatMessage(string $locale, string $pattern, array $args = []): string
    {
        $parser = new MessageParser(preg_split('//u', $pattern, -1, PREG_SPLIT_NO_EMPTY) ?: []);
        return self::render($locale, $parser->message(0, false, true), $args, null);
    }

    /** @param string|int|float|Num $a */
    private static function argNumber(mixed $a): ?string
    {
        if (is_string($a)) {
            return null;
        }
        if ($a instanceof Num || is_int($a) || is_float($a)) {
            return self::toDecimalString($a);
        }
        return '?';
    }

    /** @return array{0: string, 1?: bool, 2?: string, 3?: string}|null */
    private static function finite(string $v): ?array
    {
        try {
            $d = self::parseDec($v);
        } catch (LocaleError) {
            return null;
        }
        return $d[0] === 'finite' ? $d : null;
    }

    private static function decEq(string $a, string $b): bool
    {
        $x = self::finite($a);
        $y = self::finite($b);
        if ($x === null || $y === null) {
            return false;
        }
        $zero = static fn (array $d): bool => $d[2] === '0' && trim($d[3], '0') === '';
        if ($zero($x) && $zero($y)) {
            return true;
        }
        return $x[1] === $y[1] && $x[2] === $y[2] && rtrim($x[3], '0') === rtrim($y[3], '0');
    }

    private static function decSub(string $value, int $offset): ?string
    {
        if ($offset === 0) {
            return $value;
        }
        [, $neg, $ip, $fp] = self::finite($value);
        if (strlen($ip . $fp) > 36) {
            return null;
        }
        // Exact arithmetic on digit strings: value * 10^scale - offset * 10^scale.
        $scale = strlen($fp);
        $a = ltrim($ip . $fp, '0');
        $b = $offset . str_repeat('0', $scale);
        $aNeg = $neg && $a !== '';
        $a = $a === '' ? '0' : $a;
        // result = (aNeg ? -a : a) - b
        if ($aNeg) {
            $mag = self::addDigits($a, $b);
            $rneg = true;
        } elseif (self::cmpDigits($a, $b) >= 0) {
            $mag = self::subDigits($a, $b);
            $rneg = false;
        } else {
            $mag = self::subDigits($b, $a);
            $rneg = true;
        }
        $mag = ltrim($mag, '0');
        $rneg = $rneg && $mag !== '';
        $s = str_pad($mag, $scale + 1, '0', STR_PAD_LEFT);
        $out = substr($s, 0, strlen($s) - $scale) . ($scale > 0 ? '.' . substr($s, strlen($s) - $scale) : '');
        return ($rneg ? '-' : '') . $out;
    }

    private static function cmpDigits(string $a, string $b): int
    {
        return strlen($a) <=> strlen($b) ?: strcmp($a, $b);
    }

    private static function addDigits(string $a, string $b): string
    {
        $out = '';
        $carry = 0;
        for ($i = strlen($a) - 1, $j = strlen($b) - 1; $i >= 0 || $j >= 0 || $carry; $i--, $j--) {
            $s = ($i >= 0 ? (int) $a[$i] : 0) + ($j >= 0 ? (int) $b[$j] : 0) + $carry;
            $out = ($s % 10) . $out;
            $carry = intdiv($s, 10);
        }
        return $out;
    }

    /** $a >= $b */
    private static function subDigits(string $a, string $b): string
    {
        $out = '';
        $borrow = 0;
        for ($i = strlen($a) - 1, $j = strlen($b) - 1; $i >= 0; $i--, $j--) {
            $s = (int) $a[$i] - ($j >= 0 ? (int) $b[$j] : 0) - $borrow;
            $borrow = $s < 0 ? 1 : 0;
            $out = ($s + 10 * $borrow) . $out;
        }
        return $out;
    }

    private static function forPlural(string $value): string
    {
        [, , $ip, $fp] = self::finite($value);
        [$ip, $fp] = self::round($ip, $fp, 3);
        $fp = rtrim($fp, '0');
        return $fp === '' ? $ip : "$ip.$fp";
    }

    /** @param array<string, mixed> $args */
    private static function lookup(array $args, string $name): mixed
    {
        if (!array_key_exists($name, $args)) {
            throw new LocaleError('MISSING_ARGUMENT', $name);
        }
        return $args[$name];
    }

    private static function fmtArg(string $locale, string $name, string $value, string $style = 'decimal', ?int $maxf = null): string
    {
        try {
            return self::formatNumber($locale, $value, $style, null, null, $maxf);
        } catch (LocaleError) {
            throw new LocaleError('BAD_ARGUMENT', $name);
        }
    }

    /**
     * @param list<array<int, mixed>> $parts
     * @param array<string, mixed> $args
     */
    private static function render(string $locale, array $parts, array $args, ?string $pound): string
    {
        $out = '';
        foreach ($parts as $p) {
            switch ($p[0]) {
                case 'text':
                    $out .= $p[1];
                    break;
                case 'pound':
                    $out .= $pound === null ? '#' : self::formatNumber($locale, $pound);
                    break;
                case 'simple':
                    $a = self::lookup($args, $p[1]);
                    $n = self::argNumber($a);
                    $out .= $n === null ? $a : self::fmtArg($locale, $p[1], $n);
                    break;
                case 'number':
                    $n = self::argNumber(self::lookup($args, $p[1]));
                    if ($n === null) {
                        throw new LocaleError('BAD_ARGUMENT', $p[1]);
                    }
                    $out .= match ($p[2]) {
                        'integer' => self::fmtArg($locale, $p[1], $n, 'decimal', 0),
                        'percent' => self::fmtArg($locale, $p[1], $n, 'percent'),
                        default => self::fmtArg($locale, $p[1], $n),
                    };
                    break;
                case 'date':
                    $a = self::lookup($args, $p[1]);
                    if (!is_string($a)) {
                        throw new LocaleError('BAD_ARGUMENT', $p[1]);
                    }
                    try {
                        $out .= self::formatDate($locale, $a, $p[2]);
                    } catch (LocaleError) {
                        throw new LocaleError('BAD_ARGUMENT', $p[1]);
                    }
                    break;
                default:
                    [, $kind, $name, $offset, $cases] = $p;
                    $a = self::lookup($args, $name);
                    if ($kind === 'select') {
                        $key = is_string($a) ? $a : self::argNumber($a);
                        $out .= self::render($locale, $cases[$key] ?? $cases['other'], $args, $pound);
                        break;
                    }
                    $n = self::argNumber($a);
                    if ($n === null || self::finite($n) === null) {
                        throw new LocaleError('BAD_ARGUMENT', $name);
                    }
                    $shown = self::decSub($n, $offset);
                    if ($shown === null) {
                        throw new LocaleError('BAD_ARGUMENT', $name);
                    }
                    $chosen = null;
                    foreach ($cases as $key => $body) {
                        $key = (string) $key;
                        if (str_starts_with($key, '=') && self::decEq(substr($key, 1), $n)) {
                            $chosen = $body;
                            break;
                        }
                    }
                    if ($chosen === null) {
                        $v = self::forPlural($shown);
                        $cat = $kind === 'selectordinal' ? self::ordinalCategory($locale, $v) : self::pluralCategory($locale, $v);
                        $chosen = $cases[$cat] ?? $cases['other'];
                    }
                    $out .= self::render($locale, $chosen, $args, $shown);
            }
        }
        return $out;
    }

    // ------------------------------------------------------------ catalogs

    /**
     * Parses a catalog: a JSON object (RFC 8259) whose values are all strings.
     *
     * @return array<string, string>
     */
    public static function parseCatalog(string $json): array
    {
        if (preg_match('//u', $json) !== 1) {
            throw new LocaleError('BAD_JSON', '0');
        }
        $check = new JsonReader($json);
        $check->skip(0);
        $check->ws();
        if ($check->i !== strlen($json)) {
            throw $check->bad();
        }
        $r = new JsonReader($json);
        $r->ws();
        if (($json[$r->i] ?? '') !== '{') {
            throw new LocaleError('NOT_OBJECT');
        }
        $r->i++;
        $out = [];
        $r->ws();
        if ($json[$r->i] === '}') {
            return $out;
        }
        while (true) {
            $r->ws();
            $key = $r->string();
            $r->ws();
            $r->i++;
            $r->ws();
            if ($json[$r->i] !== '"') {
                throw new LocaleError('NON_STRING_VALUE', $key);
            }
            $value = $r->string();
            if (array_key_exists($key, $out)) {
                throw new LocaleError('DUPLICATE_KEY', $key);
            }
            $out[$key] = $value;
            $r->ws();
            if ($json[$r->i] !== ',') {
                return $out;
            }
            $r->i++;
        }
    }

    /**
     * Looks up $messageId and formats it; a missing id or a failing pattern
     * gives "!!messageId!!".
     *
     * @param array<string, string> $catalog
     * @param array<string, string|int|float|Num> $args
     */
    public static function resolve(string $locale, array $catalog, string $code, string $messageId, array $args = []): LocalizedMessage
    {
        if (array_key_exists($messageId, $catalog)) {
            try {
                return new LocalizedMessage($code, $messageId, self::formatMessage($locale, $catalog[$messageId], $args), false);
            } catch (LocaleError) {
                // visible marker below
            }
        }
        return new LocalizedMessage($code, $messageId, "!!$messageId!!", true);
    }
}
