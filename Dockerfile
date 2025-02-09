FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt update && apt install lld clang -y

# planning stage to cache dependencies
FROM chef AS plan
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# build stage
FROM chef AS build
COPY --from=plan /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . . 
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin zero2prod

# runtime stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update -y \
  && apt-get install -y --no-install-recommends openssl ca-certificates \
  && apt-get autoremove -y \
  && apt-get clean -y \
  && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/zero2prod zero2prod
COPY config/ config/
ENV ZERO2PROD_APP_ENV=prod
ENTRYPOINT ["./zero2prod"]
