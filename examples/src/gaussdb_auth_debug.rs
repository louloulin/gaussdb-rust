//! GaussDB认证问题诊断工具

use tokio_gaussdb::{connect, NoTls};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 GaussDB认证问题诊断工具");
    println!("================================");

    let host = "localhost";
    let port = 5433;
    let user = "gaussdb";
    let password = "Gaussdb@123";
    let dbname = "postgres";

    println!("📋 测试配置:");
    println!("  Host: {}", host);
    println!("  Port: {}", port);
    println!("  User: {}", user);
    println!("  Password: {}", password);
    println!("  Database: {}", dbname);
    println!();

    // 测试基本连接
    println!("🧪 测试: 基本连接 (NoTls)");
    let conn_str = format!("host={} port={} user={} password={} dbname={}", 
        host, port, user, password, dbname);
    
    print!("  连接中 ... ");
    match connect(&conn_str, NoTls).await {
        Ok((client, connection)) => {
            println!("✅ 连接成功");
            
            let connection_handle = tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("Connection error: {}", e);
                }
            });

            match client.query("SELECT 1", &[]).await {
                Ok(_) => println!("    查询测试: ✅ 成功"),
                Err(e) => println!("    查询测试: ❌ 失败 - {}", e),
            }

            if let Ok(rows) = client.query("SELECT version()", &[]).await {
                if let Ok(version) = rows[0].try_get::<_, &str>(0) {
                    println!("    数据库版本: {}", version.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
                }
            }

            drop(client);
            let _ = connection_handle.await;
        }
        Err(e) => {
            println!("❌ 连接失败");
            println!("    错误: {}", e);
            
            let error_str = e.to_string();
            if error_str.contains("sasl") {
                println!("    🔍 这是SASL认证错误 - 可能是认证方法不兼容");
                println!("    💡 建议: 检查GaussDB的pg_hba.conf配置，尝试使用md5或sha256认证");
            } else if error_str.contains("password") {
                println!("    �� 这是密码认证错误 - 检查用户名密码");
            } else if error_str.contains("connection") {
                println!("    🔍 这是连接错误 - 检查网络和服务状态");
            }
        }
    }

    println!("\n📊 诊断总结:");
    println!("如果测试失败并显示SASL错误，这表明:");
    println!("1. GaussDB的SASL实现可能与标准PostgreSQL不兼容");
    println!("2. 可能需要使用GaussDB特定的认证方法");
    println!("3. 建议检查GaussDB的认证配置 (pg_hba.conf)");
    println!("\n💡 建议的解决方案:");
    println!("1. 在GaussDB中配置MD5或SHA256认证而不是SCRAM");
    println!("2. 检查pg_hba.conf中的认证方法设置");
    println!("3. 尝试使用trust认证进行测试");

    Ok(())
}
