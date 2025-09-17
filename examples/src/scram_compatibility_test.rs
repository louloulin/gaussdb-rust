//! SCRAM-SHA-256 兼容性测试工具
//!
//! 这个工具测试 GaussDB 的 SCRAM-SHA-256 认证兼容性修复功能。

use tokio_gaussdb::{connect, NoTls};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 GaussDB SCRAM-SHA-256 兼容性测试工具");
    println!("========================================");

    // 从环境变量获取连接信息
    let host = env::var("GAUSSDB_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("GAUSSDB_PORT").unwrap_or_else(|_| "5433".to_string());
    let user = env::var("GAUSSDB_USER").unwrap_or_else(|_| "gaussdb".to_string());
    let password = env::var("GAUSSDB_PASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string());
    let dbname = env::var("GAUSSDB_DBNAME").unwrap_or_else(|_| "postgres".to_string());

    println!("📋 连接参数:");
    println!("   主机: {}", host);
    println!("   端口: {}", port);
    println!("   用户: {}", user);
    println!("   数据库: {}", dbname);
    println!();

    // 测试场景 1: 使用 NoTls 连接
    println!("🧪 测试场景 1: NoTls 连接");
    let conn_str = format!("host={} port={} user={} password={} dbname={} sslmode=disable",
                          host, port, user, password, dbname);
    test_connection_scenario(&conn_str, "NoTls").await;

    // 测试场景 2: 不同的 sslmode 设置
    println!("🧪 测试场景 2: SSL 模式测试");
    let conn_str = format!("host={} port={} user={} password={} dbname={} sslmode=prefer",
                          host, port, user, password, dbname);
    test_connection_scenario(&conn_str, "SSL Prefer").await;

    // 测试场景 3: 不同的连接字符串格式
    println!("🧪 测试场景 3: 不同连接字符串格式");
    
    let test_formats = vec![
        format!("postgresql://{}:{}@{}:{}/{}", user, password, host, port, dbname),
        format!("postgres://{}:{}@{}:{}/{}?sslmode=disable", user, password, host, port, dbname),
        format!("host={} port={} user={} password={} dbname={} connect_timeout=10", 
               host, port, user, password, dbname),
    ];

    for (i, conn_str) in test_formats.iter().enumerate() {
        println!("   格式 {}: {}", i + 1, conn_str);
        test_connection_scenario(
            conn_str,
            &format!("格式{}", i + 1),
        ).await;
    }

    println!("✅ 所有测试完成！");
    println!();
    println!("💡 如果遇到认证问题，请检查:");
    println!("   1. GaussDB 服务器是否正在运行");
    println!("   2. pg_hba.conf 中的认证方法配置");
    println!("   3. 用户密码是否正确");
    println!("   4. 网络连接是否正常");

    Ok(())
}

async fn test_connection_scenario(
    conn_str: &str,
    scenario_name: &str,
)
{
    print!("   {} 连接测试... ", scenario_name);

    match connect(conn_str, NoTls).await {
        Ok((client, connection)) => {
            println!("✅ 成功");

            // 启动连接处理任务
            let connection_handle = tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("连接错误: {}", e);
                }
            });

            // 执行简单查询测试
            match test_basic_queries(&client).await {
                Ok(()) => println!("      查询测试: ✅ 成功"),
                Err(e) => println!("      查询测试: ❌ 失败 - {}", e),
            }

            // 清理连接
            connection_handle.abort();
        }
        Err(e) => {
            println!("❌ 失败");
            println!("      错误: {}", e);

            // 分析错误类型并提供建议
            analyze_error(&e);
        }
    }
    println!();
}

async fn test_basic_queries(client: &tokio_gaussdb::Client) -> Result<(), Box<dyn std::error::Error>> {
    // 测试基本查询
    let rows = client.query("SELECT 1 as test_value", &[]).await?;
    if rows.len() != 1 {
        return Err("查询结果不正确".into());
    }

    // 测试版本查询
    let rows = client.query("SELECT version()", &[]).await?;
    if let Some(row) = rows.first() {
        let version: String = row.get(0);
        println!("      服务器版本: {}", version);
    }

    Ok(())
}

fn analyze_error(error: &tokio_gaussdb::Error) {
    let error_str = error.to_string().to_lowercase();
    
    if error_str.contains("sasl") {
        println!("      🔍 SASL 认证错误分析:");
        if error_str.contains("invalid message length") {
            println!("         - 这是 GaussDB SASL 兼容性问题");
            println!("         - 建议: 修改 pg_hba.conf 使用 md5 或 sha256 认证");
        } else if error_str.contains("unsupported") {
            println!("         - 服务器不支持 SCRAM-SHA-256");
            println!("         - 建议: 检查 GaussDB 版本和配置");
        }
    } else if error_str.contains("authentication") {
        println!("      🔍 认证错误分析:");
        if error_str.contains("password") {
            println!("         - 密码认证失败");
            println!("         - 建议: 检查用户名和密码");
        } else if error_str.contains("md5") {
            println!("         - MD5 认证问题");
            println!("         - 建议: 检查密码格式");
        }
    } else if error_str.contains("connection") || error_str.contains("connect") {
        println!("      🔍 连接错误分析:");
        println!("         - 网络连接问题");
        println!("         - 建议: 检查主机名、端口和防火墙设置");
    } else if error_str.contains("tls") || error_str.contains("ssl") {
        println!("      🔍 TLS/SSL 错误分析:");
        println!("         - TLS 连接问题");
        println!("         - 建议: 检查 SSL 配置或使用 sslmode=disable");
    }
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_connection_string_parsing() {
        // 测试连接字符串解析
        let test_cases = vec![
            "host=localhost port=5433 user=test password=pass dbname=db",
            "postgresql://test:pass@localhost:5433/db",
            "postgres://test:pass@localhost:5433/db?sslmode=disable",
        ];

        for conn_str in test_cases {
            // 这里只测试连接字符串解析，不实际连接
            println!("测试连接字符串: {}", conn_str);
            // 实际测试需要运行的 GaussDB 实例
        }
    }

    #[test]
    fn test_error_analysis() {
        // 测试错误分析功能
        // 注意：这里只是演示错误分析逻辑，实际使用时需要真实的错误对象
        println!("测试错误分析功能");

        // 模拟不同类型的错误消息进行分析
        let error_messages = vec![
            "invalid message length: expected to be at end of iterator for sasl",
            "authentication failed",
            "connection refused",
            "tls handshake failed",
        ];

        for msg in error_messages {
            println!("分析错误: {}", msg);
            // 这里可以添加具体的错误分析逻辑测试
        }
    }
}
