# GaussDB Rust Features 配置说明

本项目支持通过 Cargo features 在 OpenGauss 和 GaussDB 之间进行配置。

## 可用的 Features

### 默认 Feature：`opengauss`

- **opengauss**（默认启用）: 启用 OpenGauss 特性支持
  - 包含 cancel_query 功能
  - 包含 domain 类型支持
  - 包含 GaussDB 特有的认证方法（SHA256、MD5_SHA256）
  - 包含 GaussDB 兼容的 SASL 认证

- **gauss**: 启用 GaussDB 企业版特性支持
  - 包含所有基础 GaussDB 特性
  - 与 OpenGauss 使用相同的协议扩展

## 使用方法

### 1. 使用默认配置（OpenGauss）

```toml
[dependencies]
gaussdb = "0.1"
tokio-gaussdb = "0.1"
```

这将自动启用 `opengauss` feature。

### 2. 使用 GaussDB 企业版

```toml
[dependencies]
gaussdb = { version = "0.1", default-features = false, features = ["gauss"] }
tokio-gaussdb = { version = "0.1", default-features = false, features = ["gauss", "runtime"] }
```

### 3. 禁用所有扩展特性（仅 PostgreSQL 兼容）

```toml
[dependencies]
gaussdb = { version = "0.1", default-features = false }
tokio-gaussdb = { version = "0.1", default-features = false, features = ["runtime"] }
```

注意：禁用所有扩展特性后，以下功能将不可用：
- cancel_query API
- domain 类型支持
- 部分 GaussDB 特有的测试

## Feature 差异说明

| 功能 | PostgreSQL | OpenGauss | GaussDB |
|------|-----------|-----------|---------|
| 基础连接 | ✅ | ✅ | ✅ |
| 标准 SCRAM-SHA-256 认证 | ✅ | ✅ | ✅ |
| GaussDB SCRAM 兼容模式 | ❌ | ✅ | ✅ |
| SHA256 认证 | ❌ | ✅ | ✅ |
| MD5_SHA256 认证 | ❌ | ✅ | ✅ |
| cancel_query API | ❌ | ✅ | ✅ |
| Domain 类型 | ❌ | ✅ | ✅ |

## 代码示例

### OpenGauss 连接示例（默认）

```rust
use gaussdb::{Client, NoTls};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = Client::connect(
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres",
        NoTls,
    )?;
    
    // 使用 cancel_query（仅在 opengauss feature 下可用）
    #[cfg(feature = "opengauss")]
    {
        let cancel_token = client.cancel_token();
        // ...
    }
    
    Ok(())
}
```

### 异步连接示例

```rust
use tokio_gaussdb::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let (client, connection) = tokio_gaussdb::connect(
        "host=localhost user=gaussdb password=Gaussdb@123",
        NoTls,
    ).await?;

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let rows = client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await?;

    let value: &str = rows[0].get(0);
    println!("Result: {}", value);

    Ok(())
}
```

## 测试配置

运行测试时指定 feature：

```bash
# 使用默认 opengauss feature
cargo test

# 使用 gauss feature
cargo test --no-default-features --features gauss

# 不使用任何扩展 feature（仅 PostgreSQL 兼容）
cargo test --no-default-features --features runtime
```

## 编译选项

### 完整编译（所有特性）

```bash
cargo build --all-features
```

### 最小编译（仅核心功能）

```bash
cargo build --no-default-features --features runtime
```

### OpenGauss 编译（默认）

```bash
cargo build
# 或显式指定
cargo build --features opengauss
```

### GaussDB 编译

```bash
cargo build --no-default-features --features gauss,runtime
```

## 条件编译示例

在您的代码中可以使用条件编译：

```rust
#[cfg(feature = "opengauss")]
use tokio_gaussdb::cancel_query;

#[cfg(feature = "opengauss")]
fn use_opengauss_feature() {
    // OpenGauss 特有功能
}

#[cfg(feature = "gauss")]
fn use_gauss_feature() {
    // GaussDB 特有功能
}

// 同时支持两者
#[cfg(any(feature = "opengauss", feature = "gauss"))]
fn use_gaussdb_features() {
    // OpenGauss 或 GaussDB 共同特性
}
```

## 迁移指南

### 从无 feature 版本迁移

如果您之前使用的是没有 feature 区分的版本：

1. **无需改动**：默认启用 `opengauss`，行为保持不变
2. **如果只需要 PostgreSQL 兼容**：添加 `default-features = false`
3. **如果需要 GaussDB 特性**：使用 `features = ["gauss"]`

### 示例：Cargo.toml 更新

```toml
# 之前
[dependencies]
gaussdb = "0.1.0"

# 现在（保持默认行为）
[dependencies]
gaussdb = "0.1.1"  # 自动启用 opengauss

# 或者明确指定
[dependencies]
gaussdb = { version = "0.1.1", features = ["opengauss"] }

# 或者使用 GaussDB
[dependencies]
gaussdb = { version = "0.1.1", default-features = false, features = ["gauss"] }
```

## 注意事项

1. **默认行为**：如果不指定任何 feature，将自动使用 `opengauss` feature
2. **互斥性**：`opengauss` 和 `gauss` feature 可以同时启用，但通常只需要一个
3. **测试覆盖**：部分测试需要特定 feature 才能运行
4. **向后兼容**：默认配置确保与之前版本的行为一致

## 常见问题

### Q: 我应该使用哪个 feature？
A: 
- 连接 OpenGauss：使用默认配置或 `opengauss` feature
- 连接 GaussDB 企业版：使用 `gauss` feature  
- 连接标准 PostgreSQL：禁用默认 features

### Q: 两个 feature 有什么区别？
A: 目前 `opengauss` 和 `gauss` 提供相同的功能集，主要用于未来可能的差异化支持。

### Q: 不启用任何 feature 可以吗？
A: 可以，但会失去 GaussDB/OpenGauss 特有的功能，如 cancel_query 和 domain 支持。

### Q: 如何确认使用了哪个 feature？
A: 运行 `cargo tree -f "{p} {f}"` 查看启用的 features。

## 更多信息

- [项目主页](https://github.com/HuaweiCloudDeveloper/gaussdb-rust)
- [API 文档](https://docs.rs/gaussdb)
- [OpenGauss 官方文档](https://opengauss.org)
- [GaussDB 官方文档](https://support.huaweicloud.com/gaussdb/)

