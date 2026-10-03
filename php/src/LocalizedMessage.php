<?php

declare(strict_types=1);

namespace LombokLocale;

/** A catalog-backed message; `fallback` is true when `text` is "!!id!!". */
final class LocalizedMessage
{
    public function __construct(
        public readonly string $code,
        public readonly string $messageId,
        public readonly string $text,
        public readonly bool $fallback,
    ) {
    }
}
