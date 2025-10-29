//! GaussDB Authentication Methods Example (Synchronous)
//!
//! This example demonstrates:
//! - SHA256 authentication (GaussDB specific)
//! - MD5_SHA256 authentication (GaussDB specific)
//! - Standard MD5 authentication (PostgreSQL compatible)
//! - Connection configuration
//! - Authentication error handling
//!
//! Run with: cargo run --example sync_authentication

use gaussdb::{Client, Config, Error, NoTls};
use std::env;

fn main() -> Result<(), Error> {
    println!("🔐 GaussDB Authentication Methods Demo (Synchronous)");
    println!("====================================================");

    // Test different authentication methods
    test_connection_string_auth()?;
    test_config_builder_auth()?;
    test_authentication_methods()?;
    
    println!("\n🎉 Authentication examples completed successfully!");
    
    Ok(())
}

/// Test authentication using connection string
fn test_connection_string_auth() -> Result<(), Error> {
    println!("\n1️⃣  Connection String Authentication");
    println!("-----------------------------------");

    // Get connection parameters from environment or use defaults
    let host = env::var("GAUSSDB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("GAUSSDB_PORT").unwrap_or_else(|_| "5433".to_string());
    let user = env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string());
    let password = env::var("GAUSSDB_PASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string());
    let database = env::var("GAUSSDB_DATABASE").unwrap_or_else(|_| "postgres".to_string());

    // Build connection string
    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        host, port, user, password, database
    );

    println!("🔗 Connecting with connection string...");
    println!("   Connection: {}", mask_password(&conn_str));

    match Client::connect(&conn_str, NoTls) {
        Ok(mut client) => {
            println!("   ✅ Connection successful!");
            
            // Test the connection with a simple query
            let row = client.query_one("SELECT version()", &[])?;
            let version: &str = row.get(0);
            println!("   📊 Database version: {}", version);
            
            // Check authentication method used
            if version.contains("openGauss") || version.contains("GaussDB") {
                println!("   🔐 Likely using GaussDB-specific authentication (SHA256 or MD5_SHA256)");
            } else {
                println!("   🔐 Using PostgreSQL-compatible authentication");
            }
        }
        Err(e) => {
            println!("   ❌ Connection failed: {}", e);
            println!("   💡 This might be due to:");
            println!("      - Incorrect credentials");
            println!("      - Database not running");
            println!("      - Network connectivity issues");
            println!("      - Authentication method mismatch");
        }
    }

    Ok(())
}

/// Test authentication using Config builder
fn test_config_builder_auth() -> Result<(), Error> {
    println!("\n2️⃣  Config Builder Authentication");
    println!("--------------------------------");

    // Build configuration step by step
    let mut config = Config::new();
    config
        .host(&env::var("GAUSSDB_HOST").unwrap_or_else(|_| "localhost".to_string()))
        .port(env::var("GAUSSDB_PORT").unwrap_or_else(|_| "5433".to_string()).parse().unwrap_or(5433))
        .user(&env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string()))
        .password(&env::var("GAUSSDB_PASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string()))
        .dbname(&env::var("GAUSSDB_DATABASE").unwrap_or_else(|_| "postgres".to_string()));

    println!("🔗 Connecting with Config builder...");
    println!("   Host: {:?}", config.get_hosts()[0]);
    println!("   Port: {}", config.get_ports()[0]);
    println!("   User: {}", config.get_user().unwrap_or("(not set)"));
    println!("   Database: {}", config.get_dbname().unwrap_or("(not set)"));

    match config.connect(NoTls) {
        Ok(mut client) => {
            println!("   ✅ Connection successful!");
            
            // Get current user and database
            let row = client.query_one("SELECT current_user, current_database()", &[])?;
            let current_user: &str = row.get(0);
            let current_db: &str = row.get(1);
            println!("   👤 Connected as: {}", current_user);
            println!("   🗄️  Database: {}", current_db);
            
            // Test authentication by checking user privileges
            let rows = client.query("
                SELECT 
                    rolname,
                    rolsuper,
                    rolcreaterole,
                    rolcreatedb,
                    rolcanlogin
                FROM pg_roles 
                WHERE rolname = current_user
            ", &[])?;
            
            if let Some(row) = rows.first() {
                let rolname: &str = row.get(0);
                let rolsuper: bool = row.get(1);
                let rolcreaterole: bool = row.get(2);
                let rolcreatedb: bool = row.get(3);
                let rolcanlogin: bool = row.get(4);
                
                println!("   🔑 User privileges for '{}':", rolname);
                println!("      - Superuser: {}", if rolsuper { "Yes" } else { "No" });
                println!("      - Create roles: {}", if rolcreaterole { "Yes" } else { "No" });
                println!("      - Create databases: {}", if rolcreatedb { "Yes" } else { "No" });
                println!("      - Can login: {}", if rolcanlogin { "Yes" } else { "No" });
            }
        }
        Err(e) => {
            println!("   ❌ Connection failed: {}", e);
            analyze_auth_error(&e);
        }
    }

    Ok(())
}

/// Test different authentication scenarios
fn test_authentication_methods() -> Result<(), Error> {
    println!("\n3️⃣  Authentication Method Testing");
    println!("--------------------------------");

    // Test 1: Valid credentials
    println!("\n🧪 Test 1: Valid credentials");
    test_auth_scenario(
        "localhost",
        5433,
        &env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string()),
        &env::var("GAUSSDB_PASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string()),
        "postgres",
    );

    // Test 2: Invalid password
    println!("\n🧪 Test 2: Invalid password");
    test_auth_scenario(
        "localhost",
        5433,
        &env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string()),
        "wrong_password",
        "postgres",
    );

    // Test 3: Invalid user
    println!("\n🧪 Test 3: Invalid user");
    test_auth_scenario(
        "localhost",
        5433,
        "nonexistent_user",
        "any_password",
        "postgres",
    );

    // Test 4: Invalid database
    println!("\n🧪 Test 4: Invalid database");
    test_auth_scenario(
        "localhost",
        5433,
        &env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string()),
        &env::var("GAUSSDB_PASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string()),
        "nonexistent_db",
    );

    Ok(())
}

/// Test a specific authentication scenario
fn test_auth_scenario(host: &str, port: u16, user: &str, password: &str, dbname: &str) {
    let mut config = Config::new();
    config
        .host(host)
        .port(port)
        .user(user)
        .password(password)
        .dbname(dbname);

    println!("   🔗 Attempting connection:");
    println!("      Host: {}", host);
    println!("      Port: {}", port);
    println!("      User: {}", user);
    println!("      Password: {}", if password.len() > 0 { "***" } else { "(empty)" });
    println!("      Database: {}", dbname);

    match config.connect(NoTls) {
        Ok(mut client) => {
            println!("      ✅ Connection successful!");
            
            // Quick test query
            if let Ok(row) = client.query_one("SELECT 1 as test", &[]) {
                let test: i32 = row.get(0);
                println!("      ✅ Test query result: {}", test);
            }
        }
        Err(e) => {
            println!("      ❌ Connection failed: {}", e);
            analyze_auth_error(&e);
        }
    }
}

/// Analyze authentication errors and provide helpful suggestions
fn analyze_auth_error(error: &Error) {
    let error_msg = error.to_string().to_lowercase();
    
    println!("      🔍 Error analysis:");
    
    if error_msg.contains("password authentication failed") {
        println!("         - This is an authentication failure");
        println!("         - Check username and password");
        println!("         - Verify user exists and has login privileges");
        println!("         - Check pg_hba.conf authentication method");
    } else if error_msg.contains("database") && error_msg.contains("does not exist") {
        println!("         - The specified database does not exist");
        println!("         - Use an existing database (e.g., 'postgres')");
        println!("         - Create the database first if needed");
    } else if error_msg.contains("role") && error_msg.contains("does not exist") {
        println!("         - The specified user/role does not exist");
        println!("         - Check the username spelling");
        println!("         - Create the user first if needed");
    } else if error_msg.contains("connection refused") || error_msg.contains("could not connect") {
        println!("         - Database server is not running or not accessible");
        println!("         - Check if GaussDB/OpenGauss is running");
        println!("         - Verify host and port settings");
        println!("         - Check firewall and network connectivity");
    } else if error_msg.contains("ssl") || error_msg.contains("tls") {
        println!("         - SSL/TLS configuration issue");
        println!("         - Try with sslmode=disable for testing");
        println!("         - Check SSL certificate configuration");
    } else {
        println!("         - Unknown authentication error");
        println!("         - Check database logs for more details");
        println!("         - Verify all connection parameters");
    }
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
