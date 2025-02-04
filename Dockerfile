FROM rust:latest as builder

WORKDIR /usr/src/app

# Install musl-tools to make static builds work
RUN apt-get update && \
    apt-get install -y musl-tools && \
    rustup target add x86_64-unknown-linux-musl

# Copy the manifest files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    rm -rf src

# Copy the actual source code
COPY src ./src

# Build the static binary
RUN touch src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch

# Copy the static binary
COPY --from=builder /usr/src/app/target/x86_64-unknown-linux-musl/release/gizo /app

# Expose the port
EXPOSE 3000

# Run the binary
CMD ["/app"]
