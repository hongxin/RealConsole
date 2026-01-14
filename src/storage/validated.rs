//! ValidatedStorage - 数据验证存储层
//!
//! v1.79.0: 提供写入前数据验证
//!
//! ## 功能特性
//!
//! - **键验证**: 模式匹配、长度限制
//! - **值验证**: 大小限制、格式检查
//! - **自定义验证器**: 灵活的验证规则
//! - **验证链**: 多个验证器组合
//! - **详细错误**: 精确的验证失败信息
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{ValidatedStorage, MemoryStorage};
//!
//! let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
//!     .key_pattern(r"^[a-z0-9:_-]+$")
//!     .max_key_length(256)
//!     .max_value_size(1024 * 1024)  // 1 MB
//!     .build();
//!
//! // 有效键
//! storage.write("user:123", b"data").await?;
//!
//! // 无效键（包含大写字母）
//! let result = storage.write("User:123", b"data").await;
//! assert!(result.is_err());
//! ```

use super::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use regex::Regex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// Type alias for callback types to satisfy clippy::type_complexity
type WarnCallback = Arc<dyn Fn(&str, &ValidationError) + Send + Sync>;

// ============================================================================
// 验证错误
// ============================================================================

/// 验证错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// 键太长
    KeyTooLong { length: usize, max: usize },
    /// 键太短
    KeyTooShort { length: usize, min: usize },
    /// 键不匹配模式
    KeyPatternMismatch { key: String, pattern: String },
    /// 键包含非法字符
    KeyInvalidChars { key: String, invalid: Vec<char> },
    /// 值太大
    ValueTooLarge { size: usize, max: usize },
    /// 值太小
    ValueTooSmall { size: usize, min: usize },
    /// 值不是有效的 UTF-8
    ValueNotUtf8,
    /// 值不是有效的 JSON
    ValueNotJson { error: String },
    /// 自定义验证失败
    CustomValidation { message: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::KeyTooLong { length, max } => {
                write!(f, "Key too long: {} > {}", length, max)
            }
            ValidationError::KeyTooShort { length, min } => {
                write!(f, "Key too short: {} < {}", length, min)
            }
            ValidationError::KeyPatternMismatch { key, pattern } => {
                write!(f, "Key '{}' doesn't match pattern '{}'", key, pattern)
            }
            ValidationError::KeyInvalidChars { key, invalid } => {
                write!(f, "Key '{}' contains invalid chars: {:?}", key, invalid)
            }
            ValidationError::ValueTooLarge { size, max } => {
                write!(f, "Value too large: {} > {}", size, max)
            }
            ValidationError::ValueTooSmall { size, min } => {
                write!(f, "Value too small: {} < {}", size, min)
            }
            ValidationError::ValueNotUtf8 => write!(f, "Value is not valid UTF-8"),
            ValidationError::ValueNotJson { error } => {
                write!(f, "Value is not valid JSON: {}", error)
            }
            ValidationError::CustomValidation { message } => {
                write!(f, "Validation failed: {}", message)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

// ============================================================================
// 验证器 Trait
// ============================================================================

/// 键验证器
pub trait KeyValidator: Send + Sync {
    /// 验证键
    fn validate(&self, key: &str) -> Result<(), ValidationError>;

    /// 验证器名称
    fn name(&self) -> &'static str;
}

/// 值验证器
pub trait ValueValidator: Send + Sync {
    /// 验证值
    fn validate(&self, key: &str, value: &[u8]) -> Result<(), ValidationError>;

    /// 验证器名称
    fn name(&self) -> &'static str;
}

// ============================================================================
// 内置键验证器
// ============================================================================

/// 键长度验证器
pub struct KeyLengthValidator {
    min: Option<usize>,
    max: Option<usize>,
}

impl KeyLengthValidator {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self { min, max }
    }

    pub fn max(max: usize) -> Self {
        Self {
            min: None,
            max: Some(max),
        }
    }

    pub fn min(min: usize) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    pub fn range(min: usize, max: usize) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl KeyValidator for KeyLengthValidator {
    fn validate(&self, key: &str) -> Result<(), ValidationError> {
        let len = key.len();

        if let Some(min) = self.min {
            if len < min {
                return Err(ValidationError::KeyTooShort { length: len, min });
            }
        }

        if let Some(max) = self.max {
            if len > max {
                return Err(ValidationError::KeyTooLong { length: len, max });
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "KeyLengthValidator"
    }
}

/// 键模式验证器
pub struct KeyPatternValidator {
    pattern: Regex,
    pattern_str: String,
}

impl KeyPatternValidator {
    pub fn new(pattern: &str) -> Result<Self, regex::Error> {
        Ok(Self {
            pattern: Regex::new(pattern)?,
            pattern_str: pattern.to_string(),
        })
    }
}

impl KeyValidator for KeyPatternValidator {
    fn validate(&self, key: &str) -> Result<(), ValidationError> {
        if !self.pattern.is_match(key) {
            return Err(ValidationError::KeyPatternMismatch {
                key: key.to_string(),
                pattern: self.pattern_str.clone(),
            });
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "KeyPatternValidator"
    }
}

/// 禁止字符验证器
pub struct KeyForbiddenCharsValidator {
    forbidden: Vec<char>,
}

impl KeyForbiddenCharsValidator {
    pub fn new(forbidden: Vec<char>) -> Self {
        Self { forbidden }
    }

    pub fn default_forbidden() -> Self {
        Self::new(vec!['/', '\\', '\0', '\n', '\r', '\t'])
    }
}

impl KeyValidator for KeyForbiddenCharsValidator {
    fn validate(&self, key: &str) -> Result<(), ValidationError> {
        let invalid: Vec<char> = key.chars().filter(|c| self.forbidden.contains(c)).collect();

        if !invalid.is_empty() {
            return Err(ValidationError::KeyInvalidChars {
                key: key.to_string(),
                invalid,
            });
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "KeyForbiddenCharsValidator"
    }
}

// ============================================================================
// 内置值验证器
// ============================================================================

/// 值大小验证器
pub struct ValueSizeValidator {
    min: Option<usize>,
    max: Option<usize>,
}

impl ValueSizeValidator {
    pub fn new(min: Option<usize>, max: Option<usize>) -> Self {
        Self { min, max }
    }

    pub fn max(max: usize) -> Self {
        Self {
            min: None,
            max: Some(max),
        }
    }

    pub fn min(min: usize) -> Self {
        Self {
            min: Some(min),
            max: None,
        }
    }

    pub fn range(min: usize, max: usize) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
        }
    }
}

impl ValueValidator for ValueSizeValidator {
    fn validate(&self, _key: &str, value: &[u8]) -> Result<(), ValidationError> {
        let size = value.len();

        if let Some(min) = self.min {
            if size < min {
                return Err(ValidationError::ValueTooSmall { size, min });
            }
        }

        if let Some(max) = self.max {
            if size > max {
                return Err(ValidationError::ValueTooLarge { size, max });
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "ValueSizeValidator"
    }
}

/// UTF-8 验证器
pub struct Utf8Validator;

impl ValueValidator for Utf8Validator {
    fn validate(&self, _key: &str, value: &[u8]) -> Result<(), ValidationError> {
        std::str::from_utf8(value).map_err(|_| ValidationError::ValueNotUtf8)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "Utf8Validator"
    }
}

/// JSON 验证器
pub struct JsonValidator;

impl ValueValidator for JsonValidator {
    fn validate(&self, _key: &str, value: &[u8]) -> Result<(), ValidationError> {
        serde_json::from_slice::<serde_json::Value>(value).map_err(|e| {
            ValidationError::ValueNotJson {
                error: e.to_string(),
            }
        })?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "JsonValidator"
    }
}

/// 自定义函数验证器
pub struct CustomValidator<F>
where
    F: Fn(&str, &[u8]) -> Result<(), String> + Send + Sync,
{
    validator: F,
    name: &'static str,
}

impl<F> CustomValidator<F>
where
    F: Fn(&str, &[u8]) -> Result<(), String> + Send + Sync,
{
    pub fn new(name: &'static str, validator: F) -> Self {
        Self { validator, name }
    }
}

impl<F> ValueValidator for CustomValidator<F>
where
    F: Fn(&str, &[u8]) -> Result<(), String> + Send + Sync,
{
    fn validate(&self, key: &str, value: &[u8]) -> Result<(), ValidationError> {
        (self.validator)(key, value).map_err(|message| ValidationError::CustomValidation { message })
    }

    fn name(&self) -> &'static str {
        self.name
    }
}

// ============================================================================
// 配置
// ============================================================================

/// 验证配置
#[derive(Debug, Clone, Copy, Default)]
pub struct ValidationConfig {
    /// 验证失败时是否允许写入（仅警告）
    pub warn_only: bool,
}

// ============================================================================
// 统计信息
// ============================================================================

/// 验证统计
#[derive(Debug, Default)]
pub struct ValidationStats {
    /// 验证通过次数
    passed: AtomicU64,
    /// 验证失败次数
    failed: AtomicU64,
    /// 键验证失败次数
    key_failures: AtomicU64,
    /// 值验证失败次数
    value_failures: AtomicU64,
    /// 警告次数（warn_only 模式）
    warnings: AtomicU64,
}

/// 统计快照
#[derive(Debug, Clone)]
pub struct ValidationStatsSnapshot {
    pub passed: u64,
    pub failed: u64,
    pub key_failures: u64,
    pub value_failures: u64,
    pub warnings: u64,
}

impl ValidationStatsSnapshot {
    /// 通过率
    pub fn pass_rate(&self) -> f64 {
        let total = self.passed + self.failed;
        if total == 0 {
            1.0
        } else {
            self.passed as f64 / total as f64
        }
    }
}

/// 详细统计
#[derive(Debug, Clone)]
pub struct DetailedValidationStats {
    /// 快照统计
    pub snapshot: ValidationStatsSnapshot,
    /// 底层存储统计
    pub backend_stats: StorageStats,
    /// 键验证器数量
    pub key_validator_count: usize,
    /// 值验证器数量
    pub value_validator_count: usize,
}

// ============================================================================
// ValidatedStorage 实现
// ============================================================================

/// 验证存储层
///
/// 装饰器模式，包装底层存储并在写入前验证数据
pub struct ValidatedStorage<B: StorageBackend> {
    /// 底层存储
    backend: Arc<B>,
    /// 配置
    config: ValidationConfig,
    /// 键验证器
    key_validators: Vec<Box<dyn KeyValidator>>,
    /// 值验证器
    value_validators: Vec<Box<dyn ValueValidator>>,
    /// 统计信息
    stats: Arc<ValidationStats>,
    /// 警告回调
    warn_callback: Option<WarnCallback>,
}

impl<B: StorageBackend> ValidatedStorage<B> {
    /// 创建新的 ValidatedStorage
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            config: ValidationConfig::default(),
            key_validators: Vec::new(),
            value_validators: Vec::new(),
            stats: Arc::new(ValidationStats::default()),
            warn_callback: None,
        }
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self {
            backend,
            config: ValidationConfig::default(),
            key_validators: Vec::new(),
            value_validators: Vec::new(),
            stats: Arc::new(ValidationStats::default()),
            warn_callback: None,
        }
    }

    /// 添加键验证器
    pub fn add_key_validator<V: KeyValidator + 'static>(mut self, validator: V) -> Self {
        self.key_validators.push(Box::new(validator));
        self
    }

    /// 添加值验证器
    pub fn add_value_validator<V: ValueValidator + 'static>(mut self, validator: V) -> Self {
        self.value_validators.push(Box::new(validator));
        self
    }

    /// 设置仅警告模式
    pub fn warn_only(mut self, enabled: bool) -> Self {
        self.config.warn_only = enabled;
        self
    }

    /// 设置警告回调
    pub fn with_warn_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &ValidationError) + Send + Sync + 'static,
    {
        self.warn_callback = Some(Arc::new(callback));
        self
    }

    /// 验证键
    fn validate_key(&self, key: &str) -> Result<(), ValidationError> {
        for validator in &self.key_validators {
            validator.validate(key)?;
        }
        Ok(())
    }

    /// 验证值
    fn validate_value(&self, key: &str, value: &[u8]) -> Result<(), ValidationError> {
        for validator in &self.value_validators {
            validator.validate(key, value)?;
        }
        Ok(())
    }

    /// 获取统计快照
    pub fn stats_snapshot(&self) -> ValidationStatsSnapshot {
        ValidationStatsSnapshot {
            passed: self.stats.passed.load(Ordering::SeqCst),
            failed: self.stats.failed.load(Ordering::SeqCst),
            key_failures: self.stats.key_failures.load(Ordering::SeqCst),
            value_failures: self.stats.value_failures.load(Ordering::SeqCst),
            warnings: self.stats.warnings.load(Ordering::SeqCst),
        }
    }

    /// 获取详细统计
    pub fn detailed_stats(&self) -> DetailedValidationStats {
        DetailedValidationStats {
            snapshot: self.stats_snapshot(),
            backend_stats: self.backend.stats(),
            key_validator_count: self.key_validators.len(),
            value_validator_count: self.value_validators.len(),
        }
    }
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend> StorageBackend for ValidatedStorage<B> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        self.backend.read(key).await
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        // 验证键
        if let Err(e) = self.validate_key(key) {
            self.stats.key_failures.fetch_add(1, Ordering::SeqCst);

            if self.config.warn_only {
                self.stats.warnings.fetch_add(1, Ordering::SeqCst);
                if let Some(ref callback) = self.warn_callback {
                    callback(key, &e);
                }
            } else {
                self.stats.failed.fetch_add(1, Ordering::SeqCst);
                return Err(StorageError::Other(e.to_string()));
            }
        }

        // 验证值
        if let Err(e) = self.validate_value(key, data) {
            self.stats.value_failures.fetch_add(1, Ordering::SeqCst);

            if self.config.warn_only {
                self.stats.warnings.fetch_add(1, Ordering::SeqCst);
                if let Some(ref callback) = self.warn_callback {
                    callback(key, &e);
                }
            } else {
                self.stats.failed.fetch_add(1, Ordering::SeqCst);
                return Err(StorageError::Other(e.to_string()));
            }
        }

        // 执行写入
        self.backend.write(key, data).await?;
        self.stats.passed.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        self.backend.delete(key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        self.backend.list(prefix).await
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        self.backend.exists(key).await
    }

    fn stats(&self) -> StorageStats {
        self.backend.stats()
    }

    fn name(&self) -> &'static str {
        "ValidatedStorage"
    }
}

// ============================================================================
// Builder
// ============================================================================

/// ValidatedStorage 构建器
pub struct ValidatedStorageBuilder<B: StorageBackend> {
    backend: Arc<B>,
    key_validators: Vec<Box<dyn KeyValidator>>,
    value_validators: Vec<Box<dyn ValueValidator>>,
    config: ValidationConfig,
    warn_callback: Option<WarnCallback>,
}

impl<B: StorageBackend> ValidatedStorageBuilder<B> {
    /// 创建构建器
    pub fn new(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            key_validators: Vec::new(),
            value_validators: Vec::new(),
            config: ValidationConfig::default(),
            warn_callback: None,
        }
    }

    /// 从 Arc 创建
    pub fn from_arc(backend: Arc<B>) -> Self {
        Self {
            backend,
            key_validators: Vec::new(),
            value_validators: Vec::new(),
            config: ValidationConfig::default(),
            warn_callback: None,
        }
    }

    /// 添加键模式验证
    pub fn key_pattern(mut self, pattern: &str) -> Self {
        if let Ok(v) = KeyPatternValidator::new(pattern) {
            self.key_validators.push(Box::new(v));
        }
        self
    }

    /// 添加最大键长度验证
    pub fn max_key_length(mut self, max: usize) -> Self {
        self.key_validators.push(Box::new(KeyLengthValidator::max(max)));
        self
    }

    /// 添加最小键长度验证
    pub fn min_key_length(mut self, min: usize) -> Self {
        self.key_validators.push(Box::new(KeyLengthValidator::min(min)));
        self
    }

    /// 添加禁止字符验证
    pub fn forbidden_key_chars(mut self, chars: Vec<char>) -> Self {
        self.key_validators
            .push(Box::new(KeyForbiddenCharsValidator::new(chars)));
        self
    }

    /// 添加默认禁止字符验证
    pub fn default_forbidden_chars(mut self) -> Self {
        self.key_validators
            .push(Box::new(KeyForbiddenCharsValidator::default_forbidden()));
        self
    }

    /// 添加最大值大小验证
    pub fn max_value_size(mut self, max: usize) -> Self {
        self.value_validators
            .push(Box::new(ValueSizeValidator::max(max)));
        self
    }

    /// 添加最小值大小验证
    pub fn min_value_size(mut self, min: usize) -> Self {
        self.value_validators
            .push(Box::new(ValueSizeValidator::min(min)));
        self
    }

    /// 要求值为有效 UTF-8
    pub fn require_utf8(mut self) -> Self {
        self.value_validators.push(Box::new(Utf8Validator));
        self
    }

    /// 要求值为有效 JSON
    pub fn require_json(mut self) -> Self {
        self.value_validators.push(Box::new(JsonValidator));
        self
    }

    /// 添加自定义键验证器
    pub fn add_key_validator<V: KeyValidator + 'static>(mut self, validator: V) -> Self {
        self.key_validators.push(Box::new(validator));
        self
    }

    /// 添加自定义值验证器
    pub fn add_value_validator<V: ValueValidator + 'static>(mut self, validator: V) -> Self {
        self.value_validators.push(Box::new(validator));
        self
    }

    /// 设置仅警告模式
    pub fn warn_only(mut self, enabled: bool) -> Self {
        self.config.warn_only = enabled;
        self
    }

    /// 设置警告回调
    pub fn warn_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &ValidationError) + Send + Sync + 'static,
    {
        self.warn_callback = Some(Arc::new(callback));
        self
    }

    /// 构建
    pub fn build(self) -> ValidatedStorage<B> {
        ValidatedStorage {
            backend: self.backend,
            config: self.config,
            key_validators: self.key_validators,
            value_validators: self.value_validators,
            stats: Arc::new(ValidationStats::default()),
            warn_callback: self.warn_callback,
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[tokio::test]
    async fn test_validated_storage_basic() {
        let storage = ValidatedStorage::new(MemoryStorage::new());

        storage.write("key1", b"value1").await.unwrap();
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_key_length_max() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .max_key_length(10)
            .build();

        storage.write("short", b"v").await.unwrap();

        let result = storage.write("this_is_a_very_long_key", b"v").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Key too long"));
    }

    #[tokio::test]
    async fn test_key_length_min() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .min_key_length(5)
            .build();

        storage.write("valid", b"v").await.unwrap();

        let result = storage.write("ab", b"v").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Key too short"));
    }

    #[tokio::test]
    async fn test_key_pattern() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .key_pattern(r"^[a-z0-9_]+$")
            .build();

        storage.write("valid_key_123", b"v").await.unwrap();

        let result = storage.write("Invalid-Key", b"v").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("doesn't match"));
    }

    #[tokio::test]
    async fn test_forbidden_chars() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .forbidden_key_chars(vec!['/', '\\'])
            .build();

        storage.write("valid_key", b"v").await.unwrap();

        let result = storage.write("path/to/key", b"v").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid chars"));
    }

    #[tokio::test]
    async fn test_value_size_max() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .max_value_size(10)
            .build();

        storage.write("key1", b"short").await.unwrap();

        let result = storage.write("key2", b"this is way too long").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Value too large"));
    }

    #[tokio::test]
    async fn test_value_size_min() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .min_value_size(5)
            .build();

        storage.write("key1", b"valid").await.unwrap();

        let result = storage.write("key2", b"ab").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Value too small"));
    }

    #[tokio::test]
    async fn test_require_utf8() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .require_utf8()
            .build();

        storage.write("key1", b"valid utf8").await.unwrap();

        // Invalid UTF-8 bytes
        let result = storage.write("key2", &[0xff, 0xfe]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("UTF-8"));
    }

    #[tokio::test]
    async fn test_require_json() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .require_json()
            .build();

        storage
            .write("key1", b"{\"name\": \"test\"}")
            .await
            .unwrap();

        let result = storage.write("key2", b"not json").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("JSON"));
    }

    #[tokio::test]
    async fn test_multiple_validators() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .max_key_length(20)
            .key_pattern(r"^[a-z]+$")
            .max_value_size(100)
            .build();

        storage.write("valid", b"data").await.unwrap();

        // Fails key length
        let r1 = storage
            .write("verylongkeythatexceedslimit", b"v")
            .await;
        assert!(r1.is_err());

        // Fails pattern
        let r2 = storage.write("UPPER", b"v").await;
        assert!(r2.is_err());
    }

    #[tokio::test]
    async fn test_warn_only_mode() {
        use std::sync::Mutex;

        let warnings = Arc::new(Mutex::new(Vec::new()));
        let warnings_clone = Arc::clone(&warnings);

        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .max_key_length(5)
            .warn_only(true)
            .warn_callback(move |key, err| {
                warnings_clone
                    .lock()
                    .unwrap()
                    .push((key.to_string(), err.clone()));
            })
            .build();

        // Should succeed despite validation failure
        storage.write("very_long_key", b"v").await.unwrap();

        let warns = warnings.lock().unwrap();
        assert_eq!(warns.len(), 1);
        assert!(matches!(warns[0].1, ValidationError::KeyTooLong { .. }));
    }

    #[tokio::test]
    async fn test_stats_tracking() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .max_key_length(10)
            .build();

        storage.write("short", b"v").await.unwrap();
        let _ = storage.write("very_long_key", b"v").await;

        let stats = storage.stats_snapshot();
        assert_eq!(stats.passed, 1);
        assert_eq!(stats.failed, 1);
        assert_eq!(stats.key_failures, 1);
    }

    #[tokio::test]
    async fn test_pass_rate() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .max_key_length(10)
            .build();

        storage.write("ok1", b"v").await.unwrap();
        storage.write("ok2", b"v").await.unwrap();
        let _ = storage.write("very_long_key", b"v").await;

        let stats = storage.stats_snapshot();
        // 2 passed, 1 failed = 66.7%
        assert!((stats.pass_rate() - 0.667).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_validation_error_display() {
        let e1 = ValidationError::KeyTooLong {
            length: 100,
            max: 50,
        };
        assert!(e1.to_string().contains("Key too long"));

        let e2 = ValidationError::ValueNotJson {
            error: "parse error".to_string(),
        };
        assert!(e2.to_string().contains("JSON"));

        let e3 = ValidationError::CustomValidation {
            message: "custom error".to_string(),
        };
        assert!(e3.to_string().contains("custom error"));
    }

    #[tokio::test]
    async fn test_custom_validator() {
        let validator = CustomValidator::new("EvenLengthValidator", |_key, value| {
            if value.len() % 2 == 0 {
                Ok(())
            } else {
                Err("Value length must be even".to_string())
            }
        });

        let storage = ValidatedStorage::new(MemoryStorage::new()).add_value_validator(validator);

        storage.write("key1", b"ab").await.unwrap(); // even
        let result = storage.write("key2", b"abc").await; // odd
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("even"));
    }

    #[tokio::test]
    async fn test_from_arc() {
        let backend = Arc::new(MemoryStorage::new());
        let storage = ValidatedStorage::from_arc(backend);

        storage.write("key1", b"value1").await.unwrap();
        let data = storage.read("key1").await.unwrap();
        assert_eq!(data, b"value1");
    }

    #[tokio::test]
    async fn test_detailed_stats() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .max_key_length(100)
            .max_value_size(1000)
            .require_utf8()
            .build();

        let detailed = storage.detailed_stats();
        assert_eq!(detailed.key_validator_count, 1);
        assert_eq!(detailed.value_validator_count, 2);
    }

    #[tokio::test]
    async fn test_delete_and_read_bypass_validation() {
        let storage = ValidatedStorageBuilder::new(MemoryStorage::new())
            .max_key_length(5)
            .build();

        // Write with valid key
        storage.write("key", b"value").await.unwrap();

        // Delete should work (no validation)
        storage.delete("key").await.unwrap();

        // Read should work (no validation)
        let result = storage.read("nonexistent").await;
        assert!(result.is_err()); // NotFound, not validation error
    }
}
