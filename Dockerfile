FROM rust:1.66.1
WORKDIR /

# Copy workspace cargo files
COPY Cargo.* ./

# Create workspace member directories
RUN mkdir -p common/src model/src model/benches printer/src qa/src simulator/src solver/src solver/benches

# Copy workspace member cargo files
COPY common/Cargo.* common
COPY model/Cargo.* model
COPY printer/Cargo.* printer
COPY qa/Cargo.* qa
COPY simulator/Cargo.* simulator
COPY solver/Cargo.* solver

# Create workspace member dummy main files
RUN echo 'fn main() {}' > common/src/main.rs
RUN echo 'fn main() {}' > model/src/main.rs
RUN touch model/benches/benchmarks.rs
RUN echo 'fn main() {}' > printer/src/main.rs
RUN echo 'fn main() {}' > qa/src/main.rs
RUN echo 'fn main() {}' > simulator/src/main.rs
RUN echo 'fn main() {}' > solver/src/main.rs
RUN touch solver/benches/benchmarks.rs

# Build once to cache dependencies
RUN cargo build --release -p solver

# Copy real sources
COPY . .

# Build final binary
RUN cargo build --release -p solver

ENTRYPOINT ["/target/release/solver"]
