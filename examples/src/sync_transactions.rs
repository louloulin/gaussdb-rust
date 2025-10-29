//! Transaction Management Example (Synchronous)
//!
//! This example demonstrates:
//! - Basic transactions (BEGIN, COMMIT, ROLLBACK)
//! - Savepoints and nested transactions
//! - Transaction isolation levels
//! - Error handling in transactions
//! - Batch operations within transactions
//!
//! Run with: cargo run --example sync_transactions

use gaussdb::{Client, Error, NoTls, IsolationLevel};
use std::env;

fn main() -> Result<(), Error> {
    println!("💳 Transaction Management Demo (Synchronous)");
    println!("============================================");

    // Connect to database
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433".to_string()
    });

    let mut client = Client::connect(&database_url, NoTls)?;
    println!("✅ Connected to GaussDB");

    // Setup test tables
    setup_test_tables(&mut client)?;

    // Run transaction examples
    basic_transaction_example(&mut client)?;
    rollback_transaction_example(&mut client)?;
    savepoint_example(&mut client)?;
    isolation_level_example(&mut client)?;
    batch_transaction_example(&mut client)?;

    // Cleanup
    cleanup_test_tables(&mut client)?;

    println!("\n🎉 Transaction examples completed successfully!");
    Ok(())
}

/// Setup test tables for transaction examples
fn setup_test_tables(client: &mut Client) -> Result<(), Error> {
    println!("\n🏗️  Setting up test tables...");
    
    client.batch_execute("
        DROP TABLE IF EXISTS accounts CASCADE;
        DROP TABLE IF EXISTS transactions CASCADE;
        
        CREATE TABLE accounts (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            balance DECIMAL(10,2) NOT NULL DEFAULT 0.00,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
        
        CREATE TABLE transactions (
            id SERIAL PRIMARY KEY,
            from_account_id INTEGER REFERENCES accounts(id),
            to_account_id INTEGER REFERENCES accounts(id),
            amount DECIMAL(10,2) NOT NULL,
            description TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );
    ")?;

    // Insert initial account data
    client.execute("INSERT INTO accounts (name, balance) VALUES ('Alice', 1000.00)", &[])?;
    client.execute("INSERT INTO accounts (name, balance) VALUES ('Bob', 500.00)", &[])?;
    client.execute("INSERT INTO accounts (name, balance) VALUES ('Charlie', 750.00)", &[])?;

    println!("   ✅ Test tables created and populated");
    Ok(())
}

/// Basic transaction example - successful commit
fn basic_transaction_example(client: &mut Client) -> Result<(), Error> {
    println!("\n1️⃣  Basic Transaction Example");
    println!("----------------------------");

    // Show initial balances
    show_account_balances(client, "Initial balances")?;

    // Start transaction
    let mut transaction = client.transaction()?;
    println!("   🔄 Transaction started");

    // Transfer money from Alice to Bob
    let amount = 100.00;
    
    // Debit from Alice
    let updated_rows = transaction.execute(
        &format!("UPDATE accounts SET balance = balance - {} WHERE name = 'Alice'", amount),
        &[],
    )?;
    println!("   💸 Debited ${:.2} from Alice ({} rows updated)", amount, updated_rows);

    // Credit to Bob
    let updated_rows = transaction.execute(
        &format!("UPDATE accounts SET balance = balance + {} WHERE name = 'Bob'", amount),
        &[],
    )?;
    println!("   💰 Credited ${:.2} to Bob ({} rows updated)", amount, updated_rows);

    // Record transaction
    transaction.execute(
        &format!("INSERT INTO transactions (from_account_id, to_account_id, amount, description) 
         VALUES ((SELECT id FROM accounts WHERE name = 'Alice'), 
                 (SELECT id FROM accounts WHERE name = 'Bob'), {}, 'Basic transfer example')", amount),
        &[],
    )?;
    println!("   📝 Transaction recorded");

    // Commit transaction
    transaction.commit()?;
    println!("   ✅ Transaction committed successfully");

    // Show final balances
    show_account_balances(client, "After basic transaction")?;

    Ok(())
}

/// Rollback transaction example - intentional failure
fn rollback_transaction_example(client: &mut Client) -> Result<(), Error> {
    println!("\n2️⃣  Rollback Transaction Example");
    println!("--------------------------------");

    // Show initial balances
    show_account_balances(client, "Before rollback example")?;

    // Start transaction
    let mut transaction = client.transaction()?;
    println!("   🔄 Transaction started");

    let amount = 200.00;

    // Debit from Bob
    transaction.execute(
        &format!("UPDATE accounts SET balance = balance - {} WHERE name = 'Bob'", amount),
        &[],
    )?;
    println!("   💸 Debited ${:.2} from Bob", amount);

    // Simulate an error condition - try to credit to non-existent account
    let should_rollback = match transaction.execute(
        &format!("UPDATE accounts SET balance = balance + {} WHERE name = 'NonExistentUser'", amount),
        &[],
    ) {
        Ok(rows) => {
            if rows == 0 {
                println!("   ❌ No rows updated - recipient account not found");
                true
            } else {
                false
            }
        }
        Err(e) => {
            println!("   ❌ Error occurred: {}", e);
            true
        }
    };

    if should_rollback {
        println!("   🔄 Rolling back transaction...");
        transaction.rollback()?;
        println!("   ✅ Transaction rolled back successfully");
    } else {
        // If not rolling back, we still need to drop the transaction
        drop(transaction);
    }

    // Show balances - should be unchanged
    show_account_balances(client, "After rollback (unchanged)")?;

    Ok(())
}

/// Savepoint example - nested transactions
fn savepoint_example(client: &mut Client) -> Result<(), Error> {
    println!("\n3️⃣  Savepoint Example");
    println!("--------------------");

    show_account_balances(client, "Before savepoint example")?;

    // Start main transaction
    let mut transaction = client.transaction()?;
    println!("   🔄 Main transaction started");

    let amount1 = 50.00;
    let amount2 = 75.00;

    // First operation - Alice to Charlie
    transaction.execute(
        &format!("UPDATE accounts SET balance = balance - {} WHERE name = 'Alice'", amount1),
        &[],
    )?;
    transaction.execute(
        &format!("UPDATE accounts SET balance = balance + {} WHERE name = 'Charlie'", amount1),
        &[],
    )?;
    println!("   ✅ First transfer: Alice -> Charlie (${:.2})", amount1);

    // Create savepoint
    transaction.execute("SAVEPOINT sp1", &[])?;
    println!("   💾 Savepoint 'sp1' created");

    // Second operation - Bob to Charlie (this will be rolled back)
    transaction.execute(
        &format!("UPDATE accounts SET balance = balance - {} WHERE name = 'Bob'", amount2),
        &[],
    )?;
    transaction.execute(
        &format!("UPDATE accounts SET balance = balance + {} WHERE name = 'Charlie'", amount2),
        &[],
    )?;
    println!("   ✅ Second transfer: Bob -> Charlie (${:.2})", amount2);

    // Show intermediate state
    let rows = transaction.query("SELECT name, balance::float8 FROM accounts ORDER BY name", &[])?;
    println!("   📊 Intermediate balances:");
    for row in &rows {
        let name: &str = row.get(0);
        let balance: f64 = row.get(1);
        println!("      - {}: ${:.2}", name, balance);
    }

    // Rollback to savepoint (undo second transfer)
    transaction.execute("ROLLBACK TO SAVEPOINT sp1", &[])?;
    println!("   🔄 Rolled back to savepoint 'sp1'");

    // Commit main transaction (first transfer remains)
    transaction.commit()?;
    println!("   ✅ Main transaction committed");

    show_account_balances(client, "After savepoint example")?;

    Ok(())
}

/// Isolation level example
fn isolation_level_example(client: &mut Client) -> Result<(), Error> {
    println!("\n4️⃣  Isolation Level Example");
    println!("---------------------------");

    // Test different isolation levels
    let isolation_levels = vec![
        ("READ UNCOMMITTED", IsolationLevel::ReadUncommitted),
        ("READ COMMITTED", IsolationLevel::ReadCommitted),
        ("REPEATABLE READ", IsolationLevel::RepeatableRead),
        ("SERIALIZABLE", IsolationLevel::Serializable),
    ];

    for (name, level) in isolation_levels {
        println!("   🔒 Testing {} isolation level", name);
        
        let mut transaction = client.build_transaction()
            .isolation_level(level)
            .start()?;

        // Read current balance
        let row = transaction.query_one("SELECT balance::float8 FROM accounts WHERE name = $1", &[&"Alice"])?;
        let balance: f64 = row.get(0);
        println!("      Alice's balance: ${:.2}", balance);

        // Commit transaction
        transaction.commit()?;
        println!("      ✅ {} transaction completed", name);
    }

    Ok(())
}

/// Batch transaction example
fn batch_transaction_example(client: &mut Client) -> Result<(), Error> {
    println!("\n5️⃣  Batch Transaction Example");
    println!("----------------------------");

    show_account_balances(client, "Before batch operations")?;

    // Start transaction for batch operations
    let mut transaction = client.transaction()?;
    println!("   🔄 Batch transaction started");

    // Batch of operations
    let operations = vec![
        ("Alice", "Bob", 25.00, "Batch transfer 1"),
        ("Bob", "Charlie", 30.00, "Batch transfer 2"),
        ("Charlie", "Alice", 15.00, "Batch transfer 3"),
    ];

    for (from, to, amount, description) in operations {
        // Debit
        transaction.execute(
            &format!("UPDATE accounts SET balance = balance - {} WHERE name = '{}'", amount, from),
            &[],
        )?;

        // Credit
        transaction.execute(
            &format!("UPDATE accounts SET balance = balance + {} WHERE name = '{}'", amount, to),
            &[],
        )?;

        // Record
        transaction.execute(
            &format!("INSERT INTO transactions (from_account_id, to_account_id, amount, description) 
             VALUES ((SELECT id FROM accounts WHERE name = '{}'), 
                     (SELECT id FROM accounts WHERE name = '{}'), {}, '{}')", from, to, amount, description),
            &[],
        )?;

        println!("   ✅ Batch operation: {} -> {} (${:.2})", from, to, amount);
    }

    // Commit all batch operations
    transaction.commit()?;
    println!("   ✅ Batch transaction committed");

    show_account_balances(client, "After batch operations")?;

    // Show transaction history
    println!("   📜 Transaction history:");
    let rows = client.query("
        SELECT t.amount::float8, t.description, t.created_at::text,
               a1.name as from_account, a2.name as to_account
        FROM transactions t
        JOIN accounts a1 ON t.from_account_id = a1.id
        JOIN accounts a2 ON t.to_account_id = a2.id
        ORDER BY t.created_at DESC
        LIMIT 5
    ", &[])?;

    for row in &rows {
        let amount: f64 = row.get(0);
        let description: &str = row.get(1);
        let created_at: &str = row.get(2);
        let from_account: &str = row.get(3);
        let to_account: &str = row.get(4);
        
        println!("      - {} -> {}: ${:.2} ({}) at {}", 
                 from_account, to_account, amount, description, 
                 created_at);
    }

    Ok(())
}

/// Helper function to show account balances
fn show_account_balances(client: &mut Client, title: &str) -> Result<(), Error> {
    println!("   📊 {}:", title);
    let rows = client.query("SELECT name, balance::float8 FROM accounts ORDER BY name", &[])?;
    for row in &rows {
        let name: &str = row.get(0);
        let balance: f64 = row.get(1);
        println!("      - {}: ${:.2}", name, balance);
    }
    Ok(())
}

/// Cleanup test tables
fn cleanup_test_tables(client: &mut Client) -> Result<(), Error> {
    println!("\n🗑️  Cleaning up test tables...");
    client.batch_execute("
        DROP TABLE IF EXISTS transactions CASCADE;
        DROP TABLE IF EXISTS accounts CASCADE;
    ")?;
    println!("   ✅ Test tables dropped");
    Ok(())
}
