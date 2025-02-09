FROM rust:1.84.0
WORKDIR /app
RUN apt update && apt install lld clang --yes
COPY . . 

ENV SQLX_OFFLINE=true
RUN cargo build --release

ENTRYPOINT ["./target/release/zero2prod"]
