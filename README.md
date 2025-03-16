# Code for Zero to Production in Rust

My code from working through [Zero to Production in Rust].

[Zero to Production in Rust]: https://www.zero2prod.com

## Initial setup

- Install Rust
- Install Docker
- `brew install libpq` to install `psql`
- `cargo install sqlx-cli --no-default-features --features rustls,postgres`
- `cargo install bunyan` for pretty printing logs

## Development

### Run app

- `bash scripts/init_db.sh` to start database server in Docker container (detached) for testing
- `SKIP_DOCKER=true bash scripts/init_db.sh` to apply migrations to database running in container
- `psql -h localhost -p 5432 -U postgres` to connect to database server from command line
- `cargo run | bunyan` to run app with pretty-printed logs
- `cargo watch --exec 'run | bunyan' --ignore *.md` to run app on file changes

### Run tests

- `cargo watch --exec check --exec test --ignore *.md` to run tests on file changes
- `TEST_LOG=true cargo test` to see logs from tests, or `TEST_LOG=true cargo test | bunyan` with pretty printing

To manually test API endpoints, use for example:

- `curl -v http://127.0.0.1:8000/health_check`
- `curl -v http://127.0.0.1:8000/subscriptions -H "Content-Type: application/x-www-form-urlencoded" -d "email=test@test.com&name=tester"`

### Inspect database

- `psql -h localhost -p 5432 -U postgres` to connect to the database server from command line

### Build Docker image

- `cargo sqlx prepare --workspace`, to work with sqlx offline, together with `SQLX_OFFLINE=true`
- `docker build --tag zero2prod --file Dockerfile .`
- `docker run -p 8000:8000 zero2prod`

## Notes

### Tools

- `cargo clippy` for linting
- `cargo tarpaulin --ignore-tests` for code coverage
- `cargo watch` for triggering commands automatically on code changes
- `cargo audit` to check for vulnerabilities
- `cargo expand` to show result of macro expansions
- `cargo +nightly udeps` to remove unused dependencies, requires `rustup toolchain install nightly`
- `cargo tree` to display dependency tree, useful for debugging incompatible dependencies

### Commands

- `rustup update` to update the Rust toolchain including `rustc` and `cargo`
- `tmux new-session -t <existing-session>` to attach to the same session multiple times and switch windows independently, using tmux grouped sessions (see [SO post](https://unix.stackexchange.com/a/24288)), e.g. for monitoring output from `cargo watch` commands in separate windows
