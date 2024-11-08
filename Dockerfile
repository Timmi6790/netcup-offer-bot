ARG BINDARY_NAME=netcup-offer-bot
ARG USER=runner

FROM clux/muslrust:stable AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM alpine AS env

ARG USER

RUN apk add --no-cache ca-certificates
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/app" \
    --shell "/sbin/nologin" \
    "$USER"

FROM scratch AS runtime

ARG BINDARY_NAME
ARG USER

ARG version=unknown
ARG release=unreleased

LABEL version=${version} \
      release=${release}

COPY --from=env /etc/passwd /etc/passwd
COPY --from=env /etc/group /etc/group
COPY --from=env /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/ca-certificates.crt

COPY --from=env --chown=$USER:$USER /app /app

WORKDIR /app
COPY --from=builder --chown=root:root /app/target/x86_64-unknown-linux-musl/release/$BINDARY_NAME ./app

USER $USER:$USER

CMD ["./app"]