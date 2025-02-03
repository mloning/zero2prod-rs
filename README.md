# Code for Zero to Production in Rust

My code from working through [Zero to Production in Rust].

[Zero to Production in Rust]: https://www.zero2prod.com

## Initial setup

- Install Rust
- Install Docker
- `brew install libpq` to install `psql`
- `cargo install sqlx-cli --no-default-features --features rustls,postgres`

## Development

- `bash scripts/init_db.sh` to start database server in Docker container (detached) for testing
- `SKIP_DOCKER=true bash scripts/init_db.sh` to apply migrations to database running in container
- `psql -h localhost -p 5432 -U postgres` to connect to database server from command line
- `cargo watch -x check -x test` to run tests on file changes

## Useful tools

- `cargo clippy` for linting
- `cargo tarpaulin --ignore-tests` for code coverage
- `cargo watch` for triggering commands automatically on code changes
- `cargo audit` to check for vulnerabilities
- `cargo expand` to show result of macro expansions

## Notes

- `rustup update` to update the Rust toolchain including `rustc` and `cargo`
- `tmux new-session -t <existing-session>` to attach to the same, existing session multiple times and switch windows independently, using grouped sessions (see [SO post](https://unix.stackexchange.com/a/24288))
