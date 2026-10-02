<?php

declare(strict_types=1);

// Runs the shared vectors (vectors/lomboklocale-vectors-v1.json).

require_once __DIR__ . '/bootstrap.php';

use LombokLocale\Locale;
use LombokLocale\LocaleError;
use LombokLocale\Num;

$doc = json_decode((string) file_get_contents(__DIR__ . '/../../vectors/lomboklocale-vectors-v1.json'), true, 512, JSON_THROW_ON_ERROR);

function vargs(array $a): array
{
    $out = [];
    foreach ($a as $k => $v) {
        $out[(string) $k] = array_key_exists('s', $v) ? $v['s'] : new Num($v['n']);
    }
    return $out;
}

function runCase(string $kind, array $i): mixed
{
    return match ($kind) {
        'parse' => Locale::canonicalize($i['tag']),
        'negotiate' => Locale::negotiate($i['requested'], $i['available'], $i['default']),
        'dataLocale' => Locale::dataLocale($i['tag']),
        'plural' => ($i['ordinal'] ?? false) ? Locale::ordinalCategory($i['locale'], $i['value']) : Locale::pluralCategory($i['locale'], $i['value']),
        'number' => Locale::formatNumber($i['locale'], $i['value'], $i['style'] ?? 'decimal', $i['currency'] ?? null, $i['minFraction'] ?? null, $i['maxFraction'] ?? null),
        'date' => Locale::formatDate($i['locale'], $i['date'], $i['style']),
        'message' => Locale::formatMessage($i['locale'], $i['pattern'], vargs($i['args'])),
        'catalog' => (object) Locale::parseCatalog($i['json']),
        'resolve' => (function () use ($i) {
            $m = Locale::resolve($i['locale'], Locale::parseCatalog($i['catalog']), $i['code'], $i['messageId'], vargs($i['args']));
            return ['code' => $m->code, 'messageId' => $m->messageId, 'text' => $m->text, 'fallback' => $m->fallback];
        })(),
    };
}

function canon(mixed $v): mixed
{
    if (is_object($v)) {
        $v = (array) $v;
        if ($v === []) {
            return []; // {} and [] are the same empty catalog after an associative decode
        }
        $v = array_map('canon', $v);
        ksort($v, SORT_STRING);
        return (object) $v;
    }
    if (is_array($v)) {
        $v = array_map('canon', $v);
        if (!array_is_list($v)) {
            ksort($v, SORT_STRING);
        }
    }
    return $v;
}

check(count($doc['cases']) >= 100, 'at least 100 cases');
$flags = JSON_UNESCAPED_UNICODE | JSON_UNESCAPED_SLASHES;
foreach ($doc['cases'] as $c) {
    try {
        $got = ['ok' => runCase($c['kind'], $c['input'])];
    } catch (LocaleError $e) {
        $got = ['error' => $e->errorCode];
    }
    $g = json_encode(canon(json_decode(json_encode($got, $flags))), $flags);
    $w = json_encode(canon(json_decode(json_encode($c['expected'], $flags))), $flags);
    check($g === $w, "{$c['id']}: got $g want $w");
}
finish('vectors');
