//! GaussDB 压力测试示例
//!
//! 测试在高并发情况下的 SCRAM 兼容性和连接稳定性

use tokio_gaussdb::{connect, NoTls};
use std::env;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 GaussDB 压力测试");
    println!("==================");

    let host = env::var("GAUSSDB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("GAUSSDB_PORT").unwrap_or_else(|_| "5433".to_string());
    let user = env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string());
    let password = env::var("GAUSSDB_PASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string());
    let dbname = env::var("GAUSSDB_DBNAME").unwrap_or_else(|_| "postgres".to_string());

    let conn_str = format!("host={} port={} user={} password={} dbname={} sslmode=disable", 
                          host, port, user, password, dbname);

    println!("📋 测试参数:");
    println!("   连接字符串: host={} port={} user={} dbname={}", host, port, user, dbname);
    println!();

    // 测试 1: 连接稳定性测试
    println!("🧪 测试 1: 连接稳定性测试");
    test_connection_stability(&conn_str, 10).await?;

    // 测试 2: 并发连接测试
    println!("🧪 测试 2: 并发连接测试");
    test_concurrent_connections(&conn_str, 5).await?;

    // 测试 3: 长时间运行测试
    println!("🧪 测试 3: 长时间运行测试");
    test_long_running_connection(&conn_str).await?;

    // 测试 4: 认证重试测试
    println!("🧪 测试 4: 认证重试测试");
    test_auth_retry(&conn_str).await?;

    println!("✅ 所有压力测试完成！");
    Ok(())
}

/// 测试连接稳定性 - 重复连接和断开
async fn test_connection_stability(conn_str: &str, iterations: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("   测试重复连接和断开 {} 次...", iterations);
    
    let start_time = Instant::now();
    let mut success_count = 0;
    let mut error_count = 0;

    for i in 1..=iterations {
        match connect(conn_str, NoTls).await {
            Ok((client, connection)) => {
                success_count += 1;
                
                // 启动连接任务
                let conn_handle = tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("连接 {} 错误: {}", i, e);
                    }
                });

                // 执行简单查询
                match client.query("SELECT 1", &[]).await {
                    Ok(_) => print!("✅"),
                    Err(e) => {
                        print!("❌");
                        eprintln!("查询 {} 失败: {}", i, e);
                        error_count += 1;
                    }
                }

                // 清理连接
                conn_handle.abort();
            }
            Err(e) => {
                error_count += 1;
                print!("❌");
                eprintln!("连接 {} 失败: {}", i, e);
            }
        }

        if i % 10 == 0 {
            println!(" ({}/{})", i, iterations);
        }
    }

    let duration = start_time.elapsed();
    println!();
    println!("   结果: 成功 {}, 失败 {}, 耗时 {:?}", success_count, error_count, duration);
    println!("   平均连接时间: {:?}", duration / iterations as u32);
    println!();

    Ok(())
}

/// 测试并发连接
async fn test_concurrent_connections(conn_str: &str, concurrent_count: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("   测试 {} 个并发连接...", concurrent_count);
    
    let start_time = Instant::now();
    let conn_str = Arc::new(conn_str.to_string());
    
    let mut handles = Vec::new();
    
    for i in 1..=concurrent_count {
        let conn_str_clone = Arc::clone(&conn_str);
        let handle = tokio::spawn(async move {
            let result = connect(&conn_str_clone, NoTls).await;
            match result {
                Ok((client, connection)) => {
                    // 启动连接任务
                    let conn_handle = tokio::spawn(async move {
                        if let Err(e) = connection.await {
                            eprintln!("并发连接 {} 错误: {}", i, e);
                        }
                    });

                    // 执行查询
                    let query_result = client.query("SELECT $1::int as id, 'concurrent_test' as name", &[&(i as i32)]).await;
                    
                    // 清理
                    conn_handle.abort();
                    
                    match query_result {
                        Ok(rows) => {
                            if let Some(row) = rows.first() {
                                let id: i32 = row.get(0);
                                let name: String = row.get(1);
                                (true, format!("连接 {}: id={}, name={}", i, id, name))
                            } else {
                                (false, format!("连接 {} 查询无结果", i))
                            }
                        }
                        Err(e) => (false, format!("连接 {} 查询失败: {}", i, e))
                    }
                }
                Err(e) => (false, format!("连接 {} 失败: {}", i, e))
            }
        });
        handles.push(handle);
    }

    // 等待所有连接完成
    let mut success_count = 0;
    let mut error_count = 0;
    
    for handle in handles {
        match handle.await {
            Ok((success, message)) => {
                if success {
                    success_count += 1;
                    println!("   ✅ {}", message);
                } else {
                    error_count += 1;
                    println!("   ❌ {}", message);
                }
            }
            Err(e) => {
                error_count += 1;
                println!("   ❌ 任务执行失败: {}", e);
            }
        }
    }

    let duration = start_time.elapsed();
    println!("   并发测试结果: 成功 {}, 失败 {}, 总耗时 {:?}", success_count, error_count, duration);
    println!();

    Ok(())
}

/// 测试长时间运行连接
async fn test_long_running_connection(conn_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("   测试长时间运行连接 (30秒)...");
    
    let (client, connection) = connect(conn_str, NoTls).await?;
    
    // 启动连接任务
    let conn_handle = tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("长时间连接错误: {}", e);
        }
    });

    let start_time = Instant::now();
    let mut query_count = 0;
    let mut error_count = 0;

    // 运行 30 秒
    while start_time.elapsed() < Duration::from_secs(30) {
        match client.query("SELECT NOW(), $1::int", &[&query_count]).await {
            Ok(rows) => {
                query_count += 1;
                if let Some(row) = rows.first() {
                    let count: i32 = row.get(1);
                    if query_count % 10 == 0 {
                        println!("   📊 已执行 {} 次查询, 最新: {}", query_count, count);
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                println!("   ❌ 查询 {} 失败: {}", query_count, e);
            }
        }
        
        // 短暂休息
        sleep(Duration::from_millis(100)).await;
    }

    // 清理连接
    conn_handle.abort();

    println!("   长时间测试结果: 执行 {} 次查询, {} 次错误, 耗时 {:?}", 
             query_count, error_count, start_time.elapsed());
    println!();

    Ok(())
}

/// 测试认证重试机制
async fn test_auth_retry(conn_str: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("   测试认证重试机制...");
    
    // 测试正确的认证
    match connect(conn_str, NoTls).await {
        Ok((client, connection)) => {
            println!("   ✅ 正确认证成功");
            
            let conn_handle = tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("认证测试连接错误: {}", e);
                }
            });

            // 执行查询验证
            match client.query("SELECT current_user", &[]).await {
                Ok(rows) => {
                    if let Some(row) = rows.first() {
                        let user: String = row.get(0);
                        println!("   📋 当前用户: {}", user);
                    }
                }
                Err(e) => println!("   ❌ 用户查询失败: {}", e),
            }

            conn_handle.abort();
        }
        Err(e) => println!("   ❌ 正确认证失败: {}", e),
    }

    // 测试错误的认证（预期失败）
    let wrong_conn_str = conn_str.replace("password=Gaussdb@123", "password=wrong_password");
    match connect(&wrong_conn_str, NoTls).await {
        Ok(_) => println!("   ⚠️  错误密码竟然成功了（可能是 trust 认证）"),
        Err(e) => println!("   ✅ 错误密码正确失败: {}", e),
    }

    println!();
    Ok(())
}
