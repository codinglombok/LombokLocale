<?php

declare(strict_types=1);

namespace LombokLocale;

/**
 * Thrown by every operation that can fail. `errorCode` is one of the SPEC
 * error codes (section 8); `detail` names the offending input.
 */
final class LocaleError extends \InvalidArgumentException
{
    public function __construct(public readonly string $errorCode, public readonly string $detail = '')
    {
        parent::__construct($detail === '' ? $errorCode : "$errorCode: $detail");
    }
}
