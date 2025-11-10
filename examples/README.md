# GaussDB-Rust Examples

This directory contains examples demonstrating how to use the gaussdb-rust library with both synchronous and asynchronous APIs.

## Prerequisites

Before running the examples, make sure you have:

1. **GaussDB or OpenGauss database running**
   ```bash
   # Using Docker (recommended)
   docker run --name gaussdb-test \
     -e GS_PASSWORD=Gaussdb@123 \
     -e GS_USERNAME=gaussdb \
     -e GS_DATABASE=postgres \
     -p 5433:5432 \
     -d opengauss/opengauss:latest

   ```

2. **Rust environment**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

## Examples Overview

### Synchronous Examples (gaussdb)

| Example | Description | Features |
|---------|-------------|----------|
| [`sync_basic.rs`](sync_basic.rs) | Basic synchronous operations | Connection, queries, transactions |
| [`sync_authentication.rs`](sync_authentication.rs) | GaussDB authentication methods | SHA256, MD5_SHA256 auth |
| [`sync_types.rs`](sync_types.rs) | Data type conversions | All supported PostgreSQL types |
| [`sync_transactions.rs`](sync_transactions.rs) | Transaction management | Commit, rollback, savepoints |
| [`sync_copy.rs`](sync_copy.rs) | COPY operations | Bulk data import/export |

### Asynchronous Examples (tokio-gaussdb)

| Example | Description | Features |
|---------|-------------|----------|
| [`async_basic.rs`](async_basic.rs) | Basic asynchronous operations | Connection, queries, transactions |
| [`async_authentication.rs`](async_authentication.rs) | GaussDB authentication methods | SHA256, MD5_SHA256 auth |
| [`async_types.rs`](async_types.rs) | Data type conversions | All supported PostgreSQL types |
| [`async_transactions.rs`](async_transactions.rs) | Transaction management | Commit, rollback, savepoints |
| [`async_copy.rs`](async_copy.rs) | COPY operations | Bulk data import/export |
| [`async_connection_pool.rs`](async_connection_pool.rs) | Connection pooling | High-performance connection management |

### Advanced Examples
 
| Example | Description | Features |
|---------|-------------|----------|
| [`tls_connection.rs`](tls_connection.rs) | TLS/SSL connections | Secure connections |
| [`custom_types.rs`](custom_types.rs) | Custom type definitions | Enums, composites, domains |
| [`migration_example.rs`](migration_example.rs) | Migration from rust-postgres | Step-by-step migration guide |

## Running Examples

### Individual Examples

```bash
# Run a specific example
cargo run --example sync_basic

# Run with features
cargo run --example async_basic --features "with-chrono-0_4"

# Run with environment variables
DATABASE_URL="host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433" \
cargo run --example async_basic
```

### All Examples

```bash
# Run all synchronous examples
for example in examples/sync_*.rs; do
    echo "Running $(basename $example)"
    cargo run --example $(basename $example .rs)
done

# Run all asynchronous examples  
for example in examples/async_*.rs; do
    echo "Running $(basename $example)"
    cargo run --example $(basename $example .rs)
done
```

## Configuration

### Environment Variables

Set these environment variables to customize database connection:

```bash
export DATABASE_URL="host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433"
export GAUSSDB_HOST="localhost"
export GAUSSDB_PORT="5433"
export GAUSSDB_USER="gaussdb"
export GAUSSDB_PASSWORD="Gaussdb@123"
export GAUSSDB_DATABASE="postgres"
```

### Connection String Format

```
host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433 sslmode=prefer
```

## Common Issues and Solutions

### 1. Connection Refused

**Problem**: `Connection refused (os error 10061)`

**Solution**: 
- Ensure GaussDB/OpenGauss is running
- Check port number (5433 for GaussDB, 5432 for PostgreSQL)
- Verify firewall settings

### 2. Authentication Failed

**Problem**: `password authentication failed`

**Solution**:
- Check username and password
- Verify authentication method in pg_hba.conf
- Ensure user has proper permissions

### 3. Database Does Not Exist

**Problem**: `database "test" does not exist`

**Solution**:
- Use existing database (usually "postgres")
- Create database first: `CREATE DATABASE test;`

### 4. SSL/TLS Issues

**Problem**: SSL connection errors

**Solution**:
- Use `sslmode=disable` for testing
- Install proper certificates for production
- Use `gaussdb-native-tls` or `gaussdb-openssl` crates

## Performance Tips

1. **Use Connection Pooling**: For high-concurrency applications
2. **Prepared Statements**: For repeated queries
3. **Batch Operations**: Use COPY for bulk data
4. **Async for I/O**: Use tokio-gaussdb for I/O-bound applications
5. **Sync for CPU**: Use gaussdb for CPU-bound applications

## Security Best Practices

1. **Use TLS**: Always enable TLS in production
2. **Strong Passwords**: Use complex passwords
3. **Least Privilege**: Grant minimal required permissions
4. **Parameter Binding**: Always use parameterized queries
5. **Connection Limits**: Set appropriate connection limits

## Contributing

To add new examples:

1. Create a new `.rs` file in the `examples/` directory
2. Follow the naming convention: `sync_*` or `async_*`
3. Include comprehensive error handling
4. Add documentation comments
5. Update this README.md

## Support

- [GitHub Issues](https://github.com/HuaweiCloudDeveloper/gaussdb-rust/issues)
- [Documentation](https://docs.rs/gaussdb)
- [GaussDB Documentation](https://docs.opengauss.org/)
