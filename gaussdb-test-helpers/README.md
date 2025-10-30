# gaussdb-test-helpers

全局统一的 GaussDB Rust 客户端测试辅助模块。

## 设计目标

### 1. 单一职责
- 仅负责测试环境配置管理
- 不包含业务逻辑

### 2. 全局复用
- 所有 GaussDB Rust crate 共享同一套配置逻辑
- 避免在每个 crate 中重复定义相同的函数
- 统一的接口和行为

### 3. 灵活配置
- 支持环境变量配置
- 支持 .env 文件
- 提供合理的默认值

## 使用方法

### 1. 添加依赖

在需要使用的 crate 的 `Cargo.toml` 中添加：

```toml
[dev-dependencies]
gaussdb-test-helpers = { path = "../gaussdb-test-helpers" }
```

### 2. 导入并使用

```rust
use gaussdb_test_helpers::*;

#[test]
fn my_test() {
    let conn_str = get_test_conn_str();
    let client = Client::connect(&conn_str, NoTls).unwrap();
    // 测试代码...
}
```

## API 文档

### 核心函数

#### `get_test_conn_str() -> String`

获取完整的测试数据库连接字符串。

**优先级**：`TEST_CONN_STR` > 单个环境变量 > 默认值

```rust
let conn_str = get_test_conn_str();
// "user=gaussdb password=Gaussdb@123 host=localhost port=5433 dbname=postgres"
```

#### `get_test_host() -> String`

获取数据库主机地址（默认：`localhost`）

#### `get_test_port() -> String`

获取数据库端口（默认：`5433`）

#### `get_test_user() -> String`

获取数据库用户名（默认：`gaussdb`）

#### `get_test_password() -> String`

获取数据库密码（默认：`Gaussdb@123`）

#### `get_test_database() -> String`

获取数据库名称（默认：`postgres`）

### 高级函数

#### `get_test_host_port() -> (String, String)`

获取主机和端口元组，适用于 TCP 连接。

```rust
let (host, port) = get_test_host_port();
let addr = format!("{}:{}", host, port);
let socket = TcpStream::connect(&addr).await?;
```

#### `get_multi_host_conn_str(hosts: &str, ports: &str) -> String`

构建多主机连接字符串。

```rust
let conn_str = get_multi_host_conn_str("host1,host2", "5432,5433");
// "host=host1,host2 port=5432,5433 user=gaussdb password=Gaussdb@123 dbname=postgres"
```

#### `get_hostaddr() -> String`

获取 IP 格式的主机地址（自动将 `localhost` 转换为 `127.0.0.1`）。

```rust
let hostaddr = get_hostaddr();
// 如果 PGHOST=localhost，返回 "127.0.0.1"
```

#### `build_conn_str_with_params(extra_params: &[(&str, &str)]) -> String`

构建带有额外参数的连接字符串。

```rust
let conn_str = build_conn_str_with_params(&[
    ("application_name", "my_test"),
    ("connect_timeout", "10"),
]);
```

## 环境变量

| 变量名 | 说明 | 默认值 |
|--------|------|--------|
| `TEST_CONN_STR` | 完整连接字符串（最高优先级） | - |
| `PGUSER` | 数据库用户名 | `gaussdb` |
| `PGPASSWORD` | 数据库密码 | `Gaussdb@123` |
| `PGHOST` | 数据库主机 | `localhost` |
| `PGPORT` | 数据库端口 | `5433` |
| `PGDATABASE` | 数据库名称 | `postgres` |

## 配置文件

在项目根目录创建 `.env` 文件：

```bash
# 方式一：使用单个环境变量
PGUSER=gaussdb
PGPASSWORD=Gaussdb@123
PGHOST=localhost
PGPORT=5433
PGDATABASE=postgres

# 方式二：使用完整连接字符串（优先级更高）
TEST_CONN_STR=user=gaussdb password=Gaussdb@123 host=localhost port=5433 dbname=postgres
```

## 使用示例

### 基本用法

```rust
use gaussdb_test_helpers::*;
use gaussdb::{Client, NoTls};

#[test]
fn test_basic_query() {
    let mut client = Client::connect(&get_test_conn_str(), NoTls).unwrap();
    let rows = client.query("SELECT 1", &[]).unwrap();
    assert_eq!(rows.len(), 1);
}
```

### 异步测试

```rust
use gaussdb_test_helpers::*;
use tokio_gaussdb::NoTls;

#[tokio::test]
async fn test_async_query() {
    let (client, connection) = tokio_gaussdb::connect(&get_test_conn_str(), NoTls)
        .await
        .unwrap();
    
    tokio::spawn(connection);
    
    let rows = client.query("SELECT 1", &[]).await.unwrap();
    assert_eq!(rows.len(), 1);
}
```

### 自定义连接参数

```rust
use gaussdb_test_helpers::*;

#[test]
fn test_with_custom_params() {
    let conn_str = build_conn_str_with_params(&[
        ("application_name", "integration_test"),
        ("connect_timeout", "30"),
    ]);
    
    let client = Client::connect(&conn_str, NoTls).unwrap();
    // 测试代码...
}
```

## 架构优势

### 1. 代码复用
- 所有测试共享同一套配置逻辑
- 减少重复代码
- 统一的行为和接口

### 2. 易于维护
- 配置逻辑集中在一个地方
- 修改配置只需更新一个 crate
- 清晰的职责划分

### 3. 灵活性
- 支持多种配置方式
- 易于扩展新功能
- 向后兼容

## 测试

运行测试辅助模块自身的测试：

```bash
cargo test -p gaussdb-test-helpers
```

## 许可证

MIT OR Apache-2.0

