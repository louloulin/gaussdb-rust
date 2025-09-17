//! 自适应认证管理器
//!
//! 这个模块提供了智能的认证方法选择和回退机制，
//! 能够自动处理不同数据库系统之间的认证兼容性问题。

use crate::{Config, Error};
use gaussdb_protocol::authentication::gaussdb_sasl::{GaussDbScramSha256, CompatibilityMode};
use gaussdb_protocol::authentication::sasl::ChannelBinding;
use gaussdb_protocol::message::backend::{AuthenticationSaslBody, Message};
use fallible_iterator::FallibleIterator;
use std::collections::HashMap;
use std::time::Instant;

/// 自适应认证管理器
///
/// 负责管理不同的认证方法，自动检测服务器支持的认证类型，
/// 并在认证失败时提供智能的回退机制。
pub struct AdaptiveAuthManager {
    /// 认证方法偏好顺序
    auth_preferences: Vec<AuthMethod>,
    /// 服务器兼容性缓存
    compatibility_cache: HashMap<String, ServerCompatibility>,
    /// 认证统计信息
    stats: AuthStats,
}

/// 支持的认证方法
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuthMethod {
    /// SCRAM-SHA-256 (标准模式)
    ScramSha256Standard,
    /// SCRAM-SHA-256 (GaussDB 兼容模式)
    ScramSha256GaussDb,
    /// SHA256 (GaussDB 特有)
    Sha256,
    /// MD5_SHA256 (GaussDB 特有)
    Md5Sha256,
    /// MD5 (标准)
    Md5,
    /// 明文密码
    Cleartext,
}

/// 服务器兼容性信息
#[derive(Debug, Clone)]
struct ServerCompatibility {
    /// 支持的认证方法
    supported_methods: Vec<AuthMethod>,
    /// 推荐的认证方法
    recommended_method: AuthMethod,
    /// 最后更新时间
    last_updated: Instant,
    /// 服务器类型检测结果
    server_type: ServerType,
}

/// 服务器类型
#[derive(Debug, Clone, PartialEq)]
pub enum ServerType {
    /// 标准 PostgreSQL
    PostgreSQL,
    /// GaussDB/openGauss
    GaussDB,
    /// 未知类型
    Unknown,
}

/// 认证统计信息
#[derive(Debug, Default)]
pub struct AuthStats {
    /// 成功认证次数
    successful_auths: u64,
    /// 失败认证次数
    failed_auths: u64,
    /// 各认证方法的使用统计
    method_usage: HashMap<AuthMethod, u64>,
}

impl AdaptiveAuthManager {
    /// 创建新的自适应认证管理器
    pub fn new() -> Self {
        Self {
            auth_preferences: vec![
                // 优先使用 GaussDB 特有的认证方法
                AuthMethod::Sha256,
                AuthMethod::Md5Sha256,
                // 然后尝试 SCRAM (GaussDB 兼容模式)
                AuthMethod::ScramSha256GaussDb,
                // 标准 SCRAM
                AuthMethod::ScramSha256Standard,
                // 回退到 MD5
                AuthMethod::Md5,
                // 最后尝试明文 (仅用于测试)
                AuthMethod::Cleartext,
            ],
            compatibility_cache: HashMap::new(),
            stats: AuthStats::default(),
        }
    }

    /// 检测服务器类型和支持的认证方法
    pub fn detect_server_compatibility(&mut self, server_version: Option<&str>) -> ServerType {
        if let Some(version) = server_version {
            if version.contains("openGauss") || version.contains("GaussDB") {
                ServerType::GaussDB
            } else if version.contains("PostgreSQL") {
                ServerType::PostgreSQL
            } else {
                ServerType::Unknown
            }
        } else {
            ServerType::Unknown
        }
    }

    /// 根据服务器消息选择最佳认证方法
    pub fn select_auth_method(&mut self, message: &Message, config: &Config) -> Result<AuthStrategy, Error> {
        match message {
            Message::AuthenticationSasl(body) => {
                self.handle_sasl_auth(body, config)
            }
            Message::AuthenticationSha256Password(_) => {
                Ok(AuthStrategy::Sha256)
            }
            Message::AuthenticationMd5Sha256Password(_) => {
                Ok(AuthStrategy::Md5Sha256)
            }
            Message::AuthenticationMd5Password(_) => {
                Ok(AuthStrategy::Md5)
            }
            Message::AuthenticationCleartextPassword => {
                Ok(AuthStrategy::Cleartext)
            }
            _ => {
                Err(Error::authentication("unsupported authentication method".into()))
            }
        }
    }

    /// 处理 SASL 认证
    fn handle_sasl_auth(&mut self, body: &AuthenticationSaslBody, config: &Config) -> Result<AuthStrategy, Error> {
        let mut mechanisms = body.mechanisms();
        let mut supported_scram = false;
        let mut supported_scram_plus = false;

        // 检查支持的 SASL 机制
        while let Some(mechanism) = mechanisms.next().map_err(Error::parse)? {
            match mechanism {
                "SCRAM-SHA-256" => supported_scram = true,
                "SCRAM-SHA-256-PLUS" => supported_scram_plus = true,
                _ => {}
            }
        }

        if supported_scram || supported_scram_plus {
            // 首先尝试 GaussDB 兼容模式
            Ok(AuthStrategy::ScramSha256 {
                compatibility_mode: CompatibilityMode::Auto,
                use_plus: supported_scram_plus && config.channel_binding != crate::config::ChannelBinding::Disable,
            })
        } else {
            Err(Error::authentication("no supported SASL mechanisms".into()))
        }
    }

    /// 记录认证结果
    pub fn record_auth_result(&mut self, method: &AuthMethod, success: bool) {
        if success {
            self.stats.successful_auths += 1;
        } else {
            self.stats.failed_auths += 1;
        }

        *self.stats.method_usage.entry(method.clone()).or_insert(0) += 1;
    }

    /// 获取认证统计信息
    pub fn get_stats(&self) -> &AuthStats {
        &self.stats
    }

    /// 获取推荐的认证方法顺序
    pub fn get_recommended_methods(&self, server_type: ServerType) -> Vec<AuthMethod> {
        match server_type {
            ServerType::GaussDB => vec![
                AuthMethod::Sha256,
                AuthMethod::Md5Sha256,
                AuthMethod::ScramSha256GaussDb,
                AuthMethod::Md5,
            ],
            ServerType::PostgreSQL => vec![
                AuthMethod::ScramSha256Standard,
                AuthMethod::Md5,
                AuthMethod::Cleartext,
            ],
            ServerType::Unknown => self.auth_preferences.clone(),
        }
    }
}

/// 认证策略
#[derive(Debug)]
pub enum AuthStrategy {
    /// SCRAM-SHA-256 认证
    ScramSha256 {
        /// 兼容模式
        compatibility_mode: CompatibilityMode,
        /// 是否使用 PLUS 变体
        use_plus: bool,
    },
    /// SHA256 认证 (GaussDB 特有)
    Sha256,
    /// MD5_SHA256 认证 (GaussDB 特有)
    Md5Sha256,
    /// MD5 认证
    Md5,
    /// 明文认证
    Cleartext,
}

impl Default for AdaptiveAuthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建 GaussDB 兼容的 SCRAM 认证器
pub fn create_gaussdb_scram(
    password: &[u8],
    channel_binding: ChannelBinding,
    compatibility_mode: CompatibilityMode,
) -> GaussDbScramSha256 {
    GaussDbScramSha256::new_with_compatibility(password, channel_binding, compatibility_mode)
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    fn test_server_type_detection() {
        let mut manager = AdaptiveAuthManager::new();

        assert_eq!(
            manager.detect_server_compatibility(Some("openGauss 3.0.0")),
            ServerType::GaussDB
        );

        assert_eq!(
            manager.detect_server_compatibility(Some("GaussDB 5.0.1")),
            ServerType::GaussDB
        );

        assert_eq!(
            manager.detect_server_compatibility(Some("PostgreSQL 14.5")),
            ServerType::PostgreSQL
        );

        assert_eq!(
            manager.detect_server_compatibility(Some("PostgreSQL 15.2 on x86_64")),
            ServerType::PostgreSQL
        );

        assert_eq!(
            manager.detect_server_compatibility(None),
            ServerType::Unknown
        );

        assert_eq!(
            manager.detect_server_compatibility(Some("Unknown Database 1.0")),
            ServerType::Unknown
        );
    }

    #[test]
    fn test_auth_method_preferences() {
        let manager = AdaptiveAuthManager::new();

        let gaussdb_methods = manager.get_recommended_methods(ServerType::GaussDB);
        assert_eq!(gaussdb_methods[0], AuthMethod::Sha256);
        assert_eq!(gaussdb_methods[1], AuthMethod::Md5Sha256);
        assert_eq!(gaussdb_methods[2], AuthMethod::ScramSha256GaussDb);

        let postgres_methods = manager.get_recommended_methods(ServerType::PostgreSQL);
        assert_eq!(postgres_methods[0], AuthMethod::ScramSha256Standard);
        assert_eq!(postgres_methods[1], AuthMethod::Md5);

        let unknown_methods = manager.get_recommended_methods(ServerType::Unknown);
        assert!(!unknown_methods.is_empty());
        assert_eq!(unknown_methods[0], AuthMethod::Sha256); // 默认偏好
    }

    #[test]
    fn test_auth_stats() {
        let mut manager = AdaptiveAuthManager::new();

        // 测试初始状态
        let stats = manager.get_stats();
        assert_eq!(stats.successful_auths, 0);
        assert_eq!(stats.failed_auths, 0);
        assert!(stats.method_usage.is_empty());

        // 记录成功认证
        manager.record_auth_result(&AuthMethod::Sha256, true);
        manager.record_auth_result(&AuthMethod::Sha256, true);
        manager.record_auth_result(&AuthMethod::ScramSha256GaussDb, false);
        manager.record_auth_result(&AuthMethod::Md5, true);

        let stats = manager.get_stats();
        assert_eq!(stats.successful_auths, 3);
        assert_eq!(stats.failed_auths, 1);
        assert_eq!(stats.method_usage.get(&AuthMethod::Sha256), Some(&2));
        assert_eq!(stats.method_usage.get(&AuthMethod::ScramSha256GaussDb), Some(&1));
        assert_eq!(stats.method_usage.get(&AuthMethod::Md5), Some(&1));
    }

    #[test]
    fn test_auth_method_equality() {
        // 测试认证方法的相等性比较
        assert_eq!(AuthMethod::Sha256, AuthMethod::Sha256);
        assert_ne!(AuthMethod::Sha256, AuthMethod::Md5);
        assert_ne!(AuthMethod::ScramSha256Standard, AuthMethod::ScramSha256GaussDb);
    }

    #[test]
    fn test_server_type_equality() {
        // 测试服务器类型的相等性比较
        assert_eq!(ServerType::GaussDB, ServerType::GaussDB);
        assert_ne!(ServerType::GaussDB, ServerType::PostgreSQL);
        assert_ne!(ServerType::PostgreSQL, ServerType::Unknown);
    }

    #[test]
    fn test_adaptive_auth_manager_creation() {
        // 测试自适应认证管理器的创建
        let manager = AdaptiveAuthManager::new();
        let default_manager = AdaptiveAuthManager::default();

        // 验证默认偏好设置
        assert!(!manager.auth_preferences.is_empty());
        assert!(!default_manager.auth_preferences.is_empty());

        // 验证初始统计
        assert_eq!(manager.get_stats().successful_auths, 0);
        assert_eq!(default_manager.get_stats().successful_auths, 0);
    }

    #[test]
    fn test_multiple_server_detections() {
        // 测试多次服务器检测
        let mut manager = AdaptiveAuthManager::new();

        let test_cases = vec![
            ("openGauss 2.1.0", ServerType::GaussDB),
            ("openGauss 3.0.0 build abc123", ServerType::GaussDB),
            ("GaussDB Kernel V500R002C00", ServerType::GaussDB),
            ("PostgreSQL 13.7", ServerType::PostgreSQL),
            ("PostgreSQL 14.5 on x86_64-pc-linux-gnu", ServerType::PostgreSQL),
            ("MySQL 8.0.30", ServerType::Unknown),
            ("", ServerType::Unknown),
        ];

        for (version_string, expected_type) in test_cases {
            let detected_type = manager.detect_server_compatibility(Some(version_string));
            assert_eq!(detected_type, expected_type, "版本字符串: '{}'", version_string);
        }
    }

    #[test]
    fn test_auth_strategy_debug() {
        // 测试认证策略的调试输出
        let strategy = AuthStrategy::ScramSha256 {
            compatibility_mode: CompatibilityMode::Auto,
            use_plus: false,
        };

        let debug_str = format!("{:?}", strategy);
        assert!(debug_str.contains("ScramSha256"));
        assert!(debug_str.contains("Auto"));
        assert!(debug_str.contains("false"));
    }

    #[test]
    fn test_create_gaussdb_scram() {
        // 测试 GaussDB SCRAM 创建函数
        let password = b"test_password";
        let channel_binding = ChannelBinding::unsupported();
        let compatibility_mode = CompatibilityMode::GaussDb;

        let scram = create_gaussdb_scram(password, channel_binding, compatibility_mode);
        let message = scram.message();

        assert!(!message.is_empty());
        assert!(std::str::from_utf8(message).is_ok());
    }

    #[test]
    fn test_auth_method_hash() {
        // 测试认证方法可以用作 HashMap 键
        use std::collections::HashMap;

        let mut method_counts = HashMap::new();
        method_counts.insert(AuthMethod::Sha256, 5);
        method_counts.insert(AuthMethod::Md5, 3);
        method_counts.insert(AuthMethod::ScramSha256GaussDb, 2);

        assert_eq!(method_counts.get(&AuthMethod::Sha256), Some(&5));
        assert_eq!(method_counts.get(&AuthMethod::Md5), Some(&3));
        assert_eq!(method_counts.get(&AuthMethod::Cleartext), None);
    }

    #[test]
    fn test_compatibility_mode_variants() {
        // 测试兼容模式的所有变体
        let modes = vec![
            CompatibilityMode::Standard,
            CompatibilityMode::GaussDb,
            CompatibilityMode::Auto,
        ];

        for mode in modes {
            let debug_str = format!("{:?}", mode);
            assert!(!debug_str.is_empty());
        }
    }
}
