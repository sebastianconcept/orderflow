# syntax=docker/dockerfile:1

# Build all pipeline binaries from the workspace.
FROM rust:1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates

RUN cargo build --release \
    -p engine-core \
    -p input \
    -p screener \
    -p sequencer \
    -p scheduler \
    -p output

RUN mkdir -p /app/bin && \
    for name in engine-core input screener sequencer scheduler output; do \
      path="$(find /app/target -type f -path "*/release/${name}" | head -n 1)"; \
      if [ -z "${path}" ]; then \
        echo "missing binary: ${name}" >&2; \
        exit 1; \
      fi; \
      install -m 755 "${path}" "/app/bin/${name}"; \
    done

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tini \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/bin /app/bin
COPY docker/entrypoint.sh /app/entrypoint.sh
RUN chmod 755 /app/entrypoint.sh

ENV PATH="/app/bin:${PATH}" \
    ORDERFLOW_BIN=engine-core \
    ORDERFLOW_ENV=production

ENTRYPOINT ["/usr/bin/tini", "--", "/app/entrypoint.sh"]

# Kamal proxy defaults to port 80; expose when HTTP health/API lands on engine.
EXPOSE 80
