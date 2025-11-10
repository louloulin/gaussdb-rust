# 测试环境配置说明

## 概述

本项目支持通过环境变量灵活配置测试数据库连接，支持本地 Docker 和远程服务器两种环境。

## 快速开始

### 1. 创建环境配置文件

```bash
# 复制示例配置文件
cp env.example .env

# 编辑配置文件
vim .env
```

### 2. 配置环境变量

#### 方式一：使用单个环境变量（推荐）

编辑 `.env` 文件，配置各个连接参数：

```bash
PGUSER=gaussdb
PGPASSWORD=Gaussdb@123
PGHOST=localhost
PGPORT=5433
PGDATABASE=postgres
```

#### 方式二：使用完整连接字符串

```bash
TEST_CONN_STR=user=gaussdb password=Gaussdb@123 host=localhost port=5433 dbname=postgres
```

**优先级**：`TEST_CONN_STR` > 单个环境变量 > 默认值

### 3. 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块的测试
cargo test -p gaussdb
cargo test -p tokio-gaussdb
cargo test -p gaussdb-derive-test
```

## 环境配置

### 本地 Docker 环境

```bash
# .env 配置
PGUSER=gaussdb
PGPASSWORD=Gaussdb@123
PGHOST=localhost
PGPORT=5433
PGDATABASE=postgres
```

启动 Docker 容器：

```bash
docker-compose up -d
```

### 远程服务器环境

```bash
# .env 配置
PGUSER=gaussdb
PGPASSWORD=Gaussdb@123
PGHOST=113.44.80.136
PGPORT=8000
PGDATABASE=postgres
```

## 支持的环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `PGUSER` | 数据库用户名 | `gaussdb` |
| `PGPASSWORD` | 数据库密码 | `Gaussdb@123` |
| `PGHOST` | 数据库主机地址 | `localhost` |
| `PGPORT` | 数据库端口 | `5433` |
| `PGDATABASE` | 数据库名称 | `postgres` |
| `TEST_CONN_STR` | 完整连接字符串（可选） | - |
| `RUST_LOG` | 日志级别 | `info` |
| `RUST_BACKTRACE` | 错误堆栈跟踪 | `1` |

## 技术实现

### 架构设计

项目采用**全局测试辅助模块**的设计，避免在每个测试文件中重复定义相同的配置函数：

1. **gaussdb** 模块：`gaussdb/src/test_helpers.rs`
2. **tokio-gaussdb** 模块：`tokio-gaussdb/tests/test/test_helpers.rs`
3. **gaussdb-derive-test** 模块：`gaussdb-derive-test/src/test_helpers.rs`

### 全局测试辅助模块

每个 crate 都有一个独立的 `test_helpers` 模块，提供统一的环境变量配置接口：

```rust
// gaussdb/src/test_helpers.rs
use std::sync::Once;

static INIT: Once = Once::new();

/// 加载环境变量（仅执行一次）
pub fn load_env() {
    INIT.call_once(|| {
        let _ = dotenvy::dotenv();
        let _ = dotenvy::from_filename("../.env");
    });
}

/// 获取测试数据库连接字符串
pub fn get_test_conn_str() -> String {
    load_env();
    
    // 优先使用完整连接字符串
    if let Ok(conn_str) = std::env::var("TEST_CONN_STR") {
        return conn_str;
    }
    
    // 从单个环境变量构建连接字符串
    let user = std::env::var("PGUSER").unwrap_or_else(|_| "gaussdb".to_string());
    let password = std::env::var("PGPASSWORD").unwrap_or_else(|_| "Gaussdb@123".to_string());
    let host = std::env::var("PGHOST").unwrap_or_else(|_| "localhost".to_string());
    let port = std::env::var("PGPORT").unwrap_or_else(|_| "5433".to_string());
    let dbname = std::env::var("PGDATABASE").unwrap_or_else(|_| "postgres".to_string());
    
    format!(
        "user={} password={} host={} port={} dbname={}",
        user, password, host, port, dbname
    )
}
```

### 测试文件使用方式

在测试文件中，只需导入全局模块即可使用：

```rust
// gaussdb/src/test.rs
use crate::test_helpers::*;

#[test]
fn my_test() {
    let mut client = Client::connect(&get_test_conn_str(), NoTls).unwrap();
    // 测试代码...
}
```

### 提供的辅助函数

| 函数名 | 说明 | 返回值 |
|--------|------|--------|
| `load_env()` | 加载 .env 文件（仅执行一次） | - |
| `get_test_conn_str()` | 获取完整连接字符串 | `String` |
| `get_test_host()` | 获取数据库主机 | `String` |
| `get_test_port()` | 获取数据库端口 | `String` |
| `get_test_user()` | 获取数据库用户名 | `String` |
| `get_test_password()` | 获取数据库密码 | `String` |
| `get_test_database()` | 获取数据库名称 | `String` |
| `get_test_host_port()` | 获取主机和端口（tokio-gaussdb） | `(String, String)` |
| `get_multi_host_conn_str()` | 多主机连接字符串（tokio-gaussdb） | `String` |
| `get_hostaddr()` | 获取 IP 格式地址（tokio-gaussdb） | `String` |

## 注意事项

1. **安全性**：`.env` 文件包含敏感信息，已加入 `.gitignore`，不会提交到版本库
2. **环境隔离**：本地开发和远程测试使用不同的 `.env` 配置
3. **默认值**：即使不配置 `.env` 文件，测试也能使用默认值运行
4. **连接字符串格式**：必须使用 PostgreSQL 标准格式，参数之间用空格分隔

## 故障排除

### 连接被拒绝

```
Error: Connection refused (os error 61)
```

**解决方案**：
1. 检查数据库服务是否启动：`docker ps`
2. 检查 `PGHOST` 和 `PGPORT` 配置是否正确
3. 确认防火墙规则允许连接

### 认证失败

```
Error: password authentication failed
```

**解决方案**：
1. 检查 `PGUSER` 和 `PGPASSWORD` 配置
2. 确认用户在数据库中存在
3. 检查数据库的认证配置

### 环境变量未生效

**解决方案**：
1. 确认 `.env` 文件在项目根目录
2. 检查 `.env` 文件格式（不要有多余的引号或空格）
3. 重启 IDE 或终端会话

## 相关文件

### 配置文件
- `env.example` - 环境变量配置示例
- `.gitignore` - 已配置忽略 `.env` 文件

### 依赖配置
- `gaussdb/Cargo.toml` - 添加了 `dotenvy` 依赖
- `tokio-gaussdb/Cargo.toml` - 添加了 `dotenvy` 依赖
- `gaussdb-derive-test/Cargo.toml` - 添加了 `dotenvy` 依赖

### 全局测试辅助模块
- `gaussdb/src/test_helpers.rs` - gaussdb 测试辅助模块
- `tokio-gaussdb/tests/test/test_helpers.rs` - tokio-gaussdb 测试辅助模块
- `gaussdb-derive-test/src/test_helpers.rs` - gaussdb-derive-test 测试辅助模块

### 模块引用
- `gaussdb/src/lib.rs` - 引入 `test_helpers` 模块
- `tokio-gaussdb/tests/test/main.rs` - 引入 `test_helpers` 模块
- `gaussdb-derive-test/src/lib.rs` - 引入 `test_helpers` 模块

## 测试覆盖范围

以下测试文件已支持环境变量配置：

### gaussdb 模块
- `gaussdb/src/test.rs` - 所有同步客户端测试（25个连接点）

### tokio-gaussdb 模块
- `tokio-gaussdb/tests/test/main.rs` - 所有异步客户端测试（4个连接点）
- `tokio-gaussdb/tests/test/runtime.rs` - 运行时相关测试（15个连接点）

### gaussdb-derive-test 模块
- `gaussdb-derive-test/src/domains.rs` - Domain 类型测试（5个连接点）
- `gaussdb-derive-test/src/enums.rs` - 枚举类型测试（10个连接点）
- `gaussdb-derive-test/src/composites.rs` - 复合类型测试（9个连接点）
- `gaussdb-derive-test/src/transparent.rs` - 透明类型测试（1个连接点）

**总计**：已替换 **69个** 硬编码连接字符串为环境变量配置

