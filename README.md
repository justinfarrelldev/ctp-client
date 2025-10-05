# CTP Client

A custom Call to Power client for strategic civilization gameplay.

**IMPORTANT: THIS REPO IS IN EARLY DEVELOPMENT. The features below will not be complete, and should act as a plan rather than a complete feature set**

## Description

This client provides a modern interface for connecting to and playing Call to Power games with enhanced features and improved user experience. Built with Rust for performance and safety, this client aims to deliver a robust gaming experience.

## Features

- **Network Multiplayer Support**: Connect and play with other players online
- **Advanced Game State Management**: Efficient handling of complex game states
- **Modern UI/UX Design**: Clean and intuitive user interface
- **Cross-platform Compatibility**: Works on Windows, macOS, and Linux
- **High Performance**: Built with Rust for optimal performance
- **Memory Safety**: Leverages Rust's safety guarantees

## Installation

```bash
# Clone the repository
git clone https://github.com/user/ctp-client.git
cd ctp-client

# Build the project
cargo build --release

# Run the client
cargo run --release
```

## Development

### Prerequisites

- Rust 1.70.0 or later
- Cargo

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

More build types are in `Makefile.toml`.

### Testing

```bash
# Run all tests
cargo test

# Run tests with logging
RUST_LOG=debug cargo test
```

### Linting

This project uses extremely strict linting rules to ensure code quality:

```bash
# Run linting
cargo make lint

# Run linting and fix auto-fixable issues
cargo clippy --fix
```

## License

This project is licensed under the Unlicense - see the [LICENSE](LICENSE) file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Ensure all tests pass and linting is clean
5. Submit a pull request

## Support

For support and questions, please open an issue on the GitHub repository.
