//! Simple synchronous GaussDB example
//!
//! This example demonstrates basic synchronous operations with GaussDB.
//!
//! Run with: cargo run --bin simple_sync

use gaussdb::{Client, Error, NoTls};
use std::env;

fn main() -> Result<(), Error> {
    println!("🔗 Simple GaussDB Synchronous Example");
    println!("=====================================");

    // Get connection string from environment or use default
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433".to_string()
    });

    println!("📊 Connecting to GaussDB...");
    println!("   Connection: {}", mask_password(&database_url));

    // Connect to the database
    let mut client = Client::connect(&database_url, NoTls)?;
    println!("✅ Connected successfully!");

    // Test basic query
    println!("\n🔍 Testing basic query...");
    let row = client.query_one("SELECT version()", &[])?;
    let version: &str = row.get(0);
    println!("   Database version: {}", version);

    // Test simple table operations
    println!("\n🏗️  Creating test table...");
    client.execute("DROP TABLE IF EXISTS simple_test", &[])?;
    client.execute(
        "CREATE TABLE simple_test (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            value INTEGER
        )",
        &[],
    )?;
    println!("   ✅ Table created");

    // Insert test data
    println!("\n📝 Inserting test data...");
    let rows_affected = client.execute(
        "INSERT INTO simple_test (name, value) VALUES ($1, $2)",
        &[&"test_item", &42],
    )?;
    println!("   ✅ Inserted {} row(s)", rows_affected);

    // Query test data
    println!("\n📖 Querying test data...");
    let rows = client.query("SELECT id, name, value FROM simple_test", &[])?;
    for row in &rows {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        let value: i32 = row.get(2);
        println!("   Found: id={}, name={}, value={}", id, name, value);
    }

    // Test transaction
    println!("\n💳 Testing transaction...");
    let mut transaction = client.transaction()?;
    transaction.execute(
        "INSERT INTO simple_test (name, value) VALUES ($1, $2)",
        &[&"transaction_test", &100],
    )?;
    transaction.commit()?;
    println!("   ✅ Transaction committed");

    // Final count
    let row = client.query_one("SELECT COUNT(*) FROM simple_test", &[])?;
    let count: i64 = row.get(0);
    println!("   Total rows: {}", count);

    // Cleanup
    println!("\n🗑️  Cleaning up...");
    client.execute("DROP TABLE simple_test", &[])?;
    println!("   ✅ Test table dropped");

    println!("\n🎉 Simple synchronous example completed successfully!");
    println!("💡 This demonstrates:");
    println!("   - Basic database connection");
    println!("   - Simple queries and data manipulation");
    println!("   - Transaction management");
    println!("   - Error handling");

    Ok(())
}

/// Mask password in connection string for logging
fn mask_password(conn_str: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_password() {
        let conn_str = "host=localhost user=test password=secret123 dbname=test";
        let masked = mask_password(conn_str);
        assert_eq!(masked, "host=localhost user=test password=*** dbname=test");
    }
}
