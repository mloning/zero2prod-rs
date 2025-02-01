# Code for Zero to Production in Rust

My code from working through [Zero to Production in Rust].

[Zero to Production in Rust]: https://www.zero2prod.com

## Useful tools

- `cargo clippy` for linting
- `cargo tarpaulin --ignore-tests` for code coverage
- `cargo watch` for triggering commands automatically on code changes
- `cargo audit` to check for vulnerabilities
- `cargo expand` to show result of macro expansions

## Notes

- `tmux new-session -t <existing-session>` to attach to the same, existing session multiple times and switch windows independently, using grouped sessions (see [SO post](https://unix.stackexchange.com/a/24288))
