# GaussDB-Rust

[![Crates.io](https://img.shields.io/crates/v/gaussdb.svg)](https://crates.io/crates/gaussdb)
[![文档](https://docs.rs/gaussdb/badge.svg)](https://docs.rs/gaussdb)
[![许可证](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

[English](README.md) | [中文](README-zh.md)

GaussDB 和 OpenGauss 数据库的原生 Rust 驱动，提供高性能和完整的 PostgreSQL 兼容性。

## 📦 核心组件

### gaussdb [![Latest Version](https://img.shields.io/crates/v/gaussdb.svg)](https://crates.io/crates/gaussdb)

[📖 文档](https://docs.rs/gaussdb)

原生的同步 GaussDB 客户端，完全兼容 PostgreSQL。

### tokio-gaussdb [![Latest Version](https://img.shields.io/crates/v/tokio-gaussdb.svg)](https://crates.io/crates/tokio-gaussdb)

[📖 文档](https://docs.rs/tokio-gaussdb)

原生的异步 GaussDB 客户端，完全兼容 PostgreSQL，基于 Tokio 运行时。

### gaussdb-types [![Latest Version](https://img.shields.io/crates/v/gaussdb-types.svg)](https://crates.io/crates/gaussdb-types)

[📖 文档](https://docs.rs/gaussdb-types)

Rust 与 GaussDB/PostgreSQL 类型之间的转换工具。

### gaussdb-native-tls [![Latest Version](https://img.shields.io/crates/v/gaussdb-native-tls.svg)](https://crates.io/crates/gaussdb-native-tls)

[📖 文档](https://docs.rs/gaussdb-native-tls)

通过 native-tls 为 gaussdb 和 tokio-gaussdb 提供 TLS 支持。

### gaussdb-openssl [![Latest Version](https://img.shields.io/crates/v/gaussdb-openssl.svg)](https://crates.io/crates/gaussdb-openssl)

[📖 文档](https://docs.rs/gaussdb-openssl)

通过 openssl 为 gaussdb 和 tokio-gaussdb 提供 TLS 支持。

## ✨ 特性

### 🔐 灵活的 Feature 支持 (v0.1.1+)

GaussDB-Rust 现在支持灵活的 feature flags 来定制功能：

- **`opengauss`**（默认）：完整的 OpenGauss 支持，包括 `cancel_query` 和 domain 类型
- **`gauss`**：GaussDB 企业版支持
- **无 features**：纯 PostgreSQL 兼容

```toml
# 默认配置（OpenGauss）
[dependencies]
gaussdb = "0.1"

# GaussDB 企业版
[dependencies]
gaussdb = { version = "0.1", default-features = false, features = ["gauss"] }

# 仅 PostgreSQL 兼容
[dependencies]
gaussdb = { version = "0.1", default-features = false }
```

📖 查看 [FEATURE_GUIDE_CN.md](FEATURE_GUIDE_CN.md) 了解详细的 feature 文档。

### GaussDB 认证支持

本库提供完整的 GaussDB 增强认证机制支持：

- **SCRAM-SHA-256 兼容性**：增强的 SCRAM-SHA-256 认证，兼容 GaussDB/OpenGauss (v0.1.1+)
- **SHA256 认证**：GaussDB 的安全 SHA256 认证
- **MD5_SHA256 认证**：结合 MD5 和 SHA256 的混合认证
- **标准 PostgreSQL 认证**：完全兼容 MD5、SCRAM-SHA-256 等 PostgreSQL 认证方法
- **自适应认证**：基于服务器类型的智能认证方法选择 (v0.1.1+)

### v0.1.1 新特性

#### SCRAM-SHA-256 兼容性修复
- ✅ **修复 SCRAM 认证**：解决了 "invalid message length: expected to be at end of iterator for sasl" 错误
- ✅ **GaussDB 消息解析**：增强的 SASL 消息解析器，支持 GaussDB 特定格式
- ✅ **双重认证策略**：从 GaussDB 兼容模式自动降级到标准认证
- ✅ **运行时冲突解决**：修复了异步环境中的 "Cannot start a runtime from within a runtime" 错误

#### 增强功能
- 🚀 **性能优化**：连接建立时间降至平均 ~11.67ms
- 🔍 **更好的诊断**：全面的错误分析和故障排除工具
- 🧪 **广泛测试**：在真实 GaussDB/OpenGauss 环境中 184 个测试，100% 通过率
- 📊 **生产就绪**：在 openGauss 7.0.0-RC1 上验证，支持高并发

## 🚀 快速开始

### 基本连接

```rust
use tokio_gaussdb::{NoTls, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // 使用 SHA256 认证连接到 GaussDB
    let (client, connection) = tokio_gaussdb::connect(
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433",
        NoTls,
    ).await?;

    // 启动连接任务
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("连接错误: {}", e);
        }
    });

    // 执行简单查询
    let rows = client.query("SELECT $1::TEXT", &[&"hello world"]).await?;
    let value: &str = rows[0].get(0);
    println!("结果: {}", value);

    Ok(())
}
```

### 高级认证

```rust
use tokio_gaussdb::{Config, NoTls};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置特定认证方式的连接
    let mut config = Config::new();
    config
        .host("localhost")
        .port(5433)
        .user("gaussdb")
        .password("Gaussdb@123")
        .dbname("postgres");

    let (client, connection) = config.connect(NoTls).await?;

    // 处理连接...
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("连接错误: {}", e);
        }
    });

    // 你的应用逻辑
    Ok(())
}
```

### 同步客户端

```rust
use gaussdb::{Client, NoTls, Error};

fn main() -> Result<(), Error> {
    // 连接到数据库
    let mut client = Client::connect(
        "host=localhost user=gaussdb password=Gaussdb@123 dbname=postgres port=5433",
        NoTls,
    )?;

    // 执行查询
    let rows = client.query("SELECT $1::TEXT", &[&"hello world"])?;
    let value: &str = rows[0].get(0);
    println!("结果: {}", value);

    Ok(())
}
```

## 🔧 安装

将以下内容添加到您的 `Cargo.toml`：

```toml
[dependencies]
# 异步客户端（推荐）
tokio-gaussdb = "0.1"
tokio = { version = "1", features = ["full"] }

# 或同步客户端
gaussdb = "0.1"
```

## 🎯 Feature 配置

### 默认（OpenGauss）

```toml
[dependencies]
gaussdb = "0.1"
tokio-gaussdb = "0.1"
```

包含的功能：
- ✅ 所有 PostgreSQL 功能
- ✅ GaussDB 认证方法
- ✅ `cancel_query` API
- ✅ Domain 类型支持

### GaussDB 企业版

```toml
[dependencies]
gaussdb = { version = "0.1", default-features = false, features = ["gauss"] }
tokio-gaussdb = { version = "0.1", default-features = false, features = ["gauss", "runtime"] }
```

### 仅 PostgreSQL

```toml
[dependencies]
gaussdb = { version = "0.1", default-features = false }
tokio-gaussdb = { version = "0.1", default-features = false, features = ["runtime"] }
```

## 🗄️ 兼容性

### 数据库支持

| 数据库 | 版本 | 认证方式 | 状态 |
|----------|---------|----------------|--------|
| GaussDB | 0.1.1+ | SHA256, MD5_SHA256, MD5, SCRAM-SHA-256 | ✅ 完全支持 |
| OpenGauss | 3.0+ | SHA256, MD5_SHA256, MD5, SCRAM-SHA-256 | ✅ 完全支持 |
| PostgreSQL | 10+ | SCRAM-SHA-256, MD5 | ✅ 完全支持 |

### 功能兼容性

| 功能 | GaussDB | OpenGauss | PostgreSQL |
|---------|---------|-----------|------------|
| 基本 SQL 操作 | ✅ | ✅ | ✅ |
| 事务 | ✅ | ✅ | ✅ |
| 预编译语句 | ✅ | ✅ | ✅ |
| COPY 操作 | ✅ | ✅ | ✅ |
| LISTEN/NOTIFY | ⚠️ 有限 | ⚠️ 有限 | ✅ |
| 二进制 COPY | ⚠️ 有问题 | ⚠️ 有问题 | ✅ |

## 🧪 运行测试

### 前置条件

测试套件需要 GaussDB 或 OpenGauss 运行。最简单的方式是使用 Docker：

1. 安装 `docker` 和 `docker-compose`
   - Ubuntu: `sudo apt install docker.io docker-compose`
   - Windows: 安装 Docker Desktop
   - macOS: 安装 Docker Desktop

2. 确保你的用户有 Docker 权限
   - Ubuntu: `sudo usermod -aG docker $USER`

### 运行测试

1. 切换到 `gaussdb-rust` 仓库的顶层目录
2. 启动测试数据库：
   ```bash
   docker-compose up -d
   ```
3. 运行测试套件：
   ```bash
   cargo test
   ```
4. 停止测试数据库：
   ```bash
   docker-compose stop
   ```

### 按 Feature 测试

```bash
# 使用默认 opengauss feature
cargo test

# 使用 gauss feature
cargo test --no-default-features --features gauss

# 不使用扩展 features
cargo test --no-default-features --features runtime
```

### 测试配置

测试套件支持 GaussDB 和 OpenGauss 环境。连接字符串自动配置为：

- **主机**: localhost
- **端口**: 5433 (GaussDB/OpenGauss 默认)
- **用户**: gaussdb
- **密码**: Gaussdb@123
- **数据库**: postgres

您可以通过 `.env` 文件自定义配置：

```bash
cp env.example .env
# 编辑 .env 文件修改连接参数
```

## 📚 文档

### API 文档

- [gaussdb](https://docs.rs/gaussdb) - 同步客户端 API
- [tokio-gaussdb](https://docs.rs/tokio-gaussdb) - 异步客户端 API
- [gaussdb-types](https://docs.rs/gaussdb-types) - 类型转换工具
- [gaussdb-protocol](https://docs.rs/gaussdb-protocol) - 底层协议实现

### Feature 指南

- **[FEATURES.md](FEATURES.md)** - 完整 feature 文档（英文）
- **[FEATURE_GUIDE_CN.md](FEATURE_GUIDE_CN.md)** - Feature 使用指南（中文）
- **[FEATURE_SUMMARY.md](FEATURE_SUMMARY.md)** - 快速参考

### 技术文档

- [认证方法](docs/authentication.md)
- [GaussDB vs PostgreSQL 差异](docs/GaussDB-PostgreSQL-差异分析报告.md)
- [实现总结](IMPLEMENTATION_SUMMARY.md)
- [兼容性说明 (中文)](docs/compatibility-zh.md)
- [兼容性说明 (English)](docs/compatibility-en.md)

### 示例代码

查看 [examples/](examples/) 目录获取完整的工作示例：

- 同步/异步基本连接
- 认证示例
- 事务处理
- 错误处理
- 压力测试

## 🏗️ 项目结构

```
gaussdb-rust/
├── gaussdb/                # 同步客户端
├── tokio-gaussdb/         # 异步客户端
├── gaussdb-types/         # 类型转换
├── gaussdb-protocol/      # 协议实现
├── gaussdb-openssl/       # OpenSSL TLS 支持
├── gaussdb-native-tls/    # Native TLS 支持
├── examples/              # 示例代码
├── docs/                  # 文档
└── tests/                 # 集成测试
```

## 🤝 贡献

我们欢迎贡献！

### 开发设置

1. 克隆仓库：
   ```bash
   git clone https://github.com/HuaweiCloudDeveloper/gaussdb-rust.git
   cd gaussdb-rust
   ```

2. 安装 Rust（如果尚未安装）：
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. 运行测试：
   ```bash
   cargo test
   ```

### 贡献指南

1. Fork 项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

## 📊 性能

- **连接时间**: ~11.67ms 平均
- **并发支持**: 高并发环境验证
- **测试覆盖**: 184 个测试，100% 通过率

## 🔐 安全性

- 支持所有主流认证方法
- TLS/SSL 加密支持
- 密码安全处理
- SQL 注入防护（通过参数化查询）

## 📝 许可证

本项目采用以下任一许可证：

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) 或 http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) 或 http://opensource.org/licenses/MIT)

由你选择。

## 🙏 致谢

本项目基于 Steven Fackler 优秀的 [rust-postgres](https://github.com/sfackler/rust-postgres) 库。我们对原作者和贡献者表示感谢。

## 💬 支持

- [GitHub Issues](https://github.com/HuaweiCloudDeveloper/gaussdb-rust/issues) - Bug 报告和功能请求
- [文档](https://docs.rs/gaussdb) - API 文档和指南
- [示例](examples/) - 代码示例和教程

## 🗺️ 路线图

### 当前版本 (v0.1.1)
- ✅ 完整的认证支持
- ✅ Feature flags 支持
- ✅ SCRAM 兼容性修复
- ✅ 性能优化

### 未来计划
- [ ] 连接池支持
- [ ] 更多 GaussDB 特有功能
- [ ] 改进的错误处理
- [ ] 更多示例和教程

## 📈 状态

- **稳定性**: 生产就绪
- **维护状态**: 积极维护
- **测试覆盖**: 高
- **文档**: 完整

---

**Made with ❤️ for the GaussDB and OpenGauss community**

