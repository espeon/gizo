FROM rust:alpine3.21 AS chef

WORKDIR /app

RUN apk update
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static

RUN cargo install cargo-chef

# planner
FROM chef AS planner

COPY . .

RUN cargo chef prepare --recipe-path recipe.json
# end planner

# cook
FROM chef AS cook

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json
# end cook

# builder
FROM cook AS builder

COPY . .

RUN cargo build --release
# end builder

# runner
FROM scratch

COPY --from=builder /app/target/release/gizo /app

EXPOSE 3000

CMD ["/app"]
