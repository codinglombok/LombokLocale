<?php

declare(strict_types=1);

namespace LombokLocale;

/** @internal ICU MessageFormat subset parser (SPEC section 7). */
final class MessageParser
{
    private int $i = 0;

    /** @param list<string> $s characters of the pattern */
    public function __construct(private readonly array $s)
    {
    }

    private function syntax(): LocaleError
    {
        return new LocaleError('SYNTAX', (string) $this->i);
    }

    private function at(int $k): string
    {
        return $this->s[$k] ?? '';
    }

    /** @return list<array<int, mixed>> */
    public function message(int $depth, bool $inPlural, bool $top): array
    {
        if ($depth > Locale::MAX_DEPTH) {
            throw new LocaleError('TOO_DEEP');
        }
        $n = count($this->s);
        $parts = [];
        $buf = '';
        while ($this->i < $n) {
            $c = $this->s[$this->i];
            if ($c === "'") {
                $next = $this->at($this->i + 1);
                if ($next === "'") {
                    $buf .= "'";
                    $this->i += 2;
                } elseif (in_array($next, ['{', '}', '|'], true) || ($next === '#' && $inPlural)) {
                    $this->i++;
                    while ($this->i < $n) {
                        if ($this->s[$this->i] === "'") {
                            if ($this->at($this->i + 1) === "'") {
                                $buf .= "'";
                                $this->i += 2;
                                continue;
                            }
                            $this->i++;
                            break;
                        }
                        $buf .= $this->s[$this->i++];
                    }
                } else {
                    $buf .= "'";
                    $this->i++;
                }
            } elseif ($c === '{') {
                if ($buf !== '') {
                    $parts[] = ['text', $buf];
                    $buf = '';
                }
                $this->i++;
                $parts[] = $this->argument($depth, $inPlural);
            } elseif ($c === '}') {
                if ($top) {
                    throw $this->syntax();
                }
                break;
            } elseif ($c === '#' && $inPlural) {
                if ($buf !== '') {
                    $parts[] = ['text', $buf];
                    $buf = '';
                }
                $parts[] = ['pound'];
                $this->i++;
            } else {
                $buf .= $c;
                $this->i++;
            }
        }
        if ($buf !== '') {
            $parts[] = ['text', $buf];
        }
        return $parts;
    }

    private function ws(): void
    {
        while (in_array($this->at($this->i), [' ', "\t", "\r", "\n"], true)) {
            $this->i++;
        }
    }

    private function word(): string
    {
        $this->ws();
        $out = '';
        while ($this->i < count($this->s) && !in_array($this->s[$this->i], [' ', "\t", "\r", "\n", ',', '{', '}'], true)) {
            $out .= $this->s[$this->i++];
        }
        return $out;
    }

    private function expect(string $c): void
    {
        $this->ws();
        if ($this->at($this->i) !== $c) {
            throw $this->syntax();
        }
        $this->i++;
    }

    /** @return array<int, mixed> */
    private function argument(int $depth, bool $inPlural): array
    {
        $name = $this->word();
        if (!preg_match('/^[A-Za-z0-9_]+$/D', $name)) {
            throw $this->syntax();
        }
        $this->ws();
        if ($this->i >= count($this->s)) {
            throw $this->syntax();
        }
        if ($this->s[$this->i] === '}') {
            $this->i++;
            return ['simple', $name];
        }
        $this->expect(',');
        $kind = $this->word();
        if ($kind === 'number' || $kind === 'date') {
            $this->ws();
            $style = null;
            if ($this->at($this->i) === ',') {
                $this->i++;
                $style = $this->word();
                $allowed = $kind === 'number' ? ['integer', 'percent'] : ['full', 'long', 'medium', 'short'];
                if (!in_array($style, $allowed, true)) {
                    throw $this->syntax();
                }
            }
            $this->expect('}');
            return [$kind, $name, $kind === 'date' ? ($style ?? 'medium') : $style];
        }
        if (!in_array($kind, ['plural', 'selectordinal', 'select'], true)) {
            throw $this->syntax();
        }
        $this->expect(',');
        $offset = 0;
        $seenOffset = false;
        $cases = [];
        while (true) {
            $this->ws();
            if ($this->i >= count($this->s)) {
                throw $this->syntax();
            }
            if ($this->s[$this->i] === '}') {
                $this->i++;
                break;
            }
            $key = $this->word();
            if (str_starts_with($key, 'offset:') && $kind === 'plural' && $cases === [] && !$seenOffset) {
                $rest = substr($key, 7);
                if ($rest === '') {
                    $rest = $this->word();
                }
                if (!preg_match('/^[0-9]+$/D', $rest) || strlen($rest) > 18) {
                    throw $this->syntax();
                }
                $offset = (int) $rest;
                $seenOffset = true;
                continue;
            }
            if ($key === '' || array_key_exists($key, $cases)) {
                throw $this->syntax();
            }
            if ($kind === 'select') {
                $valid = (bool) preg_match('/^[A-Za-z0-9_]+$/D', $key);
            } elseif (str_starts_with($key, '=')) {
                $valid = (bool) preg_match('/^=[0-9]+(\.[0-9]+)?$/D', $key);
            } else {
                $valid = in_array($key, ['zero', 'one', 'two', 'few', 'many', 'other'], true);
            }
            if (!$valid) {
                throw $this->syntax();
            }
            $this->expect('{');
            $body = $this->message($depth + 1, $kind !== 'select' || $inPlural, false);
            if ($this->at($this->i) !== '}') {
                throw $this->syntax();
            }
            $this->i++;
            $cases[$key] = $body;
        }
        if (!array_key_exists('other', $cases)) {
            throw new LocaleError('MISSING_OTHER', $name);
        }
        return ['choice', $kind, $name, $offset, $cases];
    }
}
