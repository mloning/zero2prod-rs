# Code for Zero to Production in Rust

My code from working through [Zero to Production in Rust].

[Zero to Production in Rust]: https://www.zero2prod.com

## Initial setup

- Install Rust
- Install Docker
- `brew install libpq` to install `psql`
- `cargo install sqlx-cli --no-default-features --features rustls,postgres`

## Development

### Run app

- `bash ./dev_tools/init_db.sh` to start database server in Docker container (detached) for testing
- `SKIP_DOCKER=true bash scripts/init_db.sh` to apply migrations if database is already running in container
- `cargo watch -x run` to run app on file changes

### Run tests

- `cargo watch -x check -x test` to run tests on file changes

To manually test API endpoints, use for example:

- `curl http://127.0.0.1:8000/health_check`
- `curl http://127.0.0.1:8000/subscriptions --header "Content-Type: application/x-www-form-urlencoded" --data-urlencode "email=test@test.com" --data-urlencode "name=Tester"`

### Inspect database

- `psql -h localhost -p 5432 -U postgres` to connect to database server from command line

## Notes

### Tools

- `cargo clippy` for linting
- `cargo tarpaulin --ignore-tests` for code coverage
- `cargo watch` for triggering commands automatically on code changes
- `cargo audit` to check for vulnerabilities
- `cargo expand` to show result of macro expansions

### Commands

- `rustup update` to update the Rust toolchain including `rustc` and `cargo`
- `tmux new-session -t <existing-session>` to attach to the same session multiple times and switch windows independently, using tmux grouped sessions (see [SO post](https://unix.stackexchange.com/a/24288)), e.g. for monitoring output from `cargo watch` commands in separate windows
