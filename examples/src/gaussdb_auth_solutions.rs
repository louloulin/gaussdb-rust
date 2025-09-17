//! GaussDB认证问题解决方案示例
//!
//! 展示如何处理GaussDB特有的认证问题

use tokio_gaussdb::{connect, Config, NoTls};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 GaussDB认证问题解决方案");
    println!("================================");

    let host = "localhost";
    let port = 5433;
    let user = "gaussdb";
    let password = "Gaussdb@123";
    let dbname = "postgres";

    // 解决方案1: 使用不同的连接字符串格式
    println!("🧪 解决方案1: 优化连接字符串");
    let connection_strings = vec![
        // 基本连接字符串
        format!("host={} port={} user={} password={} dbname={}", 
            host, port, user, password, dbname),
        
        // 显式禁用SSL和SCRAM
        format!("host={} port={} user={} password={} dbname={} sslmode=disable", 
            host, port, user, password, dbname),
        
        // 指定认证方法偏好
        format!("host={} port={} user={} password={} dbname={} sslmode=disable gssencmode=disable", 
            host, port, user, password, dbname),
        
        // 使用IP地址而不是localhost
        format!("host=127.0.0.1 port={} user={} password={} dbname={} sslmode=disable", 
            port, user, password, dbname),
    ];

    for (i, conn_str) in connection_strings.iter().enumerate() {
        println!("  测试连接字符串 {} ...", i + 1);
        match connect(conn_str, NoTls).await {
            Ok((client, connection)) => {
                println!("    ✅ 连接成功！");
                
                let connection_handle = tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("Connection error: {}", e);
                    }
                });

                // 测试基本操作
                if let Ok(rows) = client.query("SELECT current_user, version()", &[]).await {
                    let current_user: &str = rows[0].get(0);
                    let version: &str = rows[0].get(1);
                    println!("    用户: {}", current_user);
                    println!("    版本: {}", version.split_whitespace().take(3).collect::<Vec<_>>().join(" "));
                }

                drop(client);
                let _ = connection_handle.await;
                
                println!("    🎉 找到可用的连接方式！");
                break;
            }
            Err(e) => {
                println!("    ❌ 失败: {}", e);
                if e.to_string().contains("sasl") {
                    println!("      → SASL认证错误，尝试下一种方式");
                }
            }
        }
    }

    // 解决方案2: 使用Config构建器并设置特定参数
    println!("\n🧪 解决方案2: 使用Config构建器");
    let mut config = Config::new();
    config
        .host(host)
        .port(port)
        .user(user)
        .password(password)
        .dbname(dbname)
        .application_name("gaussdb-rust-test")
        .connect_timeout(std::time::Duration::from_secs(10));

    match config.connect(NoTls).await {
        Ok((client, connection)) => {
            println!("  ✅ Config构建器连接成功！");
            
            let connection_handle = tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("Connection error: {}", e);
                }
            });

            drop(client);
            let _ = connection_handle.await;
        }
        Err(e) => {
            println!("  ❌ Config构建器连接失败: {}", e);
        }
    }

    println!("\n📋 故障排除指南:");
    println!("如果仍然遇到SASL认证错误，请检查以下配置：");
    println!();
    println!("1. 检查GaussDB的pg_hba.conf文件：");
    println!("   sudo find /opt -name pg_hba.conf 2>/dev/null");
    println!("   # 或者");
    println!("   sudo find /usr/local -name pg_hba.conf 2>/dev/null");
    println!();
    println!("2. 推荐的pg_hba.conf配置：");
    println!("   # 使用MD5认证（兼容性最好）");
    println!("   host    all             gaussdb         127.0.0.1/32            md5");
    println!("   host    all             gaussdb         ::1/128                 md5");
    println!("   ");
    println!("   # 或者使用SHA256认证（GaussDB特有）");
    println!("   host    all             gaussdb         127.0.0.1/32            sha256");
    println!("   ");
    println!("   # 临时测试可以使用trust认证");
    println!("   host    all             gaussdb         127.0.0.1/32            trust");
    println!();
    println!("3. 重启GaussDB服务：");
    println!("   sudo systemctl restart gaussdb");
    println!("   # 或者");
    println!("   gs_ctl restart -D /path/to/data");
    println!();
    println!("4. 验证用户和密码：");
    println!("   gsql -h localhost -p 5433 -U gaussdb -d postgres");

    Ok(())
}
