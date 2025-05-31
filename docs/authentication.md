# GaussDB Authentication Guide

## Overview

This guide explains how to use the various authentication methods supported by gaussdb-rust, including GaussDB-specific authentication mechanisms.

## Supported Authentication Methods

### 1. SHA256 Authentication (GaussDB Specific)

SHA256 is GaussDB's enhanced authentication method that provides better security than traditional MD5.

#### Algorithm
```
SHA256(password + username + salt)
```

#### Usage Example
```rust
use tokio_gaussdb::{Config, NoTls};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (client, connection) = tokio_gaussdb::connect(
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433",
        NoTls,
    ).await?;

    // The library automatically detects and uses SHA256 authentication
    // when the server requests it
    
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // Your application logic here
    Ok(())
}
```

### 2. MD5_SHA256 Authentication (GaussDB Specific)

MD5_SHA256 is a hybrid authentication method that combines SHA256 and MD5 for backward compatibility.

#### Algorithm
```
MD5(SHA256(password) + username + salt)
```

#### Configuration
This authentication method is automatically used when the GaussDB server is configured for MD5_SHA256 authentication.

### 3. Standard PostgreSQL Authentication

The library maintains full compatibility with PostgreSQL authentication methods:

- **MD5**: Traditional MD5-based authentication
- **SCRAM-SHA-256**: Modern secure authentication (PostgreSQL only)
- **Trust**: No password authentication
- **Password**: Plain text password (not recommended)

## Configuration Examples

### Basic Configuration

```rust
use tokio_gaussdb::{Config, NoTls};

let mut config = Config::new();
config
    .host("localhost")
    .port(5433)
    .user("gaussdb")
    .password("Gaussdb@123")
    .dbname("postgres");

let (client, connection) = config.connect(NoTls).await?;
```

### Connection String Format

```rust
// GaussDB connection
let conn_str = "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433";

// PostgreSQL connection  
let conn_str = "host=localhost user=postgres password=password dbname=mydb port=5432";

let (client, connection) = tokio_gaussdb::connect(conn_str, NoTls).await?;
```

### SSL/TLS Configuration

```rust
use tokio_gaussdb::{Config, NoTls};
use tokio_gaussdb_native_tls::{MakeTlsConnector, TlsConnector};

// For production use with TLS
let connector = TlsConnector::builder()
    .danger_accept_invalid_certs(false)
    .build()?;
let tls = MakeTlsConnector::new(connector);

let (client, connection) = tokio_gaussdb::connect(
    "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433 sslmode=require",
    tls,
).await?;
```

## Authentication Flow

### 1. Connection Establishment

```
Client                          GaussDB Server
  |                                    |
  |  1. Connection Request             |
  |  --------------------------------> |
  |                                    |
  |  2. Authentication Request         |
  |  <-------------------------------- |
  |     (SHA256/MD5_SHA256/MD5)        |
  |                                    |
  |  3. Authentication Response        |
  |  --------------------------------> |
  |     (Hashed password)              |
  |                                    |
  |  4. Authentication OK/Error        |
  |  <-------------------------------- |
```

### 2. Automatic Authentication Detection

The gaussdb-rust library automatically detects the authentication method requested by the server and responds appropriately:

```rust
// The library handles this automatically
match auth_type {
    AuthenticationSha256Password => {
        // Use SHA256 algorithm
        let hash = sha256_hash(username, password, salt);
        send_password_message(hash);
    }
    AuthenticationMd5Sha256Password => {
        // Use MD5_SHA256 algorithm  
        let hash = md5_sha256_hash(username, password, salt);
        send_password_message(hash);
    }
    AuthenticationMd5Password => {
        // Use standard MD5 algorithm
        let hash = md5_hash(username, password, salt);
        send_password_message(hash);
    }
}
```

## Error Handling

### Common Authentication Errors

```rust
use tokio_gaussdb::Error;

match tokio_gaussdb::connect(conn_str, NoTls).await {
    Ok((client, connection)) => {
        // Connection successful
    }
    Err(Error::Db(db_error)) => {
        match db_error.code() {
            Some(code) if code.code() == "28P01" => {
                eprintln!("Invalid password");
            }
            Some(code) if code.code() == "28000" => {
                eprintln!("Invalid authorization specification");
            }
            _ => {
                eprintln!("Database error: {}", db_error);
            }
        }
    }
    Err(e) => {
        eprintln!("Connection error: {}", e);
    }
}
```

### Authentication Debugging

Enable debug logging to troubleshoot authentication issues:

```rust
use log::LevelFilter;
use env_logger;

// Initialize logger
env_logger::Builder::from_default_env()
    .filter_level(LevelFilter::Debug)
    .init();

// Authentication details will be logged
let (client, connection) = tokio_gaussdb::connect(conn_str, NoTls).await?;
```

## Best Practices

### 1. Security Recommendations

- **Use SHA256**: Prefer SHA256 authentication over MD5 when possible
- **Enable TLS**: Always use TLS in production environments
- **Strong Passwords**: Use complex passwords with sufficient entropy
- **Connection Pooling**: Use connection pooling to reduce authentication overhead

### 2. Performance Considerations

- **Connection Reuse**: Reuse connections when possible to avoid repeated authentication
- **Authentication Caching**: The library caches authentication parameters for efficiency
- **Timeout Configuration**: Set appropriate connection timeouts

```rust
use tokio_gaussdb::Config;
use std::time::Duration;

let mut config = Config::new();
config
    .host("localhost")
    .port(5433)
    .user("gaussdb")
    .password("Gaussdb@123")
    .dbname("postgres")
    .connect_timeout(Duration::from_secs(10));
```

### 3. Error Recovery

```rust
use tokio_gaussdb::{Config, NoTls};
use std::time::Duration;

async fn connect_with_retry() -> Result<(tokio_gaussdb::Client, tokio_gaussdb::Connection<tokio_gaussdb::Socket, NoTls>), Box<dyn std::error::Error>> {
    let mut attempts = 0;
    let max_attempts = 3;
    
    loop {
        match tokio_gaussdb::connect(
            "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433",
            NoTls,
        ).await {
            Ok(result) => return Ok(result),
            Err(e) if attempts < max_attempts => {
                attempts += 1;
                eprintln!("Connection attempt {} failed: {}", attempts, e);
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
}
```

## Migration from rust-postgres

If you're migrating from rust-postgres, the authentication API is largely compatible:

```rust
// rust-postgres
use postgres::{Client, NoTls};
let mut client = Client::connect(conn_str, NoTls)?;

// gaussdb-rust (synchronous)
use gaussdb::{Client, NoTls};
let mut client = Client::connect(conn_str, NoTls)?;

// gaussdb-rust (asynchronous) 
use tokio_gaussdb::{connect, NoTls};
let (client, connection) = connect(conn_str, NoTls).await?;
```

The main differences:
1. Package names changed from `postgres` to `gaussdb`
2. Automatic support for GaussDB authentication methods
3. Enhanced error handling for GaussDB-specific scenarios

## Troubleshooting

### Common Issues

1. **Authentication Method Not Supported**
   - Ensure your GaussDB server supports the authentication method
   - Check pg_hba.conf configuration

2. **Connection Timeout**
   - Verify network connectivity
   - Check firewall settings
   - Increase connection timeout

3. **SSL/TLS Issues**
   - Verify SSL configuration on server
   - Check certificate validity
   - Use appropriate TLS connector

### Debug Information

Enable detailed logging for authentication debugging:

```bash
RUST_LOG=tokio_gaussdb=debug cargo run
```

This will show detailed information about the authentication process, including:
- Authentication method requested by server
- Hash calculations
- Network communication details
