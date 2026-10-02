<?php

declare(strict_types=1);

namespace LombokLocale;

/** @internal Strict RFC 8259 reader used by catalog parsing. */
final class JsonReader
{
    public int $i = 0;
    private const MAX_DEPTH = 64;

    public function __construct(private readonly string $s)
    {
    }

    public function bad(): LocaleError
    {
        return new LocaleError('BAD_JSON', (string) $this->i);
    }

    public function ws(): void
    {
        while ($this->i < strlen($this->s) && str_contains(" \t\n\r", $this->s[$this->i])) {
            $this->i++;
        }
    }

    private function hex4(): int
    {
        $h = substr($this->s, $this->i, 4);
        if (!preg_match('/^[0-9A-Fa-f]{4}$/D', $h)) {
            throw $this->bad();
        }
        $this->i += 4;
        return (int) hexdec($h);
    }

    public function string(): string
    {
        if (($this->s[$this->i] ?? '') !== '"') {
            throw $this->bad();
        }
        $this->i++;
        $out = '';
        $esc = ['"' => '"', '\\' => '\\', '/' => '/', 'b' => "\x08", 'f' => "\x0C", 'n' => "\n", 'r' => "\r", 't' => "\t"];
        $n = strlen($this->s);
        while (true) {
            if ($this->i >= $n) {
                throw $this->bad();
            }
            $c = $this->s[$this->i++];
            if ($c === '"') {
                return $out;
            }
            if ($c === '\\') {
                $e = $this->s[$this->i++] ?? '';
                if (isset($esc[$e])) {
                    $out .= $esc[$e];
                } elseif ($e === 'u') {
                    $cp = $this->hex4();
                    if ($cp >= 0xD800 && $cp < 0xDC00) {
                        if (substr($this->s, $this->i, 2) !== '\\u') {
                            throw $this->bad();
                        }
                        $this->i += 2;
                        $lo = $this->hex4();
                        if ($lo < 0xDC00 || $lo >= 0xE000) {
                            throw $this->bad();
                        }
                        $cp = 0x10000 + (($cp - 0xD800) << 10) + ($lo - 0xDC00);
                    } elseif ($cp >= 0xDC00 && $cp < 0xE000) {
                        throw $this->bad();
                    }
                    $out .= self::encode($cp);
                } else {
                    throw $this->bad();
                }
            } elseif (ord($c) < 0x20) {
                throw $this->bad();
            } else {
                $out .= $c;
            }
        }
    }

    private static function encode(int $cp): string
    {
        if ($cp < 0x80) {
            return chr($cp);
        }
        if ($cp < 0x800) {
            return chr(0xC0 | ($cp >> 6)) . chr(0x80 | ($cp & 0x3F));
        }
        if ($cp < 0x10000) {
            return chr(0xE0 | ($cp >> 12)) . chr(0x80 | (($cp >> 6) & 0x3F)) . chr(0x80 | ($cp & 0x3F));
        }
        return chr(0xF0 | ($cp >> 18)) . chr(0x80 | (($cp >> 12) & 0x3F)) . chr(0x80 | (($cp >> 6) & 0x3F)) . chr(0x80 | ($cp & 0x3F));
    }

    public function skip(int $depth): void
    {
        if ($depth > self::MAX_DEPTH) {
            throw $this->bad();
        }
        $this->ws();
        $c = $this->s[$this->i] ?? '';
        if ($c === '"') {
            $this->string();
            return;
        }
        if ($c === '{' || $c === '[') {
            $close = $c === '{' ? '}' : ']';
            $this->i++;
            $this->ws();
            if (($this->s[$this->i] ?? '') === $close) {
                $this->i++;
                return;
            }
            while (true) {
                $this->ws();
                if ($close === '}') {
                    $this->string();
                    $this->ws();
                    if (($this->s[$this->i] ?? '') !== ':') {
                        throw $this->bad();
                    }
                    $this->i++;
                }
                $this->skip($depth + 1);
                $this->ws();
                $next = $this->s[$this->i++] ?? '';
                if ($next === $close) {
                    return;
                }
                if ($next !== ',') {
                    throw $this->bad();
                }
            }
        }
        foreach (['true', 'false', 'null'] as $lit) {
            if (substr($this->s, $this->i, strlen($lit)) === $lit) {
                $this->i += strlen($lit);
                return;
            }
        }
        if (!preg_match('/-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?/A', $this->s, $m, 0, $this->i)) {
            throw $this->bad();
        }
        $this->i += strlen($m[0]);
    }
}
