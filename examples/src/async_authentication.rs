//! GaussDB Authentication Methods Example (Asynchronous)
//!
//! This example demonstrates:
//! - SHA256 authentication (GaussDB specific)
//! - MD5_SHA256 authentication (GaussDB specific)
//! - Standard MD5 authentication (PostgreSQL compatible)
//! - Async connection configuration
//! - Authentication error handling
//! - Connection pooling considerations
//!
//! Run with: cargo run --example async_authentication

use tokio_gaussdb::{connect, Config, Error, NoTls};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("🔐 GaussDB Authentication Methods Demo (Asynchronous)");
    println!("=====================================================");

    // Test different authentication methods
    test_connection_string_auth().await?;
    test_config_builder_auth().await?;
    test_concurrent_connections().await?;
    test_authentication_methods().await?;
    
    println!("\n🎉 Asynchronous authentication examples completed successfully!");
    
    Ok(())
}

/// Test authentication using connection string
async fn test_connection_string_auth() -> Result<(), Error> {
    println!("\n1️⃣  Connection String Authentication (Async)");
    println!("--------------------------------------------");

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

    println!("🔗 Connecting asynchronously with connection string...");
    println!("   Connection: {}", mask_password(&conn_str));

    match connect(&conn_str, NoTls).await {
        Ok((client, connection)) => {
            // Spawn connection task
            let connection_handle = tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("Connection error: {}", e);
                }
            });

            println!("   ✅ Async connection successful!");
            
            // Test the connection with a simple query
            let row = client.query_one("SELECT version(), current_timestamp::text", &[]).await?;
            let version: &str = row.get(0);
            let timestamp: &str = row.get(1);
            
            println!("   📊 Database version: {}", version);
            println!("   🕐 Server time: {}", timestamp);
            
            // Check authentication method used
            if version.contains("openGauss") || version.contains("GaussDB") {
                println!("   🔐 Using GaussDB-specific authentication");
                
                // Try to get authentication method info
                if let Ok(auth_rows) = client.query("
                    SELECT setting 
                    FROM pg_settings 
                    WHERE name = 'password_encryption_type'
                ", &[]).await {
                    if let Some(auth_row) = auth_rows.first() {
                        let auth_method: &str = auth_row.get(0);
                        println!("   🔑 Password encryption type: {}", auth_method);
                    }
                }
            } else {
                println!("   🔐 Using PostgreSQL-compatible authentication");
            }

            // Clean up
            drop(client);
            connection_handle.await.unwrap();
        }
        Err(e) => {
            println!("   ❌ Async connection failed: {}", e);
            analyze_auth_error(&e);
        }
    }

    Ok(())
}

/// Test authentication using Config builder
async fn test_config_builder_auth() -> Result<(), Error> {
    println!("\n2️⃣  Config Builder Authentication (Async)");
    println!("------------------------------------------");

    // Build configuration step by step
    let mut config = Config::new();
    config
        .host(&env::var("GAUSSDB_HOST").unwrap_or_else(|_| "localhost".to_string()))
        .port(env::var("GAUSSDB_PORT").unwrap_or_else(|_| "5433".to_string()).parse().unwrap_or(5433))
        .user(&env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string()))
        .password(&env::var("GAUSSDB_PASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string()))
        .dbname(&env::var("GAUSSDB_DATABASE").unwrap_or_else(|_| "postgres".to_string()));

    println!("🔗 Connecting asynchronously with Config builder...");
    println!("   Host: {:?}", config.get_hosts()[0]);
    println!("   Port: {}", config.get_ports()[0]);
    println!("   User: {}", config.get_user().unwrap_or("(not set)"));
    println!("   Database: {}", config.get_dbname().unwrap_or("(not set)"));

    match config.connect(NoTls).await {
        Ok((client, connection)) => {
            // Spawn connection task
            let connection_handle = tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("Connection error: {}", e);
                }
            });

            println!("   ✅ Async connection successful!");
            
            // Get current user and database
            let row = client.query_one("SELECT current_user, current_database(), inet_server_addr(), inet_server_port()", &[]).await?;
            let current_user: &str = row.get(0);
            let current_db: &str = row.get(1);
            let server_addr: Option<std::net::IpAddr> = row.get(2);
            let server_port: Option<i32> = row.get(3);
            
            println!("   👤 Connected as: {}", current_user);
            println!("   🗄️  Database: {}", current_db);
            println!("   🌐 Server: {}:{}", 
                     server_addr.map(|a| a.to_string()).unwrap_or_else(|| "localhost".to_string()),
                     server_port.unwrap_or(5433));
            
            // Test concurrent queries for user info
            let (roles_result, settings_result) = tokio::join!(
                client.query("
                    SELECT 
                        rolname,
                        rolsuper,
                        rolcreaterole,
                        rolcreatedb,
                        rolcanlogin
                    FROM pg_roles 
                    WHERE rolname = current_user
                ", &[]),
                client.query("
                    SELECT name, setting, short_desc 
                    FROM pg_settings 
                    WHERE name IN ('max_connections', 'shared_buffers', 'effective_cache_size')
                    ORDER BY name
                ", &[])
            );
            
            // Display user privileges
            if let Ok(roles) = roles_result {
                if let Some(row) = roles.first() {
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

            // Display server settings
            if let Ok(settings) = settings_result {
                println!("   ⚙️  Server settings:");
                for row in &settings {
                    let name: &str = row.get(0);
                    let setting: &str = row.get(1);
                    let desc: &str = row.get(2);
                    println!("      - {}: {} ({})", name, setting, desc);
                }
            }

            // Clean up
            drop(client);
            connection_handle.await.unwrap();
        }
        Err(e) => {
            println!("   ❌ Async connection failed: {}", e);
            analyze_auth_error(&e);
        }
    }

    Ok(())
}

/// Test concurrent connections
async fn test_concurrent_connections() -> Result<(), Error> {
    println!("\n3️⃣  Concurrent Connections Test");
    println!("------------------------------");

    let conn_str = format!(
        "host={} port={} user={} password={} dbname={}",
        env::var("GAUSSDB_HOST").unwrap_or_else(|_| "localhost".to_string()),
        env::var("GAUSSDB_PORT").unwrap_or_else(|_| "5433".to_string()),
        env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string()),
        env::var("GAUSSDB_PASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string()),
        env::var("GAUSSDB_DATABASE").unwrap_or_else(|_| "postgres".to_string())
    );

    println!("🔗 Creating 3 concurrent connections...");

    // Create multiple concurrent connections
    let connection_tasks = (1..=3).map(|i| {
        let conn_str = conn_str.clone();
        tokio::spawn(async move {
            match connect(&conn_str, NoTls).await {
                Ok((client, connection)) => {
                    // Spawn connection task
                    let connection_handle = tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            eprintln!("Connection {} error: {}", i, e);
                        }
                    });

                    // Test query
                    let result = client.query_one("SELECT $1::int as connection_id, pg_backend_pid() as pid", &[&i]).await;
                    
                    // Clean up
                    drop(client);
                    connection_handle.await.unwrap();
                    
                    result
                }
                Err(e) => Err(e),
            }
        })
    }).collect::<Vec<_>>();

    // Wait for all connections to complete
    let results = futures_util::future::join_all(connection_tasks).await;
    
    println!("   📊 Connection results:");
    for (i, result) in results.into_iter().enumerate() {
        match result {
            Ok(Ok(row)) => {
                let conn_id: i32 = row.get(0);
                let pid: i64 = row.get(1);
                println!("   ✅ Connection {}: ID={}, PID={}", i + 1, conn_id, pid);
            }
            Ok(Err(e)) => {
                println!("   ❌ Connection {}: Query failed - {}", i + 1, e);
            }
            Err(e) => {
                println!("   ❌ Connection {}: Task failed - {}", i + 1, e);
            }
        }
    }

    Ok(())
}

/// Test different authentication scenarios
async fn test_authentication_methods() -> Result<(), Error> {
    println!("\n4️⃣  Authentication Method Testing (Async)");
    println!("------------------------------------------");

    // Test scenarios with different credentials
    let test_cases = vec![
        ("Valid credentials", 
         env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string()),
         env::var("GAUSSDB_PASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string()),
         "postgres"),
        ("Invalid password", 
         env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string()),
         "wrong_password".to_string(),
         "postgres"),
        ("Invalid user", 
         "nonexistent_user".to_string(),
         "any_password".to_string(),
         "postgres"),
        ("Invalid database", 
         env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string()),
         env::var("GAUSSDB_PASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string()),
         "nonexistent_db"),
    ];

    // Test all scenarios concurrently
    let test_tasks = test_cases.into_iter().enumerate().map(|(i, (desc, user, password, dbname))| {
        tokio::spawn(async move {
            println!("\n🧪 Test {}: {}", i + 1, desc);
            test_auth_scenario_async("localhost", 5433, &user, &password, dbname).await;
        })
    }).collect::<Vec<_>>();

    // Wait for all tests to complete
    futures_util::future::join_all(test_tasks).await;

    Ok(())
}

/// Test a specific authentication scenario asynchronously
async fn test_auth_scenario_async(host: &str, port: u16, user: &str, password: &str, dbname: &str) {
    let mut config = Config::new();
    config
        .host(host)
        .port(port)
        .user(user)
        .password(password)
        .dbname(dbname);

    println!("   🔗 Attempting async connection:");
    println!("      Host: {}", host);
    println!("      Port: {}", port);
    println!("      User: {}", user);
    println!("      Password: {}", if password.len() > 0 { "***" } else { "(empty)" });
    println!("      Database: {}", dbname);

    match config.connect(NoTls).await {
        Ok((client, connection)) => {
            // Spawn connection task
            let connection_handle = tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("Connection error: {}", e);
                }
            });

            println!("      ✅ Async connection successful!");
            
            // Quick test query
            if let Ok(row) = client.query_one("SELECT 1 as test, now()::text as timestamp", &[]).await {
                let test: i32 = row.get(0);
                let timestamp: &str = row.get(1);
                println!("      ✅ Test query result: {}, timestamp: {}", test, timestamp);
            }

            // Clean up
            drop(client);
            connection_handle.await.unwrap();
        }
        Err(e) => {
            println!("      ❌ Async connection failed: {}", e);
            analyze_auth_error(&e);
        }
    }
}

/// Analyze authentication errors and provide helpful suggestions
fn analyze_auth_error(error: &Error) {
    let error_msg = error.to_string().to_lowercase();
    
    println!("      🔍 Error analysis:");
    
    if error_msg.contains("password authentication failed") {
        println!("         - Authentication failure (wrong credentials)");
        println!("         - For GaussDB: Check SHA256/MD5_SHA256 authentication");
        println!("         - Verify user exists and has login privileges");
        println!("         - Check pg_hba.conf authentication method configuration");
    } else if error_msg.contains("database") && error_msg.contains("does not exist") {
        println!("         - Database does not exist");
        println!("         - Use 'postgres' database for testing");
        println!("         - Create database first: CREATE DATABASE dbname;");
    } else if error_msg.contains("role") && error_msg.contains("does not exist") {
        println!("         - User/role does not exist");
        println!("         - Create user: CREATE USER username WITH PASSWORD 'password';");
        println!("         - Grant login: ALTER USER username WITH LOGIN;");
    } else if error_msg.contains("connection refused") || error_msg.contains("could not connect") {
        println!("         - Database server not accessible");
        println!("         - Check if GaussDB/OpenGauss is running on port 5433");
        println!("         - Verify network connectivity and firewall settings");
        println!("         - For Docker: docker ps to check container status");
    } else if error_msg.contains("timeout") {
        println!("         - Connection timeout");
        println!("         - Server may be overloaded or network is slow");
        println!("         - Try increasing connection timeout");
    } else {
        println!("         - Unknown error: {}", error);
        println!("         - Check database server logs");
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
