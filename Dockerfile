FROM rust:1-slim-buster AS base

LABEL org.label-schema.vendor="Begley Brothers Inc" \
  org.label-schema.url="https://github.com/begleybrothers/swanling" \
  org.label-schema.name="Swanling" \
  org.label-schema.version="mainline" \
  org.label-schema.vcs-url="github.com:begleybrothers/swanling.git" \
  org.label-schema.docker.schema-version="1.0"

ENV SWANLING_EXAMPLE=umami \
    SWANLING_FEATURES="regatta"

ARG DEBIAN_FRONTEND=noninteractive

COPY . /build
WORKDIR ./build

RUN apt-get update && \
  apt-get install -y libssl-dev gcc pkg-config cmake && \
  apt-get clean && \
  rm -rf /var/lib/apt/lists/*

RUN cargo build --features "${SWANLING_FEATURES}" --release --example "${SWANLING_EXAMPLE}"
RUN chmod +x ./docker-entrypoint.sh

EXPOSE 5115
ENTRYPOINT ["./docker-entrypoint.sh"]
