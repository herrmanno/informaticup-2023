FROM rust:1.64.0-slim-buster
WORKDIR /

# TODO: cache rust dependencies (or at least crates.io index) to speed up builds

COPY . .
RUN cargo build --release -p solver

ENTRYPOINT ["/target/release/solver"]
