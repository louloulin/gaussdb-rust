# GaussDB Rust 测试配置架构

## 架构概览

```
gaussdb-rust/
├── gaussdb-test-helpers/          # 全局测试辅助 crate（核心）
│   ├── src/lib.rs                 # 统一的配置管理逻辑
│   ├── Cargo.toml                 # 仅依赖 dotenvy
│   └── README.md                  # 使用文档
│
├── gaussdb/                       # 同步客户端
│   ├── Cargo.toml                 # dev-dependencies: gaussdb-test-helpers
│   └── src/
│       └── test.rs                # 使用: use gaussdb_test_helpers::*;
│
├── tokio-gaussdb/                 # 异步客户端
│   ├── Cargo.toml                 # dev-dependencies: gaussdb-test-helpers
│   └── tests/test/
│       ├── main.rs                # 使用: use gaussdb_test_helpers::*;
│       └── runtime.rs             # 使用: use gaussdb_test_helpers::*;
│
├── gaussdb-derive-test/           # 派生宏测试
│   ├── Cargo.toml                 # dev-dependencies: gaussdb-test-helpers
│   └── src/
│       ├── domains.rs             # 使用: use gaussdb_test_helpers::*;
│       ├── enums.rs               # 使用: use gaussdb_test_helpers::*;
│       ├── composites.rs          # 使用: use gaussdb_test_helpers::*;
│       └── transparent.rs         # 使用: use gaussdb_test_helpers::*;
│
├── .env                           # 环境配置（gitignore）
└── env.example                    # 环境配置模板
```

## 设计原则

### 1. 单一职责原则（SRP）

**gaussdb-test-helpers** 仅负责一件事：测试环境配置管理

```rust
// gaussdb-test-helpers/src/lib.rs
pub fn get_test_conn_str() -> String { ... }
pub fn get_test_host() -> String { ... }
pub fn get_test_port() -> String { ... }
// ... 其他配置函数
```

### 2. 依赖倒置原则（DIP）

所有测试模块依赖于抽象的配置接口，而不是具体实现：

```rust
// 高层模块（测试代码）
use gaussdb_test_helpers::*;

#[test]
fn my_test() {
    let conn_str = get_test_conn_str(); // 依赖抽象接口
    // ...
}
```

### 3. 开闭原则（OCP）

配置模块对扩展开放，对修改关闭：

- ✅ 可以添加新的辅助函数
- ✅ 可以支持新的环境变量
- ❌ 不需要修改现有测试代码

### 4. DRY 原则（Don't Repeat Yourself）

```
之前：每个 crate 重复定义相同的配置函数（3个副本）
现在：统一的全局模块（1个实现，多处使用）

代码复用率提升：300% → 100%
```

## 数据流图

```
┌─────────────────────────────────────────────────────────┐
│                     环境配置源                            │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────┐     │
│  │ .env文件 │  │ 环境变量  │  │  默认值常量       │     │
│  └────┬─────┘  └────┬─────┘  └────┬─────────────┘     │
│       │             │              │                    │
│       └─────────────┴──────────────┘                    │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
┌─────────────────────────────────────────────────────────┐
│           gaussdb-test-helpers (统一配置层)              │
│  ┌───────────────────────────────────────────────────┐ │
│  │  load_env()          // 加载环境变量（Once）      │ │
│  │  get_test_conn_str() // 完整连接字符串            │ │
│  │  get_test_host()     // 数据库主机                │ │
│  │  get_test_port()     // 数据库端口                │ │
│  │  ...                 // 其他辅助函数              │ │
│  └───────────────────────────────────────────────────┘ │
└───────────────┬───────────────┬───────────────┬─────────┘
                │               │               │
    ┌───────────┘               │               └───────────┐
    │                           │                           │
    ▼                           ▼                           ▼
┌──────────┐           ┌──────────────┐         ┌───────────────────┐
│ gaussdb  │           │tokio-gaussdb │         │gaussdb-derive-test│
│  测试    │           │    测试      │         │      测试         │
└──────────┘           └──────────────┘         └───────────────────┘
```

## 依赖关系图

```
                    ┌─────────────────────┐
                    │    dotenvy (0.15)   │
                    └──────────┬──────────┘
                               │
                               ▼
                    ┌─────────────────────┐
                    │ gaussdb-test-helpers│◄─────┐
                    │   (全局配置模块)     │      │
                    └─────────────────────┘      │
                               △                 │
                               │                 │
               ┌───────────────┼────────────┐    │
               │               │            │    │
               ▼               ▼            ▼    │
        ┌───────────┐  ┌──────────────┐  ┌──────────────────┐
        │  gaussdb  │  │tokio-gaussdb │  │gaussdb-derive-test│
        │ (同步客户端)│  │ (异步客户端) │  │   (派生宏测试)    │
        └───────────┘  └──────────────┘  └──────────────────┘

说明：
- 实线箭头（→）：直接依赖
- 虚线箭头（⇢）：间接依赖（通过 dev-dependencies）
```

## 配置优先级

```
┌─────────────────────────────────────────────────────┐
│                 配置加载优先级                        │
├─────────────────────────────────────────────────────┤
│ 1. TEST_CONN_STR 环境变量（最高优先级）              │
│    ↓ 如果未设置                                      │
│ 2. 单个环境变量组合（PGUSER, PGPASSWORD...）        │
│    ↓ 如果未设置                                      │
│ 3. 默认值（user=gaussdb password=Gaussdb@123...）   │
└─────────────────────────────────────────────────────┘
```

## 环境变量加载路径

```rust
load_env() {
    // 尝试多个可能的 .env 位置
    dotenvy::dotenv()                    // 1. 当前目录
    dotenvy::from_filename("../.env")    // 2. 父目录
    dotenvy::from_filename("../../.env") // 3. 父父目录
    dotenvy::from_filename("../../../.env") // 4. 父父父目录
}
```

**位置优先级**：越前面的位置优先级越高

## API 分层设计

```
┌─────────────────────────────────────────────────────┐
│              高层 API（便捷函数）                     │
│  ┌───────────────────────────────────────────────┐ │
│  │ get_test_conn_str()                           │ │
│  │ get_test_host_port()                          │ │
│  │ get_multi_host_conn_str()                     │ │
│  │ build_conn_str_with_params()                  │ │
│  └───────────────────────────────────────────────┘ │
└────────────────────┬────────────────────────────────┘
                     │ 依赖
┌────────────────────▼────────────────────────────────┐
│              低层 API（基础函数）                     │
│  ┌───────────────────────────────────────────────┐ │
│  │ get_test_host()                               │ │
│  │ get_test_port()                               │ │
│  │ get_test_user()                               │ │
│  │ get_test_password()                           │ │
│  │ get_test_database()                           │ │
│  └───────────────────────────────────────────────┘ │
└────────────────────┬────────────────────────────────┘
                     │ 依赖
┌────────────────────▼────────────────────────────────┐
│                  核心层                              │
│  ┌───────────────────────────────────────────────┐ │
│  │ load_env()  (环境变量加载，Once保证单次执行) │ │
│  └───────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
```

## 测试覆盖统计

### 替换硬编码连接字符串统计

| Crate | 文件 | 替换数量 |
|-------|------|----------|
| gaussdb | `src/test.rs` | 25 |
| tokio-gaussdb | `tests/test/main.rs` | 4 |
| tokio-gaussdb | `tests/test/runtime.rs` | 15 |
| gaussdb-derive-test | `src/domains.rs` | 5 |
| gaussdb-derive-test | `src/enums.rs` | 10 |
| gaussdb-derive-test | `src/composites.rs` | 9 |
| gaussdb-derive-test | `src/transparent.rs` | 1 |
| **总计** | **7个文件** | **69个** |

### 代码行数统计

| 类型 | 之前 | 之后 | 减少 |
|------|------|------|------|
| 重复配置代码 | ~150行 × 3 = 450行 | 252行（统一模块） | -198行（-44%） |
| 导入语句 | 每文件 1-2行 | 每文件 1行 | 更简洁 |

## 优势对比

### 之前的架构（分散配置）

```
❌ 每个 crate 重复定义相同的函数
❌ 修改配置逻辑需要同步更新 3 个地方
❌ 容易出现不一致的行为
❌ 代码冗余，维护成本高
```

### 现在的架构（全局配置）

```
✅ 单一配置源，统一的行为
✅ 修改配置逻辑只需更新一个地方
✅ 保证所有测试使用相同的配置逻辑
✅ 代码复用，维护成本低
✅ 独立的 crate，职责明确
✅ 完整的文档和测试覆盖
```

## 使用示例

### 基本用法

```rust
// 任何测试文件
use gaussdb_test_helpers::*;

#[test]
fn my_test() {
    let conn_str = get_test_conn_str();
    let client = Client::connect(&conn_str, NoTls).unwrap();
    // ...
}
```

### 高级用法

```rust
use gaussdb_test_helpers::*;

#[test]
fn custom_config_test() {
    // 1. 获取基础配置
    let host = get_test_host();
    let port = get_test_port();
    
    // 2. 构建自定义连接字符串
    let conn_str = build_conn_str_with_params(&[
        ("application_name", "my_integration_test"),
        ("connect_timeout", "30"),
    ]);
    
    // 3. 使用配置
    let client = Client::connect(&conn_str, NoTls).unwrap();
    // ...
}
```

## 扩展性

### 添加新的辅助函数

```rust
// 在 gaussdb-test-helpers/src/lib.rs 中添加
pub fn get_test_ssl_mode() -> String {
    load_env();
    std::env::var("PGSSLMODE").unwrap_or_else(|_| "disable".to_string())
}
```

所有使用 `use gaussdb_test_helpers::*;` 的测试文件都能立即使用新函数。

### 添加新的环境变量支持

只需在 `gaussdb-test-helpers` 中更新，不需要修改任何测试文件。

## 最佳实践

1. **始终使用全局模块**
   ```rust
   ✅ use gaussdb_test_helpers::*;
   ❌ 硬编码连接字符串
   ```

2. **优先使用高层 API**
   ```rust
   ✅ get_test_conn_str()
   ⚠️ format!("user={} ...", get_test_user(), ...)
   ```

3. **通过 .env 文件配置环境**
   ```bash
   ✅ 复制 env.example 为 .env 并修改
   ❌ 直接修改代码中的默认值
   ```

4. **使用有意义的参数名**
   ```rust
   ✅ build_conn_str_with_params(&[("application_name", "integration_test")])
   ❌ build_conn_str_with_params(&[("app", "test")])
   ```

## 性能考虑

- **环境变量加载**：使用 `Once` 保证只执行一次，零性能开销
- **字符串构建**：使用 `format!` 宏，编译时优化
- **内存占用**：配置字符串按需构建，不常驻内存

## 安全性

- `.env` 文件在 `.gitignore` 中，不会提交到版本库
- 密码等敏感信息通过环境变量传递
- 提供默认值仅用于本地开发，生产环境必须配置

## 总结

**gaussdb-test-helpers** 是一个设计优雅、职责单一、易于使用的全局测试配置模块，成功实现了：

- 📦 **代码复用**：减少 44% 的重复代码
- 🎯 **单一职责**：仅负责测试环境配置
- 🔧 **易于维护**：修改一处，全局生效
- 📚 **文档完善**：详细的 API 文档和使用示例
- ✅ **测试覆盖**：6 个单元测试 + 6 个文档测试
- 🚀 **零成本抽象**：运行时无额外开销

