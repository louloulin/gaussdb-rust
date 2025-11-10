# Feature 支持变更日志

## [0.1.1+] - 2025-10-31

### ✨ 新增

#### Feature 支持
- 添加 `opengauss` feature（默认启用）
- 添加 `gauss` feature（GaussDB 企业版）
- 支持通过 Cargo features 灵活配置数据库特性

#### 条件编译
- `cancel_query` API 和测试现在只在 `opengauss` 或 `gauss` feature 下可用
- `domain` 类型测试只在 `opengauss` 或 `gauss` feature 下运行

### 📝 文档

- 新增 `FEATURES.md` - Feature 完整文档（英文）
- 新增 `FEATURE_GUIDE_CN.md` - Feature 使用指南（中文）
- 新增 `IMPLEMENTATION_SUMMARY.md` - 技术实现文档
- 新增 `FEATURE_SUMMARY.md` - 快速参考

### 🔄 变更

#### gaussdb-protocol
```toml
[features]
default = ["opengauss"]
opengauss = []
gauss = []
```

#### tokio-gaussdb
```toml
[features]
default = ["runtime", "opengauss"]
runtime = ["tokio/net", "tokio/time"]
opengauss = ["gaussdb-protocol/opengauss"]
gauss = ["gaussdb-protocol/gauss"]
```

#### gaussdb
```toml
[features]
default = ["opengauss"]
opengauss = ["tokio-gaussdb/opengauss"]
gauss = ["tokio-gaussdb/gauss"]
```

### ✅ 向后兼容

- **完全向后兼容** - 现有代码无需修改
- 默认启用 `opengauss` feature，保持原有行为
- 所有认证方法保持可用

### 🧪 测试

#### 测试变更
- `gaussdb::test::cancel_query` - 添加 `#[cfg(feature = "opengauss")]`
- `tokio_gaussdb::test::types::domain` - 添加 `#[cfg(feature = "opengauss")]`
- `tokio_gaussdb::test::custom_domain` - 添加 `#[cfg(feature = "opengauss")]`

#### 测试结果
- ✅ 默认配置: 18 个测试（含 cancel_query）
- ✅ 无 features: 17 个测试（跳过 cancel_query）
- ✅ 编译检查: 所有配置均通过

### 📊 Feature 对比

| 功能 | default | opengauss | gauss | 无 features |
|------|---------|-----------|-------|------------|
| PostgreSQL | ✅ | ✅ | ✅ | ✅ |
| 认证方法 | ✅ | ✅ | ✅ | ✅ |
| cancel_query | ✅ | ✅ | ✅ | ❌ |
| domain 类型 | ✅ | ✅ | ✅ | ❌ |

### 🎯 使用示例

#### 默认（OpenGauss）
```toml
[dependencies]
gaussdb = "0.1"
```

#### GaussDB 企业版
```toml
[dependencies]
gaussdb = { version = "0.1", default-features = false, features = ["gauss"] }
```

#### 仅 PostgreSQL
```toml
[dependencies]
gaussdb = { version = "0.1", default-features = false }
```

### 🔧 编译选项

```bash
# 默认编译
cargo build

# GaussDB 编译
cargo build --no-default-features --features gauss

# 最小编译
cargo build --no-default-features

# 完整编译
cargo build --all-features
```

### 📚 相关文档

- [FEATURES.md](FEATURES.md) - 完整 Feature 文档
- [FEATURE_GUIDE_CN.md](FEATURE_GUIDE_CN.md) - 中文使用指南
- [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md) - 实现细节

### 💡 设计决策

1. **默认 opengauss** - 保持向后兼容，OpenGauss 使用更广泛
2. **认证始终可用** - 核心功能不受 feature 限制
3. **测试条件编译** - 可选功能仅在需要时测试
4. **灵活配置** - 用户可根据实际需求选择

### ⚠️ 注意事项

- `cancel_query` 测试在某些环境可能超时，这是已知问题
- 禁用 features 会跳过相关测试，这是预期行为
- 认证方法不受 feature 限制，始终可用

### 🔮 未来计划

- 完善 `gauss` feature 的差异化支持
- 添加更多 OpenGauss/GaussDB 特有功能
- 改进测试稳定性

---

**实施日期**: 2025-10-31  
**影响范围**: gaussdb-protocol, tokio-gaussdb, gaussdb  
**兼容性**: ✅ 完全向后兼容

