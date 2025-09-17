# Change Log

## v0.1.0 - 2025-09-17

### Added

* **GaussDB SCRAM-SHA-256 兼容性支持**: 新增完整的 GaussDB SASL 认证支持
  * 新增 `GaussDbScramSha256` 认证器，支持 GaussDB 特有的 SASL 消息格式
  * 新增 `GaussDbSaslParser` 解析器，支持三种兼容模式：标准、GaussDB、自动检测
  * 新增 `CompatibilityMode` 枚举，控制 SASL 消息解析行为
  * 新增 `create_gaussdb_scram` 辅助函数，简化 GaussDB SCRAM 认证器创建
* **增强的 SASL 消息处理**: 改进 SASL 消息解析和错误处理
  * 支持处理带有尾随数据的 SASL 消息（GaussDB 特有格式）
  * 智能检测和处理不同格式的服务器响应
  * 改进错误诊断，提供更详细的解析失败信息
* **全面的测试覆盖**: 新增 37 个单元测试，覆盖所有新功能
  * SASL 兼容性测试（标准模式、GaussDB 模式、自动模式）
  * 边界情况和错误处理测试
  * 空白字符处理测试
  * SCRAM-SHA-256 认证器创建和消息解析测试

### Fixed

* **SASL 消息解析**: 修复 GaussDB SASL 消息中尾随数据导致的解析失败
* **兼容性问题**: 解决与 GaussDB/openGauss 服务器的协议兼容性问题
* **错误处理**: 改进 SASL 认证过程中的错误检测和报告

### Enhanced

* **向后兼容**: 保持与现有 PostgreSQL SASL 实现的完全兼容
* **性能优化**: 优化 SASL 消息解析性能，减少不必要的内存分配
* **代码质量**: 添加详细的代码注释和文档

## v0.6.8 - 2025-02-02

### Changed

* Upgraded `getrandom`.

## v0.6.7 - 2024-07-21

### Deprecated

* Deprecated `ErrorField::value`.

### Added

* Added a `Clone` implementation for `DataRowBody`.
* Added `ErrorField::value_bytes`.

### Changed

* Upgraded `base64`.

## v0.6.6 - 2023-08-19

### Added

* Added the `js` feature for WASM support.

## v0.6.5 - 2023-03-27

### Added

* Added `message::frontend::flush`.
* Added `DataRowBody::buffer_bytes`.

### Changed

* Upgraded `base64`.

## v0.6.4 - 2022-04-03

### Added

* Added parsing support for `ltree`, `lquery`, and `ltxtquery`.

## v0.6.3 - 2021-12-10

### Changed

* Upgraded `hmac`, `md-5` and `sha`.

## v0.6.2 - 2021-09-29

### Changed

* Upgraded `hmac`.

## v0.6.1 - 2021-04-03

### Added

* Added the `password` module, which can be used to hash passwords before using them in queries like `ALTER USER`.
* Added type conversions for `LSN`.

### Changed

* Moved from `md5` to `md-5`.

## v0.6.0 - 2020-12-25

### Changed

* Upgraded `bytes`, `hmac`, and `rand`.

### Added

* Added `escape::{escape_literal, escape_identifier}`.

## v0.5.3 - 2020-10-17

### Changed

* Upgraded `base64` and `hmac`.

## v0.5.2 - 2020-07-06

### Changed

* Upgraded `hmac` and `sha2`.

## v0.5.1 - 2020-03-17

### Changed

* Upgraded `base64` to 0.12.

## v0.5.0 - 2019-12-23

### Changed

* `frontend::Message` is now a true non-exhaustive enum.

## v0.5.0-alpha.2 - 2019-11-27

### Changed

* Upgraded `bytes` to 0.5.

## v0.5.0-alpha.1 - 2019-10-14

### Changed

* Frontend messages and types now serialize to `BytesMut` rather than `Vec<u8>`.

## v0.4.1 - 2019-06-29

### Added

* Added `backend::Framed` to minimally parse the structure of backend messages.

## v0.4.0 - 2019-03-05

### Added

* Added channel binding support to SCRAM authentication API.

### Changed

* Passwords are no longer required to be UTF8 strings.
* `types::array_to_sql` now automatically computes the required flags and no longer takes a has_nulls parameter.

## Older

Look at the [release tags] for information about older releases.

[release tags]: https://github.com/sfackler/rust-postgres/releases
