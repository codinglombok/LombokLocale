<?php

declare(strict_types=1);

namespace LombokLocale;

/** A number given as exact decimal text, e.g. `new Num('1.50')`. */
final class Num
{
    public function __construct(public readonly string $value)
    {
    }
}
