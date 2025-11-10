//! Basic asynchronous GaussDB operations example
//!
//! This example demonstrates:
//! - Connecting to GaussDB using asynchronous API
//! - Creating tables
//! - Inserting data
//! - Querying data
//! - Async/await patterns
//! - Error handling
//!
//! Run with: cargo run --example async_basic

use tokio_gaussdb::{connect, Error, NoTls};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Get connection string from environment or use default
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433".to_string()
    });

    println!("🔗 Connecting to GaussDB asynchronously...");
    
    // Connect to the database
    let (client, connection) = connect(&database_url, NoTls).await?;
    
    // Spawn the connection task
    let connection_handle = tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    println!("✅ Connected successfully!");
    println!("📊 Database connection info:");
    println!("   - Using asynchronous tokio-gaussdb client");
    println!("   - Connection string: {}", mask_password(&database_url));

    // Create a test table
    println!("\n🏗️  Creating test table...");
    client.batch_execute("
        DROP TABLE IF EXISTS async_example_products;
        CREATE TABLE async_example_products (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            category VARCHAR(50) NOT NULL,
            price DECIMAL(10,2) NOT NULL,
            in_stock BOOLEAN DEFAULT true,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    ").await?;
    println!("✅ Table created successfully!");

    // Insert some test data using concurrent operations
    println!("\n📝 Inserting test data concurrently...");
    
    let products = vec![
        ("Laptop Pro", "Electronics", 1299.99),
        ("Wireless Mouse", "Electronics", 29.99),
        ("Office Chair", "Furniture", 199.99),
        ("Coffee Mug", "Kitchen", 12.99),
        ("Notebook", "Stationery", 5.99),
    ];

    // Insert products sequentially (client cannot be cloned)
    for (name, category, price) in products {
        let sql = format!(
            "INSERT INTO async_example_products (name, category, price) VALUES ('{}', '{}', {})",
            name, category, price
        );
        client.execute(&sql, &[]).await?;
        println!("   ✓ Inserted: {} - {} (${:.2})", name, category, price);
    }

    // Query and display data
    println!("\n📖 Querying all products...");
    let rows = client.query(
        "SELECT id, name, category, price::float8, in_stock, created_at::text FROM async_example_products ORDER BY id", 
        &[]
    ).await?;
    
    println!("   Found {} products:", rows.len());
    println!("   ┌─────┬─────────────────┬─────────────┬─────────┬─────────┬─────────────────────┐");
    println!("   │ ID  │ Name            │ Category    │ Price   │ Stock   │ Created At          │");
    println!("   ├─────┼─────────────────┼─────────────┼─────────┼─────────┼─────────────────────┤");
    
    for row in &rows {
        let id: i32 = row.get(0);
        let name: &str = row.get(1);
        let category: &str = row.get(2);
        let price: f64 = row.get(3);
        let in_stock: bool = row.get(4);
        let created_at: String = row.get::<_, String>(5);
        
        println!("   │ {:3} │ {:15} │ {:11} │ ${:6.2} │ {:7} │ {:19} │",
                 id, name, category, price, if in_stock { "Yes" } else { "No" },
                 &created_at[..19]);
    }
    println!("   └─────┴─────────────────┴─────────────┴─────────┴─────────┴─────────────────────┘");

    // Demonstrate prepared statements with async
    println!("\n🔍 Using prepared statements asynchronously...");
    let stmt = client.prepare("SELECT name, price::float8 FROM async_example_products WHERE category = $1").await?;
    let rows = client.query(&stmt, &[&"Electronics"]).await?;
    
    println!("   Electronics products:");
    for row in &rows {
        let name: &str = row.get(0);
        let price: f64 = row.get(1);
        println!("   - {} (${:.2})", name, price);
    }

    // Demonstrate concurrent queries
    println!("\n🔄 Running concurrent queries...");
    let (electronics_task, furniture_task, kitchen_task) = tokio::join!(
        client.query("SELECT COUNT(*) FROM async_example_products WHERE category = $1", &[&"Electronics"]),
        client.query("SELECT COUNT(*) FROM async_example_products WHERE category = $1", &[&"Furniture"]),
        client.query("SELECT COUNT(*) FROM async_example_products WHERE category = $1", &[&"Kitchen"])
    );

    match (electronics_task, furniture_task, kitchen_task) {
        (Ok(e_rows), Ok(f_rows), Ok(k_rows)) => {
            let electronics_count: i64 = e_rows[0].get(0);
            let furniture_count: i64 = f_rows[0].get(0);
            let kitchen_count: i64 = k_rows[0].get(0);
            
            println!("   Category counts (queried concurrently):");
            println!("   - Electronics: {}", electronics_count);
            println!("   - Furniture: {}", furniture_count);
            println!("   - Kitchen: {}", kitchen_count);
        }
        _ => println!("   Some queries failed"),
    }

    // Demonstrate streaming results
    println!("\n🌊 Streaming query results...");
    let rows = client.query(
        "SELECT name, price::float8 FROM async_example_products WHERE price::float8 > $1 ORDER BY price DESC",
        &[&50.0f64]
    ).await?;

    println!("   Expensive products (>$50):");
    for row in &rows {
        let name: &str = row.get(0);
        let price: f64 = row.get(1);
        println!("   - {} (${:.2})", name, price);
    }

    // Update data asynchronously
    println!("\n✏️  Updating product data...");
    let updated_rows = client.execute(
        "UPDATE async_example_products SET price = price * 0.9 WHERE category = $1",
        &[&"Electronics"],
    ).await?;
    println!("   Applied 10% discount to {} electronics product(s)", updated_rows);

    // Verify updates
    let rows = client.query("SELECT name, price::float8 FROM async_example_products WHERE category = $1", &[&"Electronics"]).await?;
    println!("   Updated electronics prices:");
    for row in &rows {
        let name: &str = row.get(0);
        let price: f64 = row.get(1);
        println!("   - {} (${:.2})", name, price);
    }

    // Batch operations
    println!("\n📦 Batch operations...");
    let batch_updates = vec![
        ("Office Chair", false),
        ("Coffee Mug", true),
        ("Notebook", true),
    ];

    for (product_name, stock_status) in batch_updates {
        client.execute(
            "UPDATE async_example_products SET in_stock = $1 WHERE name = $2",
            &[&stock_status, &product_name],
        ).await?;
    }
    println!("   ✅ Batch stock updates completed");

    // Final statistics
    println!("\n📊 Final statistics...");
    let stats = client.query_one("
        SELECT 
            COUNT(*) as total_products,
            COUNT(*) FILTER (WHERE in_stock = true) as in_stock_count,
            AVG(price)::float8 as avg_price,
            MAX(price)::float8 as max_price,
            MIN(price)::float8 as min_price
        FROM async_example_products
    ", &[]).await?;

    let total: i64 = stats.get(0);
    let in_stock: i64 = stats.get(1);
    let avg_price: Option<f64> = stats.get(2);
    let max_price: f64 = stats.get(3);
    let min_price: f64 = stats.get(4);

    println!("   📈 Product Statistics:");
    println!("   - Total products: {}", total);
    println!("   - In stock: {}", in_stock);
    println!("   - Average price: ${:.2}", avg_price.unwrap_or(0.0));
    println!("   - Price range: ${:.2} - ${:.2}", min_price, max_price);

    // Clean up
    println!("\n🗑️  Cleaning up...");
    client.execute("DROP TABLE async_example_products", &[]).await?;
    println!("   ✅ Test table dropped");

    // Close connection gracefully
    drop(client);
    connection_handle.await.unwrap();

    println!("\n🎉 Asynchronous example completed successfully!");
    println!("💡 Key takeaways:");
    println!("   - Async API enables concurrent operations");
    println!("   - Perfect for I/O-bound and high-concurrency applications");
    println!("   - Non-blocking operations with async/await");
    println!("   - Connection must be spawned as a separate task");
    println!("   - Streaming results for large datasets");

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
