FROM rust:slim AS builder

RUN rustup target add x86_64-unknown-linux-musl
RUN apt update && apt install -y musl-tools musl-dev
RUN update-ca-certificates

WORKDIR /usr/src/app

# copy entire workspace
COPY . .

RUN cargo build --target x86_64-unknown-linux-musl --release


FROM alpine
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/rustcraft-bootstrap ./
CMD [ "./rustcraft-bootstrap" ]