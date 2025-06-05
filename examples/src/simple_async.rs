//! Simple asynchronous GaussDB example
//!
//! This example demonstrates basic asynchronous operations with GaussDB.
//!
//! Run with: cargo run --bin simple_async

use std::env;
use tokio_gaussdb::{connect, Error, NoTls};

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("🔗 Simple GaussDB Asynchronous Example");
    println!("======================================");

    // Get connection string from environment or use default
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433".to_string()
    });

    println!("📊 Connecting to GaussDB asynchronously...");
    println!("   Connection: {}", mask_password(&database_url));

    // Connect to the database
    let (mut client, connection) = connect(&database_url, NoTls).await?;

    // Spawn the connection task
    let connection_handle = tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    println!("✅ Connected successfully!");

    // Test basic query
    println!("\n🔍 Testing basic query...");
    let row = client.query_one("SELECT version()", &[]).await?;
    let version: &str = row.get(0);
    println!("   Database version: {}", version);

    // Test simple table operations
    println!("\n🏗️  Creating test table...");
    client
        .execute("DROP TABLE IF EXISTS async_simple_test", &[])
        .await?;
    client
        .execute(
            "CREATE TABLE async_simple_test (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            value INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
            &[],
        )
        .await?;
    println!("   ✅ Table created");

    // Insert test data concurrently
    println!("\n📝 Inserting test data concurrently...");
    let insert_tasks = vec![
        client.execute(
            "INSERT INTO async_simple_test (name, value) VALUES ($1, $2)",
            &[&"async_item_1", &10],
        ),
        client.execute(
            "INSERT INTO async_simple_test (name, value) VALUES ($1, $2)",
            &[&"async_item_2", &20],
        ),
        client.execute(
            "INSERT INTO async_simple_test (name, value) VALUES ($1, $2)",
            &[&"async_item_3", &30],
        ),
    ];

    let results = futures_util::future::join_all(insert_tasks).await;
    let mut total_inserted = 0;
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(rows) => {
                total_inserted += rows;
                println!("   ✅ Insert {} completed: {} row(s)", i + 1, rows);
            }
            Err(e) => println!("   ❌ Insert {} failed: {}", i + 1, e),
        }
    }
    println!("   Total inserted: {} rows", total_inserted);

    // Query test data
    println!("\n📖 Querying test data...");
    let rows = client
        .query(
            "SELECT id, name, value FROM async_simple_test ORDER BY id",
            &[],
        )
        .await?;
    println!("   Found {} rows:", rows.len());
    for row in &rows {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        let value: i32 = row.get(2);
        println!("   - id={}, name={}, value={}", id, name, value);
    }

    // Test concurrent queries
    println!("\n🔄 Testing concurrent queries...");
    let (count_result, max_result, min_result) = tokio::join!(
        client.query_one("SELECT COUNT(*) FROM async_simple_test", &[]),
        client.query_one("SELECT MAX(value) FROM async_simple_test", &[]),
        client.query_one("SELECT MIN(value) FROM async_simple_test", &[])
    );

    match (count_result, max_result, min_result) {
        (Ok(count_row), Ok(max_row), Ok(min_row)) => {
            let count: i64 = count_row.get(0);
            let max_value: Option<i32> = max_row.get(0);
            let min_value: Option<i32> = min_row.get(0);

            println!("   Statistics (queried concurrently):");
            println!("   - Total rows: {}", count);
            println!("   - Max value: {:?}", max_value);
            println!("   - Min value: {:?}", min_value);
        }
        _ => println!("   ❌ Some concurrent queries failed"),
    }

    // Test transaction
    println!("\n💳 Testing async transaction...");
    let transaction = client.transaction().await?;
    transaction
        .execute(
            "INSERT INTO async_simple_test (name, value) VALUES ($1, $2)",
            &[&"transaction_test", &999],
        )
        .await?;
    transaction.commit().await?;
    println!("   ✅ Async transaction committed");

    // Final count
    let row = client
        .query_one("SELECT COUNT(*) FROM async_simple_test", &[])
        .await?;
    let final_count: i64 = row.get(0);
    println!("   Final row count: {}", final_count);

    // Cleanup
    println!("\n🗑️  Cleaning up...");
    client.execute("DROP TABLE async_simple_test", &[]).await?;
    println!("   ✅ Test table dropped");

    // Close connection gracefully
    drop(client);
    connection_handle.await.unwrap();

    println!("\n🎉 Simple asynchronous example completed successfully!");
    println!("💡 This demonstrates:");
    println!("   - Async database connection with connection task management");
    println!("   - Concurrent operations and queries");
    println!("   - Async transaction management");
    println!("   - Graceful connection cleanup");

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
