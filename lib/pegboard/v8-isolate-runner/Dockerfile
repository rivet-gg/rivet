FROM clux/muslrust:1.80.0-stable

WORKDIR /app
COPY Cargo.toml .
COPY src/ src/
RUN cargo build --release --target x86_64-unknown-linux-gnu
