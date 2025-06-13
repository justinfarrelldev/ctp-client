# Common Commands

This project uses cargo-make, which is available via the command line.

## Build Commands
- `cargo make build-all-windows` - Build all Windows targets (x86 and x64)
- `cargo make build-all-linux` - Build all Linux targets (x86 and x64)
- `cargo make build-all` - Build all targets for current platform

## Linting Commands
- `cargo make lint` - Run clippy linting on the whole project with strict warnings
- `cargo make lint-fix` - Run clippy with automatic fixes on the whole project

## Testing Commands
- `cargo make test` - Run the standard test suite
- `cargo make test-verbose` - Run tests with verbose output (--nocapture)
- `cargo make test-release` - Run tests in release mode for performance testing

Just before each commit, remember to run the lint-fix command. Once your current task is complete and the lint-fix command has been run, commit your changes to Git (this will check your work with the linter, tests and build commands).

# Paradigms

Utilize test-driven development.