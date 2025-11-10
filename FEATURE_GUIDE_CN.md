# GaussDB Rust 特性配置指南

## 📋 概述

GaussDB Rust 驱动支持通过 Cargo features 来控制不同数据库系统的特性支持：

- **opengauss** (默认): OpenGauss 数据库支持
- **gauss**: GaussDB 企业版支持

## 🚀 快速开始

### 默认使用（OpenGauss）

```toml
[dependencies]
gaussdb = "0.1"
tokio-gaussdb = "0.1"
```

### 使用 GaussDB 企业版

```toml
[dependencies]
gaussdb = { version = "0.1", default-features = false, features = ["gauss"] }
tokio-gaussdb = { version = "0.1", default-features = false, features = ["gauss", "runtime"] }
```

## ⚙️ Feature 说明

### `opengauss` (默认)

OpenGauss 数据库特性支持，包括：

✅ 所有标准 PostgreSQL 功能  
✅ GaussDB 兼容的 SASL 认证  
✅ SHA256 和 MD5_SHA256 认证方法  
✅ `cancel_query` API  
✅ Domain 类型支持  

### `gauss`

GaussDB 企业版特性支持，功能集与 `opengauss` 相同：

✅ 所有标准 PostgreSQL 功能  
✅ GaussDB 兼容的 SASL 认证  
✅ SHA256 和 MD5_SHA256 认证方法  
✅ `cancel_query` API  
✅ Domain 类型支持  

## 📝 使用示例

### 同步客户端（gaussdb）

```rust
use gaussdb::{Client, NoTls, Error};

fn main() -> Result<(), Error> {
    // 连接到数据库
    let mut client = Client::connect(
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres",
        NoTls,
    )?;
    
    // 执行查询
    let rows = client.query("SELECT $1::TEXT", &[&"hello world"])?;
    let value: &str = rows[0].get(0);
    println!("Result: {}", value);
    
    // 使用 OpenGauss 特有功能（仅在启用 opengauss feature 时）
    #[cfg(feature = "opengauss")]
    {
        let cancel_token = client.cancel_token();
        // 可以在其他线程中取消查询
    }
    
    Ok(())
}
```

### 异步客户端（tokio-gaussdb）

```rust
use tokio_gaussdb::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 连接到数据库
    let (client, connection) = tokio_gaussdb::connect(
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres",
        NoTls,
    ).await?;

    // 在后台处理连接
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    // 执行异步查询
    let rows = client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await?;

    let value: &str = rows[0].get(0);
    println!("Result: {}", value);

    Ok(())
}
```

## 🧪 测试

### 运行所有测试（使用默认 opengauss feature）

```bash
cargo test
```

### 使用 gauss feature 运行测试

```bash
cargo test --no-default-features --features gauss
```

### 运行特定 feature 的测试

```bash
# 仅测试 opengauss 特性
cargo test --features opengauss

# 测试时禁用所有扩展特性
cargo test --no-default-features --features runtime
```

## 🔧 编译选项

### 默认编译（OpenGauss）

```bash
cargo build
```

### GaussDB 企业版编译

```bash
cargo build --no-default-features --features gauss,runtime
```

### 最小化编译（仅 PostgreSQL 兼容）

```bash
cargo build --no-default-features --features runtime
```

### 完整编译（所有特性）

```bash
cargo build --all-features
```

## 📊 Feature 对比表

| 功能特性 | 无 feature | opengauss | gauss |
|---------|-----------|-----------|-------|
| PostgreSQL 协议 | ✅ | ✅ | ✅ |
| 标准认证方法 | ✅ | ✅ | ✅ |
| SCRAM-SHA-256 | ✅ | ✅ | ✅ |
| GaussDB SCRAM 兼容 | ❌ | ✅ | ✅ |
| SHA256 认证 | ✅ | ✅ | ✅ |
| MD5_SHA256 认证 | ✅ | ✅ | ✅ |
| cancel_query API | ❌ | ✅ | ✅ |
| Domain 类型 | ❌ | ✅ | ✅ |

## 🔄 迁移指南

### 从旧版本迁移

如果您使用的是 0.1.0 或更早版本：

**无需任何改动** - 默认启用 `opengauss` feature，保持原有行为。

### 配置文件更新示例

```toml
# === 旧版本（0.1.0）===
[dependencies]
gaussdb = "0.1.0"
tokio-gaussdb = "0.1.0"

# === 新版本（0.1.1+）保持相同行为 ===
[dependencies]
gaussdb = "0.1.1"
tokio-gaussdb = "0.1.1"

# === 新版本使用 GaussDB 特性 ===
[dependencies]
gaussdb = { version = "0.1.1", default-features = false, features = ["gauss"] }
tokio-gaussdb = { version = "0.1.1", default-features = false, features = ["gauss", "runtime"] }

# === 新版本仅 PostgreSQL 兼容 ===
[dependencies]
gaussdb = { version = "0.1.1", default-features = false }
tokio-gaussdb = { version = "0.1.1", default-features = false, features = ["runtime"] }
```

## 💡 条件编译

在代码中使用条件编译来支持不同特性：

```rust
// 仅在 opengauss feature 启用时编译
#[cfg(feature = "opengauss")]
fn opengauss_specific_function() {
    // OpenGauss 特有功能
}

// 仅在 gauss feature 启用时编译
#[cfg(feature = "gauss")]
fn gauss_specific_function() {
    // GaussDB 特有功能
}

// 在任一 GaussDB 相关 feature 启用时编译
#[cfg(any(feature = "opengauss", feature = "gauss"))]
fn gaussdb_common_function() {
    // OpenGauss 和 GaussDB 共同功能
}

// 运行时检查
fn main() {
    #[cfg(feature = "opengauss")]
    println!("OpenGauss feature 已启用");
    
    #[cfg(feature = "gauss")]
    println!("GaussDB feature 已启用");
    
    #[cfg(not(any(feature = "opengauss", feature = "gauss")))]
    println!("仅 PostgreSQL 兼容模式");
}
```

## ❓ 常见问题

### Q1: 应该选择哪个 feature？

**A:** 
- 连接 **OpenGauss** → 使用默认配置（自动启用 `opengauss`）
- 连接 **GaussDB 企业版** → 显式指定 `gauss` feature
- 连接 **PostgreSQL** → 禁用默认 features

### Q2: `opengauss` 和 `gauss` 有什么区别？

**A:** 当前版本中，两者提供相同的功能集。分离这两个 feature 是为了：
- 未来可能的差异化特性支持
- 明确的语义区分
- 更好的文档组织

### Q3: 可以同时启用两个 feature 吗？

**A:** 技术上可以，但不推荐。通常只需要启用一个即可。

### Q4: 不启用任何 feature 会怎样？

**A:** 仍然可以连接数据库，但会失去以下功能：
- `cancel_query` API
- Domain 类型支持
- 某些 OpenGauss/GaussDB 特有的测试

### Q5: 如何查看当前启用的 features？

**A:** 使用以下命令：
```bash
cargo tree -f "{p} {f}"
```

### Q6: 现有代码需要修改吗？

**A:** 不需要。默认行为保持不变，现有代码可以无缝升级。

## 📚 相关资源

- [完整 Feature 文档](FEATURES.md)
- [项目主页](https://github.com/HuaweiCloudDeveloper/gaussdb-rust)
- [API 文档](https://docs.rs/gaussdb)
- [OpenGauss 官方文档](https://opengauss.org)
- [GaussDB 官方文档](https://support.huaweicloud.com/gaussdb/)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

如果您在使用 features 时遇到问题，请在 GitHub Issues 中反馈。

## 📄 许可证

MIT OR Apache-2.0

