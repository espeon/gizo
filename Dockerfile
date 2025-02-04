FROM --platform=$BUILDPLATFORM rust:latest as builder

# Set the target platform architecture
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
    "linux/amd64")  echo "x86_64-unknown-linux-musl" > /target_arch ;; \
    "linux/arm64")  echo "aarch64-unknown-linux-musl" > /target_arch ;; \
    *)             echo "Unsupported platform: $TARGETPLATFORM" && exit 1 ;; \
    esac

WORKDIR /usr/src/app

# Install musl-tools and set up the appropriate target
RUN apt-get update && \
    apt-get install -y musl-tools && \
    rustup target add $(cat /target_arch)

# Copy the manifest files
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --target $(cat /target_arch) && \
    rm -rf src

# Copy the actual source code
COPY src ./src

# Build the static binary
RUN touch src/main.rs && \
    cargo build --release --target $(cat /target_arch)

FROM scratch

# Copy the static binary from builder
COPY --from=builder /usr/src/app/target/$(cat /target_arch)/release/gizo /app

# Expose the port
EXPOSE 3000

# Run the binary
CMD ["/app"]
