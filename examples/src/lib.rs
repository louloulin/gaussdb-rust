//! GaussDB-Rust Examples Library
//!
//! This library provides comprehensive examples for using the gaussdb-rust ecosystem,
//! including both synchronous and asynchronous APIs.
//!
//! # Examples Overview
//!
//! ## Synchronous Examples (gaussdb)
//! - [`sync_basic`] - Basic CRUD operations and connection management
//! - [`sync_authentication`] - GaussDB authentication methods
//! - [`sync_transactions`] - Transaction management and savepoints
//!
//! ## Asynchronous Examples (tokio-gaussdb)
//! - [`async_basic`] - Async CRUD operations and concurrent processing
//! - [`async_authentication`] - Async authentication and connection pooling
//!
//! # Quick Start
//!
//! To run any example:
//!
//! ```bash
//! # From the examples directory
//! cargo run --bin sync_basic
//! cargo run --bin async_basic
//! ```
//!
//! # Environment Configuration
//!
//! Set these environment variables to customize database connection:
//!
//! ```bash
//! export DATABASE_URL="host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433"
//! export GAUSSDB_HOST="localhost"
//! export GAUSSDB_PORT="5433"
//! export GAUSSDB_USER="gaussdb"
//! export GAUSSDB_PASSWORD="Gaussdb@123"
//! export GAUSSDB_DATABASE="postgres"
//! ```

#![warn(clippy::all, rust_2018_idioms, missing_docs)]
#![allow(dead_code)]

/// Common utilities for examples
pub mod common {
    use std::env;

    /// Get database connection URL from environment or use default
    pub fn get_database_url() -> String {
        env::var("DATABASE_URL").unwrap_or_else(|_| {
            "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433".to_string()
        })
    }

    /// Get individual connection parameters from environment
    pub fn get_connection_params() -> (String, u16, String, String, String) {
        let host = env::var("GAUSSDB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("GAUSSDB_PORT")
            .unwrap_or_else(|_| "5433".to_string())
            .parse()
            .unwrap_or(5433);
        let user = env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string());
        let password = env::var("GAUSSDB_PASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string());
        let database = env::var("GAUSSDB_DATABASE").unwrap_or_else(|_| "postgres".to_string());
        
        (host, port, user, password, database)
    }

    /// Mask password in connection string for logging
    pub fn mask_password(conn_str: &str) -> String {
        conn_str
            .split_whitespace()
            .map(|part| {
                if part.starts_with("password=") {
                    "password=***"
                } else {
                    part
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Initialize logging for examples
    pub fn init_logging() {
        env_logger::Builder::from_default_env()
            .filter_level(log::LevelFilter::Info)
            .init();
    }

    /// Print a formatted header for examples
    pub fn print_header(title: &str) {
        let width = title.len() + 4;
        let border = "=".repeat(width);
        
        println!("\n{}", border);
        println!("  {}  ", title);
        println!("{}", border);
    }

    /// Print a formatted section header
    pub fn print_section(title: &str) {
        let width = title.len() + 2;
        let border = "-".repeat(width);
        
        println!("\n{}", title);
        println!("{}", border);
    }

    /// Format a table row for display
    pub fn format_table_row(columns: &[&str], widths: &[usize]) -> String {
        let mut row = String::from("│");
        for (i, (col, width)) in columns.iter().zip(widths.iter()).enumerate() {
            if i > 0 {
                row.push_str(" │ ");
            } else {
                row.push(' ');
            }
            row.push_str(&format!("{:width$}", col, width = width));
            if i == columns.len() - 1 {
                row.push_str(" │");
            }
        }
        row
    }

    /// Create a table border
    pub fn create_table_border(widths: &[usize], style: char) -> String {
        let mut border = String::from("┌");
        for (i, width) in widths.iter().enumerate() {
            if i > 0 {
                border.push_str(&format!("{}┬", style.to_string().repeat(3)));
            }
            border.push_str(&style.to_string().repeat(width + 2));
            if i == widths.len() - 1 {
                border.push('┐');
            }
        }
        border
    }
}

/// Error types for examples
pub mod error {
    use thiserror::Error;

    /// Example-specific error types
    #[derive(Error, Debug)]
    pub enum ExampleError {
        /// Database connection error
        #[error("Database connection failed: {0}")]
        Database(#[from] tokio_gaussdb::Error),

        /// Configuration error
        #[error("Configuration error: {0}")]
        Config(String),

        /// Data validation error
        #[error("Data validation error: {0}")]
        Validation(String),

        /// Example execution error
        #[error("Example execution error: {0}")]
        Execution(String),
    }

    /// Result type for examples
    pub type ExampleResult<T> = Result<T, ExampleError>;
}

/// Test utilities for examples
#[cfg(test)]
pub mod test_utils {
    use super::common::*;
    use super::error::*;

    /// Test database connection
    pub fn test_connection() -> ExampleResult<()> {
        use gaussdb::{Client, NoTls};

        let database_url = get_database_url();
        let _client = Client::connect(&database_url, NoTls)
            .map_err(|e| ExampleError::Database(e))?;
        Ok(())
    }

    /// Test async database connection
    pub async fn test_async_connection() -> ExampleResult<()> {
        use tokio_gaussdb::{connect, NoTls};
        
        let database_url = get_database_url();
        let (_client, connection) = connect(&database_url, NoTls).await?;
        
        // Spawn connection task
        let connection_handle = tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Connection error: {}", e);
            }
        });
        
        // Clean up
        connection_handle.await.unwrap();
        Ok(())
    }

    /// Create test table for examples
    pub fn create_test_table(client: &mut gaussdb::Client, table_name: &str) -> ExampleResult<()> {
        client.execute(&format!("DROP TABLE IF EXISTS {}", table_name), &[])
            .map_err(|e| ExampleError::Database(e))?;
        client.execute(&format!(
            "CREATE TABLE {} (
                id SERIAL PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                value INTEGER,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )", table_name), &[])
            .map_err(|e| ExampleError::Database(e))?;
        Ok(())
    }

    /// Create test table for async examples
    pub async fn create_async_test_table(
        client: &tokio_gaussdb::Client,
        table_name: &str
    ) -> ExampleResult<()> {
        client.execute(&format!("DROP TABLE IF EXISTS {}", table_name), &[]).await?;
        client.execute(&format!(
            "CREATE TABLE {} (
                id SERIAL PRIMARY KEY,
                name VARCHAR(100) NOT NULL,
                value INTEGER,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            )", table_name), &[]).await?;
        Ok(())
    }
}

// Re-export commonly used types
pub use gaussdb::{Client as SyncClient, Error as SyncError, NoTls};
pub use tokio_gaussdb::{Client as AsyncClient, Error as AsyncError, connect};

// Re-export example modules (these will be binary targets)
// The actual example code is in separate binary files

#[cfg(test)]
mod tests {
    use super::*;
    use super::test_utils::*;

    #[test]
    fn test_mask_password() {
        let conn_str = "host=localhost user=test password=secret123 dbname=test";
        let masked = common::mask_password(conn_str);
        assert_eq!(masked, "host=localhost user=test password=*** dbname=test");
    }

    #[test]
    fn test_get_connection_params() {
        let (host, port, user, password, database) = common::get_connection_params();
        assert!(!host.is_empty());
        assert!(port > 0);
        assert!(!user.is_empty());
        assert!(!password.is_empty());
        assert!(!database.is_empty());
    }

    #[tokio::test]
    async fn test_database_connectivity() {
        use std::env;

        // This test requires a running database
        // Skip if DATABASE_URL is not set
        if env::var("DATABASE_URL").is_err() {
            println!("Skipping connectivity test - DATABASE_URL not set");
            return;
        }

        match test_connection() {
            Ok(_) => println!("✅ Sync connection test passed"),
            Err(e) => println!("⚠️ Sync connection test failed: {}", e),
        }

        match test_async_connection().await {
            Ok(_) => println!("✅ Async connection test passed"),
            Err(e) => println!("⚠️ Async connection test failed: {}", e),
        }
    }
}
