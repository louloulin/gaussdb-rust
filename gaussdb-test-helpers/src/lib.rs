//! # GaussDB 测试辅助模块
//!
//! 这是一个全局的、统一的测试辅助 crate，为所有 GaussDB Rust 客户端测试提供环境配置管理。
//!
//! ## 设计理念
//!
//! - **单一职责**：仅负责测试环境配置管理
//! - **全局复用**：所有 crate 共享同一套配置逻辑
//! - **灵活配置**：支持环境变量、.env 文件、默认值
//!
//! ## 使用方法
//!
//! 在其他 crate 的 `Cargo.toml` 中添加：
//!
//! ```toml
//! [dev-dependencies]
//! gaussdb-test-helpers = { path = "../gaussdb-test-helpers" }
//! ```
//!
//! 在测试代码中导入：
//!
//! ```rust
//! use gaussdb_test_helpers::*;
//!
//! #[test]
//! fn my_test() {
//!     let conn_str = get_test_conn_str();
//!     // 使用连接字符串...
//! }
//! ```
//!
//! ## 环境变量
//!
//! | 变量名 | 说明 | 默认值 |
//! |--------|------|--------|
//! | `TEST_CONN_STR` | 完整连接字符串（最高优先级） | - |
//! | `PGUSER` | 数据库用户名 | `gaussdb` |
//! | `PGPASSWORD` | 数据库密码 | `Gaussdb@123` |
//! | `PGHOST` | 数据库主机 | `localhost` |
//! | `PGPORT` | 数据库端口 | `5433` |
//! | `PGDATABASE` | 数据库名称 | `postgres` |

use std::sync::Once;

static INIT: Once = Once::new();

/// 加载环境变量（仅执行一次）
///
/// 尝试从以下位置加载 .env 文件：
/// 1. 当前目录
/// 2. 父目录
/// 3. 父父目录
pub fn load_env() {
    INIT.call_once(|| {
        // 尝试多个可能的 .env 位置
        let _ = dotenvy::dotenv();
        let _ = dotenvy::from_filename("../.env");
        let _ = dotenvy::from_filename("../../.env");
        let _ = dotenvy::from_filename("../../../.env");
    });
}

/// 获取测试数据库连接字符串
///
/// 优先级：TEST_CONN_STR > 单个环境变量组合 > 默认值
///
/// # Example
///
/// ```
/// use gaussdb_test_helpers::get_test_conn_str;
///
/// let conn_str = get_test_conn_str();
/// // "user=gaussdb password=Gaussdb@123 host=localhost port=5433 dbname=postgres"
/// ```
pub fn get_test_conn_str() -> String {
    load_env();
    
    // 优先使用完整连接字符串
    if let Ok(conn_str) = std::env::var("TEST_CONN_STR") {
        return conn_str;
    }
    
    // 从单个环境变量构建连接字符串
    format!(
        "user={} password={} host={} port={} dbname={}",
        get_test_user(),
        get_test_password(),
        get_test_host(),
        get_test_port(),
        get_test_database()
    )
}

/// 获取测试数据库主机地址
pub fn get_test_host() -> String {
    load_env();
    std::env::var("PGHOST").unwrap_or_else(|_| "localhost".to_string())
}

/// 获取测试数据库端口
pub fn get_test_port() -> String {
    load_env();
    std::env::var("PGPORT").unwrap_or_else(|_| "5433".to_string())
}

/// 获取测试数据库用户名
pub fn get_test_user() -> String {
    load_env();
    std::env::var("PGUSER").unwrap_or_else(|_| "gaussdb".to_string())
}

/// 获取测试数据库密码
pub fn get_test_password() -> String {
    load_env();
    std::env::var("PGPASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string())
}

/// 获取测试数据库名称
pub fn get_test_database() -> String {
    load_env();
    std::env::var("PGDATABASE").unwrap_or_else(|_| "postgres".to_string())
}

/// 获取测试数据库主机和端口（用于 TCP 连接）
///
/// # Example
///
/// ```
/// use gaussdb_test_helpers::get_test_host_port;
///
/// let (host, port) = get_test_host_port();
/// let addr = format!("{}:{}", host, port);
/// ```
pub fn get_test_host_port() -> (String, String) {
    (get_test_host(), get_test_port())
}

/// 获取多主机连接字符串
///
/// # Example
///
/// ```
/// use gaussdb_test_helpers::get_multi_host_conn_str;
///
/// let conn_str = get_multi_host_conn_str("host1,host2", "5432,5433");
/// ```
pub fn get_multi_host_conn_str(hosts: &str, ports: &str) -> String {
    format!(
        "host={} port={} user={} password={} dbname={}",
        hosts,
        ports,
        get_test_user(),
        get_test_password(),
        get_test_database()
    )
}

/// 获取 hostaddr（IP 地址格式）
///
/// 注意：hostaddr 参数需要 IP 地址，会自动将 localhost 转换为 127.0.0.1
///
/// # Example
///
/// ```
/// use gaussdb_test_helpers::get_hostaddr;
///
/// let hostaddr = get_hostaddr();
/// // 如果 PGHOST=localhost，返回 "127.0.0.1"
/// // 否则返回 PGHOST 的值
/// ```
pub fn get_hostaddr() -> String {
    let host = get_test_host();
    // hostaddr 需要 IP 地址，将 localhost 转换为 127.0.0.1
    if host == "localhost" {
        "127.0.0.1".to_string()
    } else {
        host
    }
}

/// 构建带有额外参数的连接字符串
///
/// # Example
///
/// ```
/// use gaussdb_test_helpers::build_conn_str_with_params;
///
/// let conn_str = build_conn_str_with_params(&[
///     ("application_name", "my_test"),
///     ("connect_timeout", "10"),
/// ]);
/// ```
pub fn build_conn_str_with_params(extra_params: &[(&str, &str)]) -> String {
    let mut conn_str = get_test_conn_str();
    for (key, value) in extra_params {
        conn_str.push_str(&format!(" {}={}", key, value));
    }
    conn_str
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_env() {
        load_env();
        // 应该能够正常执行，不会 panic
    }

    #[test]
    fn test_get_conn_str() {
        let conn_str = get_test_conn_str();
        assert!(conn_str.contains("user="));
        assert!(conn_str.contains("password="));
        assert!(conn_str.contains("host="));
        assert!(conn_str.contains("port="));
        assert!(conn_str.contains("dbname="));
    }

    #[test]
    fn test_get_host_port() {
        let (host, port) = get_test_host_port();
        assert!(!host.is_empty());
        assert!(!port.is_empty());
    }

    #[test]
    fn test_get_hostaddr() {
        let hostaddr = get_hostaddr();
        assert!(!hostaddr.is_empty());
    }

    #[test]
    fn test_multi_host_conn_str() {
        let conn_str = get_multi_host_conn_str("host1,host2", "5432,5433");
        assert!(conn_str.contains("host=host1,host2"));
        assert!(conn_str.contains("port=5432,5433"));
    }

    #[test]
    fn test_build_with_params() {
        let conn_str = build_conn_str_with_params(&[
            ("application_name", "test"),
            ("connect_timeout", "10"),
        ]);
        assert!(conn_str.contains("application_name=test"));
        assert!(conn_str.contains("connect_timeout=10"));
    }
}

