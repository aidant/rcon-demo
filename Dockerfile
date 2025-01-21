FROM rust:1.84.0 AS build
WORKDIR /app
COPY . /app
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=build /app/target/release/acorn /
CMD ["./acorn"]
