<?php

declare(strict_types=1);

namespace LombokLocale;

/** A well-formed BCP 47 tag in canonical case. */
final class LanguageTag
{
    /**
     * @param list<string> $variants
     * @param list<list<string>> $extensions each: singleton followed by its subtags, sorted by singleton
     * @param list<string> $privateUse
     */
    public function __construct(
        public string $language,
        public ?string $script = null,
        public ?string $region = null,
        public array $variants = [],
        public array $extensions = [],
        public array $privateUse = [],
    ) {
    }

    /** language, script, region and variants joined with "-". */
    public function base(): string
    {
        return implode('-', array_merge(array_values(array_filter([$this->language, $this->script, $this->region], static fn ($p) => $p !== null)), $this->variants));
    }

    /** @return list<string> RFC 4647 fallback list: the base tag, then shorter prefixes. */
    public function lookupChain(): array
    {
        $parts = explode('-', $this->base());
        $out = [];
        for ($k = count($parts); $k > 0; $k--) {
            $out[] = implode('-', array_slice($parts, 0, $k));
        }
        return $out;
    }

    public function __toString(): string
    {
        $parts = [$this->base()];
        foreach ($this->extensions as $e) {
            array_push($parts, ...$e);
        }
        if ($this->privateUse !== []) {
            $parts[] = 'x';
            array_push($parts, ...$this->privateUse);
        }
        return implode('-', $parts);
    }
}
