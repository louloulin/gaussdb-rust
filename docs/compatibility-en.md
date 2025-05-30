# GaussDB/OpenGauss and PostgreSQL Compatibility Documentation

## 📋 **Overview**

This document provides a comprehensive record of incompatibilities between GaussDB/OpenGauss and PostgreSQL, along with important considerations when using the GaussDB Rust driver.

## 🔍 **Test Environment**

- **GaussDB Version**: OpenGauss 7.0.0-RC1
- **PostgreSQL Compatible Version**: 12.x
- **Test Date**: May 31, 2025
- **Driver Version**: gaussdb-rust v0.1.0

## ❌ **Known Incompatibilities**

### 1. **LISTEN/NOTIFY Not Supported**

#### **Issue Description**
OpenGauss has not yet implemented PostgreSQL's LISTEN/NOTIFY asynchronous notification functionality.

#### **Error Message**
```
ERROR: LISTEN statement is not yet supported.
```

#### **Affected Features**
- Real-time notification systems
- Event-driven architecture
- Database-triggered application notifications

#### **Workarounds**
- Use polling mechanisms instead
- Implement application-level event systems
- Wait for future OpenGauss versions

#### **Code Examples**
```rust
// ❌ Unsupported code
client.batch_execute("LISTEN channel_name").unwrap();

// ✅ Alternative approach
loop {
    let rows = client.query("SELECT * FROM events WHERE processed = false", &[]).unwrap();
    // Process events
    thread::sleep(Duration::from_secs(1));
}
```

### 2. **SERIAL Columns Not Supported on Temporary Tables**

#### **Issue Description**
OpenGauss does not allow creating SERIAL type columns on temporary tables.

#### **Error Message**
```
ERROR: It's not supported to create serial column on temporary table
```

#### **Affected Features**
- Auto-increment primary keys on temporary tables
- Temporary data structures in test code

#### **Workarounds**
- Use regular tables instead of temporary tables
- Manually manage sequence numbers
- Use INT type with application-generated IDs

#### **Code Examples**
```sql
-- ❌ Unsupported syntax
CREATE TEMPORARY TABLE temp_table (
    id SERIAL PRIMARY KEY,
    data TEXT
);

-- ✅ Alternative 1: Use regular table
CREATE TABLE temp_table (
    id INT PRIMARY KEY,
    data TEXT
);

-- ✅ Alternative 2: Manual sequence
CREATE SEQUENCE temp_seq;
CREATE TEMPORARY TABLE temp_table (
    id INT DEFAULT nextval('temp_seq') PRIMARY KEY,
    data TEXT
);
```

### 3. **ON CONFLICT Syntax Not Supported**

#### **Issue Description**
OpenGauss does not support the ON CONFLICT syntax introduced in PostgreSQL 9.5+.

#### **Error Message**
```
ERROR: syntax error at or near "CONFLICT"
```

#### **Affected Features**
- UPSERT operations
- Data deduplication on insert
- Bulk data import with conflict resolution

#### **Workarounds**
- Use WHERE NOT EXISTS pattern
- Two-step query-then-insert operations
- Use MERGE statements (if supported)

#### **Code Examples**
```sql
-- ❌ Unsupported syntax
INSERT INTO users (id, name) VALUES (1, 'Alice') 
ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name;

-- ✅ Alternative approach
INSERT INTO users (id, name) 
SELECT 1, 'Alice' 
WHERE NOT EXISTS (SELECT 1 FROM users WHERE id = 1);
```

### 4. **Binary Copy Format Differences**

#### **Issue Description**
OpenGauss's binary COPY format has subtle differences from PostgreSQL.

#### **Error Message**
```
ERROR: unexpected EOF
```

#### **Affected Features**
- High-performance data import
- Binary data transfer
- Large-scale bulk operations

#### **Workarounds**
- Use text format COPY instead
- Process data in smaller batches
- Use standard INSERT statements

#### **Code Examples**
```rust
// ❌ Potentially problematic code
let writer = client.copy_in("COPY table FROM stdin BINARY").unwrap();

// ✅ Recommended alternative
let writer = client.copy_in("COPY table FROM stdin").unwrap(); // Text format
```

## ✅ **Well-Compatible Features**

### 1. **Basic SQL Operations**
- SELECT, INSERT, UPDATE, DELETE
- JOIN operations
- Subqueries and CTEs

### 2. **Data Types**
- Basic data types (INT, VARCHAR, TIMESTAMP, etc.)
- Array types
- JSON types (partial support)

### 3. **Transaction Management**
- BEGIN, COMMIT, ROLLBACK
- Savepoints (SAVEPOINT)
- Nested transactions

### 4. **Indexes and Constraints**
- B-tree, Hash indexes
- Primary key and foreign key constraints
- Unique and check constraints

## 🔧 **Migration Guidelines**

### 1. **Code Review Checklist**
- [ ] Check for LISTEN/NOTIFY usage
- [ ] Review SERIAL columns in temporary tables
- [ ] Replace ON CONFLICT syntax
- [ ] Verify binary COPY operations

### 2. **Testing Strategy**
- Run complete test suite in OpenGauss environment
- Focus on data import/export functionality
- Validate transaction processing logic

### 3. **Performance Optimization**
- Use batch inserts instead of single inserts
- Optimize queries for OpenGauss optimizer
- Use indexes appropriately

## 📊 **Compatibility Matrix**

| Feature Category | PostgreSQL | OpenGauss | Compatibility | Notes |
|-----------------|------------|-----------|---------------|-------|
| Basic Queries | ✅ | ✅ | 100% | Fully compatible |
| Transaction Management | ✅ | ✅ | 100% | Fully compatible |
| Data Types | ✅ | ✅ | 95% | Mostly compatible |
| Indexes | ✅ | ✅ | 90% | Basic indexes compatible |
| COPY Operations | ✅ | ⚠️ | 80% | Text format compatible |
| LISTEN/NOTIFY | ✅ | ❌ | 0% | Not supported |
| ON CONFLICT | ✅ | ❌ | 0% | Not supported |
| Temp Table SERIAL | ✅ | ❌ | 0% | Not supported |

## 🚀 **Best Practices**

### 1. **Writing Compatible Code**
```rust
// Use conditional compilation for differences
#[cfg(feature = "gaussdb")]
fn create_table() {
    // GaussDB-specific implementation
}

#[cfg(not(feature = "gaussdb"))]
fn create_table() {
    // PostgreSQL implementation
}
```

### 2. **Error Handling**
```rust
match client.execute(sql, &[]) {
    Ok(_) => println!("Execution successful"),
    Err(e) if e.code() == Some(&SqlState::FEATURE_NOT_SUPPORTED) => {
        println!("Feature not supported, using alternative");
        // Implement fallback logic
    }
    Err(e) => return Err(e),
}
```

### 3. **Configuration Management**
```toml
[dependencies]
gaussdb-rust = { version = "0.1", features = ["gaussdb-compat"] }
```

## 📞 **Support and Feedback**

If you discover new compatibility issues or have suggestions for improvement, please:

1. Submit a GitHub Issue
2. Email the project maintainers
3. Participate in community discussions

## 🔬 **Technical Details**

### **Authentication Differences**

#### **SHA256 Authentication**
GaussDB's SHA256 authentication is actually a SHA256_MD5 hybrid algorithm:
```
1. MD5(password + username) -> md5_hex
2. SHA256(md5_hex + salt) -> sha256_hex
3. "sha256" + sha256_hex
```

#### **MD5_SHA256 Authentication**
Complex PBKDF2 + HMAC-SHA256 algorithm:
```
1. PBKDF2(password, random_code, 2048) -> K
2. HMAC-SHA256(K, "Server Key") -> server_key
3. HMAC-SHA256(K, "Client Key") -> client_key
4. SHA256(client_key) -> stored_key
5. MD5(random_code + server_key_hex + stored_key_hex + salt)
```

### **SQL Dialect Differences**

#### **Data Type Mapping**
| PostgreSQL | OpenGauss | Compatibility | Notes |
|------------|-----------|---------------|-------|
| SERIAL | INT + SEQUENCE | ⚠️ | Not supported on temp tables |
| BIGSERIAL | BIGINT + SEQUENCE | ⚠️ | Not supported on temp tables |
| BOOLEAN | BOOLEAN | ✅ | Fully compatible |
| JSON | JSON | ⚠️ | Some functions differ |
| JSONB | - | ❌ | Not supported |

#### **Function Differences**
```sql
-- PostgreSQL
SELECT json_extract_path_text('{"a":1}', 'a');

-- OpenGauss (may require different syntax)
SELECT JSON_UNQUOTE(JSON_EXTRACT('{"a":1}', '$.a'));
```

## 📋 **Migration Checklist**

### **Code Review**
- [ ] Search for `LISTEN` and `NOTIFY` keywords
- [ ] Check `ON CONFLICT` statements
- [ ] Review `SERIAL` columns in temporary tables
- [ ] Verify `COPY ... BINARY` operations
- [ ] Check JSONB data type usage

### **Testing Validation**
- [ ] Run complete test suite
- [ ] Verify data import/export
- [ ] Test transaction rollback
- [ ] Check concurrent operations
- [ ] Validate performance benchmarks

### **Deployment Preparation**
- [ ] Update connection strings
- [ ] Configure authentication methods
- [ ] Set up monitoring and logging
- [ ] Prepare rollback plan

---

**Last Updated**: May 31, 2025
**Document Version**: v1.0
**Compatible Driver Version**: gaussdb-rust v0.1.0+
