# GaussDB-Rust

[![Crates.io](https://img.shields.io/crates/v/gaussdb.svg)](https://crates.io/crates/gaussdb)
[![Documentation](https://docs.rs/gaussdb/badge.svg)](https://docs.rs/gaussdb)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

[English](README.md) | [中文](README-zh.md)

A native, high-performance Rust driver for GaussDB and OpenGauss databases with full PostgreSQL compatibility.

## gaussdb [![Latest Version](https://img.shields.io/crates/v/gaussdb.svg)](https://crates.io/crates/gaussdb)

[Documentation](https://docs.rs/gaussdb)

A native, synchronous GaussDB client with full PostgreSQL compatibility.

## tokio-gaussdb [![Latest Version](https://img.shields.io/crates/v/tokio-gaussdb.svg)](https://crates.io/crates/tokio-gaussdb)

[Documentation](https://docs.rs/tokio-gaussdb)

A native, asynchronous GaussDB client with full PostgreSQL compatibility.

## gaussdb-types [![Latest Version](https://img.shields.io/crates/v/gaussdb-types.svg)](https://crates.io/crates/gaussdb-types)

[Documentation](https://docs.rs/gaussdb-types)

Conversions between Rust and GaussDB/PostgreSQL types.

## gaussdb-native-tls [![Latest Version](https://img.shields.io/crates/v/gaussdb-native-tls.svg)](https://crates.io/crates/gaussdb-native-tls)

[Documentation](https://docs.rs/gaussdb-native-tls)

TLS support for gaussdb and tokio-gaussdb via native-tls.

## gaussdb-openssl [![Latest Version](https://img.shields.io/crates/v/gaussdb-openssl.svg)](https://crates.io/crates/gaussdb-openssl)

[Documentation](https://docs.rs/gaussdb-openssl)

TLS support for gaussdb and tokio-gaussdb via openssl.

# ✨ Features

## 🔐 Flexible Feature Support (v0.1.1+)

GaussDB-Rust now supports flexible feature flags to customize functionality:

- **`opengauss`** (default): Full OpenGauss support including `cancel_query` and domain types
- **`gauss`**: GaussDB enterprise edition support
- **No features**: Pure PostgreSQL compatibility

```toml
# Default (OpenGauss)
[dependencies]
gaussdb = "0.1"

# GaussDB enterprise
[dependencies]
gaussdb = { version = "0.1", default-features = false, features = ["gauss"] }

# PostgreSQL only
[dependencies]
gaussdb = { version = "0.1", default-features = false }
```

📖 See [FEATURES.md](FEATURES.md) for detailed feature documentation.

## GaussDB Authentication Support

This library provides full support for GaussDB's enhanced authentication mechanisms:

- **SCRAM-SHA-256 Compatibility**: Enhanced SCRAM-SHA-256 authentication with GaussDB/openGauss compatibility (v0.1.1+)
- **SHA256 Authentication**: GaussDB's secure SHA256-based authentication
- **MD5_SHA256 Authentication**: Hybrid authentication combining MD5 and SHA256
- **Standard PostgreSQL Authentication**: Full compatibility with MD5, SCRAM-SHA-256, and other PostgreSQL auth methods
- **Adaptive Authentication**: Intelligent authentication method selection based on server type (v0.1.1+)

## What's New in v0.1.1

### SCRAM-SHA-256 Compatibility Fixes
- ✅ **Fixed SCRAM Authentication**: Resolved "invalid message length: expected to be at end of iterator for sasl" error
- ✅ **GaussDB Message Parsing**: Enhanced SASL message parser with GaussDB-specific format support
- ✅ **Dual Authentication Strategy**: Automatic fallback from GaussDB-compatible to standard authentication
- ✅ **Runtime Conflict Resolution**: Fixed "Cannot start a runtime from within a runtime" errors in async environments

### Enhanced Features
- 🚀 **Performance Optimized**: Connection establishment time reduced to ~11.67ms average
- 🔍 **Better Diagnostics**: Comprehensive error analysis and troubleshooting tools
- 🧪 **Extensive Testing**: 184 tests with 100% pass rate on real GaussDB/openGauss environments
- 📊 **Production Ready**: Validated against openGauss 7.0.0-RC1 with high concurrency support

## Quick Start

### Basic Connection

```rust
use tokio_gaussdb::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Connect to GaussDB with SHA256 authentication
    let (client, connection) = tokio_gaussdb::connect(
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433",
        NoTls,
    ).await?;

    // Spawn the connection task
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Execute a simple query
    let rows = client.query("SELECT $1::TEXT", &[&"hello world"]).await?;
    let value: &str = rows[0].get(0);
    println!("Result: {}", value);

    Ok(())
}
```

### Advanced Authentication

```rust
use tokio_gaussdb::{Config, NoTls};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure connection with specific authentication
    let mut config = Config::new();
    config
        .host("localhost")
        .port(5433)
        .user("gaussdb")
        .password("Gaussdb@123")
        .dbname("postgres");

    let (client, connection) = config.connect(NoTls).await?;

    // Handle connection...
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Your application logic here
    Ok(())
}
```

## Compatibility

### Database Support

| Database | Version | Authentication | Status |
|----------|---------|----------------|--------|
| GaussDB | 0.1.1+ | SHA256, MD5_SHA256, MD5, SCRAM-SHA-256 | ✅ Full Support |
| OpenGauss | 3.0+ | SHA256, MD5_SHA256, MD5, SCRAM-SHA-256 | ✅ Full Support |
| PostgreSQL | 10+ | SCRAM-SHA-256, MD5 | ✅ Full Support |

### Feature Compatibility

| Feature | GaussDB | OpenGauss | PostgreSQL |
|---------|---------|-----------|------------|
| Basic SQL Operations | ✅ | ✅ | ✅ |
| Transactions | ✅ | ✅ | ✅ |
| Prepared Statements | ✅ | ✅ | ✅ |
| COPY Operations | ✅ | ✅ | ✅ |
| LISTEN/NOTIFY | ⚠️ Limited | ⚠️ Limited | ✅ |
| Binary COPY | ⚠️ Issues | ⚠️ Issues | ✅ |

## Running Tests

### Prerequisites

The test suite requires GaussDB or OpenGauss to be running. The easiest way is with Docker:

1. Install `docker` and `docker-compose`
   - On Ubuntu: `sudo apt install docker.io docker-compose`
   - On Windows: Install Docker Desktop
   - On macOS: Install Docker Desktop

2. Make sure your user has Docker permissions
   - On Ubuntu: `sudo usermod -aG docker $USER`

### Running Tests

1. Change to the top-level directory of `gaussdb-rust` repo
2. Start the test database:
   ```bash
   docker-compose up -d
   ```
3. Run the test suite:
   ```bash
   cargo test
   ```
4. Stop the test database:
   ```bash
   docker-compose stop
   ```

### Test Configuration

The test suite supports both GaussDB and OpenGauss environments. Connection strings are automatically configured for:

- **Host**: localhost
- **Port**: 5433 (GaussDB/OpenGauss default)
- **User**: gaussdb
- **Password**: Gaussdb@123
- **Database**: postgres

## 📚 Documentation

### API Documentation

- [gaussdb](https://docs.rs/gaussdb) - Synchronous client API
- [tokio-gaussdb](https://docs.rs/tokio-gaussdb) - Asynchronous client API
- [gaussdb-types](https://docs.rs/gaussdb-types) - Type conversion utilities
- [gaussdb-protocol](https://docs.rs/gaussdb-protocol) - Low-level protocol implementation

### Feature Guides

- **[FEATURES.md](FEATURES.md)** - Complete feature documentation (English)
- **[FEATURE_GUIDE_CN.md](FEATURE_GUIDE_CN.md)** - Feature usage guide (中文)
- **[FEATURE_SUMMARY.md](FEATURE_SUMMARY.md)** - Quick reference

### Technical Documentation

- [Authentication Methods](docs/authentication.md)
- [GaussDB vs PostgreSQL Differences](docs/GaussDB-PostgreSQL-差异分析报告.md)
- [Implementation Summary](IMPLEMENTATION_SUMMARY.md)

### Examples

Check out the [examples/](examples/) directory for complete working examples.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/HuaweiCloudDeveloper/gaussdb-rust.git
   cd gaussdb-rust
   ```

2. Install Rust (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. Run tests:
   ```bash
   cargo test
   ```

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

This project is based on the excellent [rust-postgres](https://github.com/sfackler/rust-postgres) library by Steven Fackler. We extend our gratitude to the original authors and contributors.

## Support

- [GitHub Issues](https://github.com/HuaweiCloudDeveloper/gaussdb-rust/issues) - Bug reports and feature requests
- [Documentation](https://docs.rs/gaussdb) - API documentation and guides
- [Examples](examples/) - Code examples and tutorials
