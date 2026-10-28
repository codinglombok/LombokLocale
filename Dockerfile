# LombokLocale container image: the Python port of the library plus a
# self-verifying smoke test as the entrypoint. The library formats numbers,
# currencies, dates and ICU MessageFormat patterns with byte-identical
# results in every language port; this image ships the dependency-free
# Python port as a runnable, self-checking artifact.
#
# Build:  docker build -t lomboklocale:local .
# Run:    docker run --rm lomboklocale:local
#         (prints "ok, lomboklocale 0.2.0 (CLDR 47.0.0) python ..." and exits 0)
#
# The Python port uses only the standard library, so this build needs no
# network access beyond the base image itself.

FROM python:3.13-slim-bookworm

ARG VERSION=0.2.0
LABEL org.opencontainers.image.title="LombokLocale" \
      org.opencontainers.image.description="BCP 47 negotiation, CLDR 47 plural rules, number/currency/date formatting and an ICU MessageFormat subset with identical results across languages (Python port smoke image)" \
      org.opencontainers.image.version="${VERSION}" \
      org.opencontainers.image.source="https://github.com/codinglombok/LombokLocale" \
      org.opencontainers.image.licenses="Apache-2.0 OR MIT AND Unicode-3.0"

WORKDIR /opt/lomboklocale
COPY python/ python/
COPY scripts/docker_smoke.py /usr/local/bin/lomboklocale-smoke

RUN useradd --system --no-create-home --shell /usr/sbin/nologin lombok
USER lombok
ENV PYTHONPATH=/opt/lomboklocale/python

# Self-verification: the smoke test runs SPEC examples through the port and
# fails loudly on any byte-level deviation.
ENTRYPOINT ["python", "/usr/local/bin/lomboklocale-smoke"]
