FROM rustlang/rust:nightly-alpine as builder

RUN apk update && \
  apk add --no-cache bash curl npm libc-dev binaryen

RUN npm install -g sass

RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /work
COPY . .

RUN LEPTOS_BIN_TARGET_TRIPLE=x86_64-unknown-linux-musl cargo leptos build --release -vv

FROM scratch as runner

WORKDIR /app

COPY --from=builder /work/target/x86_64-unknown-linux-musl/release/photo-365 /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/Cargo.toml /app/
# COPY .env /app/

ENV LEPTOS_SITE_ROOT=./site
ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
EXPOSE 8080

CMD ["/app/photo-365"]
