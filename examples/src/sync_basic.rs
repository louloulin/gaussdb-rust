//! Basic synchronous GaussDB operations example
//!
//! This example demonstrates:
//! - Connecting to GaussDB using synchronous API
//! - Creating tables
//! - Inserting data
//! - Querying data
//! - Basic error handling
//!
//! Run with: cargo run --example sync_basic

use gaussdb::{Client, Error, NoTls};
use std::env;

fn main() -> Result<(), Error> {
    // Get connection string from environment or use default
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433".to_string()
    });

    println!("🔗 Connecting to GaussDB...");
    
    // Connect to the database
    let mut client = Client::connect(&database_url, NoTls)?;
    
    println!("✅ Connected successfully!");
    println!("📊 Database connection info:");
    println!("   - Using synchronous gaussdb client");
    println!("   - Connection string: {}", mask_password(&database_url));

    // Create a test table
    println!("\n🏗️  Creating test table...");
    client.batch_execute("
        DROP TABLE IF EXISTS sync_example_users;
        CREATE TABLE sync_example_users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            email VARCHAR(100) UNIQUE NOT NULL,
            age INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    ")?;
    println!("✅ Table created successfully!");

    // Insert some test data
    println!("\n📝 Inserting test data...");
    
    let users = vec![
        ("Alice Johnson", "alice@example.com", 28),
        ("Bob Smith", "bob@example.com", 35),
        ("Charlie Brown", "charlie@example.com", 42),
        ("Diana Prince", "diana@example.com", 30),
    ];

    for (name, email, age) in &users {
        client.execute(
            "INSERT INTO sync_example_users (name, email, age) VALUES ($1, $2, $3)",
            &[name, email, age],
        )?;
        println!("   ✓ Inserted: {} ({}, age {})", name, email, age);
    }

    // Query and display data
    println!("\n📖 Querying all users...");
    let rows = client.query("SELECT id, name, email, age, created_at FROM sync_example_users ORDER BY id", &[])?;
    
    println!("   Found {} users:", rows.len());
    println!("   ┌─────┬─────────────────┬─────────────────────┬─────┬─────────────────────┐");
    println!("   │ ID  │ Name            │ Email               │ Age │ Created At          │");
    println!("   ├─────┼─────────────────┼─────────────────────┼─────┼─────────────────────┤");
    
    for row in &rows {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        let email: &str = row.get(2);
        let age: i32 = row.get(3);
        let created_at: String = row.get::<_, String>(4);

        println!("   │ {:3} │ {:15} │ {:19} │ {:3} │ {:19} │",
                 id, name, email, age, &created_at[..19]);
    }
    println!("   └─────┴─────────────────┴─────────────────────┴─────┴─────────────────────┘");

    // Demonstrate prepared statements
    println!("\n🔍 Using prepared statements...");
    let stmt = client.prepare("SELECT name, email FROM sync_example_users WHERE age > $1")?;
    let rows = client.query(&stmt, &[&30])?;
    
    println!("   Users older than 30:");
    for row in &rows {
        let name: &str = row.get(0);
        let email: &str = row.get(1);
        println!("   - {} ({})", name, email);
    }

    // Demonstrate single row query
    println!("\n👤 Finding specific user...");
    match client.query_one("SELECT name, age FROM sync_example_users WHERE email = $1", &[&"alice@example.com"]) {
        Ok(row) => {
            let name: &str = row.get(0);
            let age: i32 = row.get(1);
            println!("   Found user: {} is {} years old", name, age);
        }
        Err(e) => println!("   User not found: {}", e),
    }

    // Demonstrate optional query
    println!("\n🔍 Optional query (may not find result)...");
    match client.query_opt("SELECT name FROM sync_example_users WHERE email = $1", &[&"nonexistent@example.com"]) {
        Ok(Some(row)) => {
            let name: &str = row.get(0);
            println!("   Found user: {}", name);
        }
        Ok(None) => println!("   No user found with that email"),
        Err(e) => println!("   Query error: {}", e),
    }

    // Update data
    println!("\n✏️  Updating user data...");
    let updated_rows = client.execute(
        "UPDATE sync_example_users SET age = age + 1 WHERE name = $1",
        &[&"Alice Johnson"],
    )?;
    println!("   Updated {} row(s)", updated_rows);

    // Verify update
    let row = client.query_one("SELECT age FROM sync_example_users WHERE name = $1", &[&"Alice Johnson"])?;
    let new_age: i32 = row.get(0);
    println!("   Alice's new age: {}", new_age);

    // Delete data
    println!("\n🗑️  Cleaning up...");
    let deleted_rows = client.execute("DELETE FROM sync_example_users WHERE age > $1", &[&40])?;
    println!("   Deleted {} user(s) older than 40", deleted_rows);

    // Final count
    let row = client.query_one("SELECT COUNT(*) FROM sync_example_users", &[])?;
    let count: i64 = row.get(0);
    println!("   Remaining users: {}", count);

    // Clean up table
    client.execute("DROP TABLE sync_example_users", &[])?;
    println!("   ✅ Test table dropped");

    println!("\n🎉 Synchronous example completed successfully!");
    println!("💡 Key takeaways:");
    println!("   - Synchronous API is simple and straightforward");
    println!("   - Perfect for scripts and simple applications");
    println!("   - All operations block until completion");
    println!("   - Error handling with Result<T, Error>");

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
