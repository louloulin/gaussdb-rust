# GaussDB/OpenGauss 与 PostgreSQL 兼容性文档

## 📋 **概述**

本文档详细记录了 GaussDB/OpenGauss 与 PostgreSQL 之间的不兼容性问题，以及在使用 GaussDB Rust 驱动时需要注意的差异。

## 🔍 **测试环境**

- **GaussDB 版本**: OpenGauss 7.0.0-RC1
- **PostgreSQL 兼容版本**: 12.x
- **测试时间**: 2025年5月31日
- **驱动版本**: gaussdb-rust v0.1.0

## ❌ **已知不兼容性**

### 1. **LISTEN/NOTIFY 功能不支持**

#### **问题描述**
OpenGauss 尚未实现 PostgreSQL 的 LISTEN/NOTIFY 异步通知功能。

#### **错误信息**
```
ERROR: LISTEN statement is not yet supported.
```

#### **影响的功能**
- 实时通知系统
- 事件驱动架构
- 数据库触发的应用程序通知

#### **解决方案**
- 使用轮询机制替代
- 实现应用层的事件系统
- 等待 OpenGauss 后续版本支持

#### **代码示例**
```rust
// ❌ 不支持的代码
client.batch_execute("LISTEN channel_name").unwrap();

// ✅ 替代方案
loop {
    let rows = client.query("SELECT * FROM events WHERE processed = false", &[]).unwrap();
    // 处理事件
    thread::sleep(Duration::from_secs(1));
}
```

### 2. **临时表不支持 SERIAL 列**

#### **问题描述**
OpenGauss 不允许在临时表上创建 SERIAL 类型的列。

#### **错误信息**
```
ERROR: It's not supported to create serial column on temporary table
```

#### **影响的功能**
- 临时表的自增主键
- 测试代码中的临时数据结构

#### **解决方案**
- 使用普通表替代临时表
- 手动管理序列号
- 使用 INT 类型配合应用程序生成 ID

#### **代码示例**
```sql
-- ❌ 不支持的语法
CREATE TEMPORARY TABLE temp_table (
    id SERIAL PRIMARY KEY,
    data TEXT
);

-- ✅ 替代方案1: 使用普通表
CREATE TABLE temp_table (
    id INT PRIMARY KEY,
    data TEXT
);

-- ✅ 替代方案2: 手动序列
CREATE SEQUENCE temp_seq;
CREATE TEMPORARY TABLE temp_table (
    id INT DEFAULT nextval('temp_seq') PRIMARY KEY,
    data TEXT
);
```

### 3. **ON CONFLICT 语法不支持**

#### **问题描述**
OpenGauss 不支持 PostgreSQL 9.5+ 引入的 ON CONFLICT 语法。

#### **错误信息**
```
ERROR: syntax error at or near "CONFLICT"
```

#### **影响的功能**
- UPSERT 操作
- 数据去重插入
- 批量数据导入

#### **解决方案**
- 使用 WHERE NOT EXISTS 模式
- 先查询再插入的两步操作
- 使用 MERGE 语句（如果支持）

#### **代码示例**
```sql
-- ❌ 不支持的语法
INSERT INTO users (id, name) VALUES (1, 'Alice') 
ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name;

-- ✅ 替代方案
INSERT INTO users (id, name) 
SELECT 1, 'Alice' 
WHERE NOT EXISTS (SELECT 1 FROM users WHERE id = 1);
```

### 4. **二进制复制格式差异**

#### **问题描述**
OpenGauss 的二进制 COPY 格式与 PostgreSQL 存在细微差异。

#### **错误信息**
```
ERROR: unexpected EOF
```

#### **影响的功能**
- 高性能数据导入
- 二进制数据传输
- 大批量数据操作

#### **解决方案**
- 使用文本格式的 COPY
- 分批处理数据
- 使用标准 INSERT 语句

#### **代码示例**
```rust
// ❌ 可能有问题的代码
let writer = client.copy_in("COPY table FROM stdin BINARY").unwrap();

// ✅ 推荐的替代方案
let writer = client.copy_in("COPY table FROM stdin").unwrap(); // 文本格式
```

## ✅ **兼容性良好的功能**

### 1. **基础 SQL 操作**
- SELECT, INSERT, UPDATE, DELETE
- JOIN 操作
- 子查询和 CTE

### 2. **数据类型**
- 基础数据类型 (INT, VARCHAR, TIMESTAMP 等)
- 数组类型
- JSON 类型（部分支持）

### 3. **事务管理**
- BEGIN, COMMIT, ROLLBACK
- 保存点 (SAVEPOINT)
- 嵌套事务

### 4. **索引和约束**
- B-tree, Hash 索引
- 主键和外键约束
- 唯一约束和检查约束

## 🔧 **迁移建议**

### 1. **代码审查清单**
- [ ] 检查是否使用了 LISTEN/NOTIFY
- [ ] 审查临时表中的 SERIAL 列
- [ ] 替换 ON CONFLICT 语法
- [ ] 验证二进制 COPY 操作

### 2. **测试策略**
- 在 OpenGauss 环境中运行完整测试套件
- 重点测试数据导入/导出功能
- 验证事务处理逻辑

### 3. **性能优化**
- 使用批量插入替代单条插入
- 优化查询语句以适应 OpenGauss 优化器
- 合理使用索引

## 📊 **兼容性矩阵**

| 功能类别 | PostgreSQL | OpenGauss | 兼容性 | 备注 |
|---------|------------|-----------|--------|------|
| 基础查询 | ✅ | ✅ | 100% | 完全兼容 |
| 事务管理 | ✅ | ✅ | 100% | 完全兼容 |
| 数据类型 | ✅ | ✅ | 95% | 大部分兼容 |
| 索引 | ✅ | ✅ | 90% | 基础索引兼容 |
| COPY 操作 | ✅ | ⚠️ | 80% | 文本格式兼容 |
| LISTEN/NOTIFY | ✅ | ❌ | 0% | 不支持 |
| ON CONFLICT | ✅ | ❌ | 0% | 不支持 |
| 临时表 SERIAL | ✅ | ❌ | 0% | 不支持 |

## 🚀 **最佳实践**

### 1. **编写兼容代码**
```rust
// 使用条件编译处理差异
#[cfg(feature = "gaussdb")]
fn create_table() {
    // GaussDB 特定实现
}

#[cfg(not(feature = "gaussdb"))]
fn create_table() {
    // PostgreSQL 实现
}
```

### 2. **错误处理**
```rust
match client.execute(sql, &[]) {
    Ok(_) => println!("执行成功"),
    Err(e) if e.code() == Some(&SqlState::FEATURE_NOT_SUPPORTED) => {
        println!("功能不支持，使用替代方案");
        // 实现替代逻辑
    }
    Err(e) => return Err(e),
}
```

### 3. **配置管理**
```toml
[dependencies]
gaussdb-rust = { version = "0.1", features = ["gaussdb-compat"] }
```

## 📞 **支持和反馈**

如果您发现新的兼容性问题或有改进建议，请：

1. 提交 GitHub Issue
2. 发送邮件至项目维护者
3. 参与社区讨论

## 🔬 **技术细节**

### **认证差异**

#### **SHA256 认证**
GaussDB 使用的 SHA256 认证实际上是 SHA256_MD5 混合算法：
```
1. MD5(password + username) -> md5_hex
2. SHA256(md5_hex + salt) -> sha256_hex
3. "sha256" + sha256_hex
```

#### **MD5_SHA256 认证**
复杂的 PBKDF2 + HMAC-SHA256 算法：
```
1. PBKDF2(password, random_code, 2048) -> K
2. HMAC-SHA256(K, "Server Key") -> server_key
3. HMAC-SHA256(K, "Client Key") -> client_key
4. SHA256(client_key) -> stored_key
5. MD5(random_code + server_key_hex + stored_key_hex + salt)
```

### **SQL 方言差异**

#### **数据类型映射**
| PostgreSQL | OpenGauss | 兼容性 | 说明 |
|------------|-----------|--------|------|
| SERIAL | INT + SEQUENCE | ⚠️ | 临时表不支持 |
| BIGSERIAL | BIGINT + SEQUENCE | ⚠️ | 临时表不支持 |
| BOOLEAN | BOOLEAN | ✅ | 完全兼容 |
| JSON | JSON | ⚠️ | 部分函数不同 |
| JSONB | - | ❌ | 不支持 |

#### **函数差异**
```sql
-- PostgreSQL
SELECT json_extract_path_text('{"a":1}', 'a');

-- OpenGauss (可能需要不同语法)
SELECT JSON_UNQUOTE(JSON_EXTRACT('{"a":1}', '$.a'));
```

## 📋 **迁移检查清单**

### **代码审查**
- [ ] 搜索 `LISTEN` 和 `NOTIFY` 关键字
- [ ] 检查 `ON CONFLICT` 语句
- [ ] 审查临时表中的 `SERIAL` 列
- [ ] 验证 `COPY ... BINARY` 操作
- [ ] 检查 JSONB 数据类型使用

### **测试验证**
- [ ] 运行完整测试套件
- [ ] 验证数据导入/导出
- [ ] 测试事务回滚
- [ ] 检查并发操作
- [ ] 验证性能基准

### **部署准备**
- [ ] 更新连接字符串
- [ ] 配置认证方法
- [ ] 设置监控和日志
- [ ] 准备回滚计划

---

**最后更新**: 2025年5月31日
**文档版本**: v1.0
**适用驱动版本**: gaussdb-rust v0.1.0+
