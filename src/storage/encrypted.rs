//! 加密存储层
//!
//! v1.71.0: 提供透明的数据加密/解密支持
//!
//! ## 设计目标
//!
//! - **透明加密**: 对上层应用透明，无需修改业务代码
//! - **可插拔算法**: 支持不同的加密算法实现
//! - **密钥管理**: 支持密钥轮换和多密钥
//!
//! ## 注意事项
//!
//! 本实现使用简化的加密算法用于演示目的。
//! 生产环境请使用专业的加密库（如 ring, aes-gcm）。
//!
//! ## 使用示例
//!
//! ```ignore
//! use realconsole::storage::{EncryptedStorage, MemoryStorage, XorCipher};
//!
//! let storage = MemoryStorage::new();
//! let cipher = XorCipher::new(b"secret-key-32-bytes-long!!!!!!!!");
//! let encrypted = EncryptedStorage::new(storage, cipher);
//!
//! // 数据自动加密存储
//! encrypted.write("key1", b"sensitive data").await?;
//!
//! // 读取时自动解密
//! let data = encrypted.read("key1").await?;
//! assert_eq!(data, b"sensitive data");
//! ```

use crate::storage::{StorageBackend, StorageError, StorageResult, StorageStats};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// ============================================================================
// 加密器 Trait
// ============================================================================

/// 加密器接口
///
/// 定义加密和解密操作的标准接口
pub trait Cipher: Send + Sync {
    /// 加密数据
    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8>;

    /// 解密数据
    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CipherError>;

    /// 获取加密器名称
    fn name(&self) -> &'static str;

    /// 获取密钥 ID（用于密钥轮换）
    fn key_id(&self) -> &str {
        "default"
    }
}

/// 加密错误
#[derive(Debug, Clone)]
pub enum CipherError {
    /// 解密失败
    DecryptionFailed(String),
    /// 无效的密文格式
    InvalidFormat(String),
    /// 密钥不匹配
    KeyMismatch(String),
}

impl std::fmt::Display for CipherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CipherError::DecryptionFailed(msg) => write!(f, "Decryption failed: {}", msg),
            CipherError::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            CipherError::KeyMismatch(msg) => write!(f, "Key mismatch: {}", msg),
        }
    }
}

impl std::error::Error for CipherError {}

// ============================================================================
// XOR 加密器（演示用）
// ============================================================================

/// XOR 加密器
///
/// 简单的 XOR 加密实现，仅用于演示目的。
/// 生产环境请使用 AES-GCM 或 ChaCha20-Poly1305。
#[derive(Clone)]
pub struct XorCipher {
    /// 密钥
    key: Vec<u8>,
    /// 密钥 ID
    key_id: String,
}

impl XorCipher {
    /// 创建 XOR 加密器
    pub fn new(key: &[u8]) -> Self {
        Self {
            key: key.to_vec(),
            key_id: "xor-default".to_string(),
        }
    }

    /// 使用指定密钥 ID 创建
    pub fn with_key_id(key: &[u8], key_id: impl Into<String>) -> Self {
        Self {
            key: key.to_vec(),
            key_id: key_id.into(),
        }
    }
}

impl Cipher for XorCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        // 添加魔数头用于验证
        let mut result = vec![0xE7, 0xC7]; // magic bytes

        // 添加密钥 ID 长度和内容
        let key_id_bytes = self.key_id.as_bytes();
        result.push(key_id_bytes.len() as u8);
        result.extend_from_slice(key_id_bytes);

        // 添加原始长度（4字节）
        let len = plaintext.len() as u32;
        result.extend_from_slice(&len.to_le_bytes());

        // XOR 加密
        for (i, byte) in plaintext.iter().enumerate() {
            result.push(byte ^ self.key[i % self.key.len()]);
        }

        result
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CipherError> {
        // 检查最小长度
        if ciphertext.len() < 7 {
            return Err(CipherError::InvalidFormat("Ciphertext too short".to_string()));
        }

        // 验证魔数
        if ciphertext[0] != 0xE7 || ciphertext[1] != 0xC7 {
            return Err(CipherError::InvalidFormat("Invalid magic bytes".to_string()));
        }

        // 读取密钥 ID
        let key_id_len = ciphertext[2] as usize;
        if ciphertext.len() < 3 + key_id_len + 4 {
            return Err(CipherError::InvalidFormat("Invalid key ID length".to_string()));
        }

        let stored_key_id = String::from_utf8_lossy(&ciphertext[3..3 + key_id_len]);
        if stored_key_id != self.key_id {
            return Err(CipherError::KeyMismatch(format!(
                "Expected key '{}', got '{}'",
                self.key_id, stored_key_id
            )));
        }

        // 读取原始长度
        let len_start = 3 + key_id_len;
        let len_bytes: [u8; 4] = ciphertext[len_start..len_start + 4]
            .try_into()
            .map_err(|_| CipherError::InvalidFormat("Invalid length bytes".to_string()))?;
        let original_len = u32::from_le_bytes(len_bytes) as usize;

        // XOR 解密
        let data_start = len_start + 4;
        let encrypted_data = &ciphertext[data_start..];

        if encrypted_data.len() < original_len {
            return Err(CipherError::InvalidFormat("Data truncated".to_string()));
        }

        let mut result = Vec::with_capacity(original_len);
        for (i, byte) in encrypted_data[..original_len].iter().enumerate() {
            result.push(byte ^ self.key[i % self.key.len()]);
        }

        Ok(result)
    }

    fn name(&self) -> &'static str {
        "XorCipher"
    }

    fn key_id(&self) -> &str {
        &self.key_id
    }
}

// ============================================================================
// 空加密器（透传）
// ============================================================================

/// 空加密器
///
/// 不进行任何加密，直接透传数据。用于测试和调试。
#[derive(Clone, Default)]
pub struct NullCipher;

impl NullCipher {
    /// 创建空加密器
    pub fn new() -> Self {
        Self
    }
}

impl Cipher for NullCipher {
    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        plaintext.to_vec()
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CipherError> {
        Ok(ciphertext.to_vec())
    }

    fn name(&self) -> &'static str {
        "NullCipher"
    }
}

// ============================================================================
// Base64 编码加密器（混淆用）
// ============================================================================

/// Base64 编码器
///
/// 使用 Base64 编码数据。不是真正的加密，仅用于简单混淆。
#[derive(Clone, Default)]
pub struct Base64Cipher;

impl Base64Cipher {
    /// 创建 Base64 编码器
    pub fn new() -> Self {
        Self
    }
}

impl Cipher for Base64Cipher {
    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .encode(plaintext)
            .into_bytes()
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CipherError> {
        use base64::Engine;
        let text = String::from_utf8_lossy(ciphertext);
        base64::engine::general_purpose::STANDARD
            .decode(text.as_ref())
            .map_err(|e| CipherError::DecryptionFailed(e.to_string()))
    }

    fn name(&self) -> &'static str {
        "Base64Cipher"
    }
}

// ============================================================================
// 加密统计
// ============================================================================

/// 加密统计信息
#[derive(Debug, Default)]
pub struct EncryptionStats {
    /// 加密次数
    pub encryptions: AtomicU64,
    /// 解密次数
    pub decryptions: AtomicU64,
    /// 加密字节数
    pub bytes_encrypted: AtomicU64,
    /// 解密字节数
    pub bytes_decrypted: AtomicU64,
    /// 解密失败次数
    pub decryption_failures: AtomicU64,
}

impl EncryptionStats {
    /// 获取快照
    pub fn snapshot(&self) -> EncryptionStatsSnapshot {
        EncryptionStatsSnapshot {
            encryptions: self.encryptions.load(Ordering::Relaxed),
            decryptions: self.decryptions.load(Ordering::Relaxed),
            bytes_encrypted: self.bytes_encrypted.load(Ordering::Relaxed),
            bytes_decrypted: self.bytes_decrypted.load(Ordering::Relaxed),
            decryption_failures: self.decryption_failures.load(Ordering::Relaxed),
        }
    }
}

/// 加密统计快照
#[derive(Debug, Clone)]
pub struct EncryptionStatsSnapshot {
    /// 加密次数
    pub encryptions: u64,
    /// 解密次数
    pub decryptions: u64,
    /// 加密字节数
    pub bytes_encrypted: u64,
    /// 解密字节数
    pub bytes_decrypted: u64,
    /// 解密失败次数
    pub decryption_failures: u64,
}

impl EncryptionStatsSnapshot {
    /// 加密开销比例
    pub fn encryption_overhead(&self) -> f64 {
        if self.bytes_decrypted == 0 {
            0.0
        } else {
            (self.bytes_encrypted as f64 - self.bytes_decrypted as f64) / self.bytes_decrypted as f64
        }
    }

    /// 解密成功率
    pub fn decryption_success_rate(&self) -> f64 {
        let total = self.decryptions + self.decryption_failures;
        if total == 0 {
            1.0
        } else {
            self.decryptions as f64 / total as f64
        }
    }
}

// ============================================================================
// 加密存储配置
// ============================================================================

/// 加密存储配置
#[derive(Debug, Clone)]
pub struct EncryptedStorageConfig {
    /// 是否加密键名
    pub encrypt_keys: bool,
    /// 加密失败时是否返回错误（否则返回原始数据）
    pub fail_on_decrypt_error: bool,
}

impl Default for EncryptedStorageConfig {
    fn default() -> Self {
        Self {
            encrypt_keys: false,
            fail_on_decrypt_error: true,
        }
    }
}

// ============================================================================
// 加密存储
// ============================================================================

/// 加密存储
///
/// 透明地加密和解密存储的数据
pub struct EncryptedStorage<B: StorageBackend, C: Cipher> {
    /// 后端存储
    backend: Arc<B>,
    /// 加密器
    cipher: Arc<C>,
    /// 配置
    config: EncryptedStorageConfig,
    /// 统计信息
    stats: Arc<EncryptionStats>,
}

impl<B: StorageBackend, C: Cipher> EncryptedStorage<B, C> {
    /// 创建加密存储
    pub fn new(backend: B, cipher: C) -> Self {
        Self::with_config(backend, cipher, EncryptedStorageConfig::default())
    }

    /// 使用配置创建加密存储
    pub fn with_config(backend: B, cipher: C, config: EncryptedStorageConfig) -> Self {
        Self {
            backend: Arc::new(backend),
            cipher: Arc::new(cipher),
            config,
            stats: Arc::new(EncryptionStats::default()),
        }
    }

    /// 从 Arc 创建加密存储
    pub fn from_arc(backend: Arc<B>, cipher: C) -> Self {
        Self::from_arc_with_config(backend, cipher, EncryptedStorageConfig::default())
    }

    /// 从 Arc 使用配置创建加密存储
    pub fn from_arc_with_config(backend: Arc<B>, cipher: C, config: EncryptedStorageConfig) -> Self {
        Self {
            backend,
            cipher: Arc::new(cipher),
            config,
            stats: Arc::new(EncryptionStats::default()),
        }
    }

    /// 获取加密器名称
    pub fn cipher_name(&self) -> &'static str {
        self.cipher.name()
    }

    /// 获取密钥 ID
    pub fn key_id(&self) -> &str {
        self.cipher.key_id()
    }

    /// 获取统计信息快照
    pub fn stats_snapshot(&self) -> EncryptionStatsSnapshot {
        self.stats.snapshot()
    }

    /// 获取详细统计信息
    pub fn detailed_stats(&self) -> DetailedEncryptionStats {
        let snapshot = self.stats.snapshot();
        DetailedEncryptionStats {
            encryptions: snapshot.encryptions,
            decryptions: snapshot.decryptions,
            bytes_encrypted: snapshot.bytes_encrypted,
            bytes_decrypted: snapshot.bytes_decrypted,
            decryption_failures: snapshot.decryption_failures,
            encryption_overhead: snapshot.encryption_overhead(),
            decryption_success_rate: snapshot.decryption_success_rate(),
            cipher_name: self.cipher.name().to_string(),
            key_id: self.cipher.key_id().to_string(),
            encrypt_keys: self.config.encrypt_keys,
        }
    }

    /// 转换键名（如果配置了加密键名）
    fn transform_key(&self, key: &str) -> String {
        if self.config.encrypt_keys {
            // 使用 Base64 编码加密后的键名
            use base64::Engine;
            let encrypted = self.cipher.encrypt(key.as_bytes());
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&encrypted)
        } else {
            key.to_string()
        }
    }

    /// 加密数据
    fn encrypt_data(&self, data: &[u8]) -> Vec<u8> {
        self.stats.encryptions.fetch_add(1, Ordering::Relaxed);
        self.stats
            .bytes_decrypted
            .fetch_add(data.len() as u64, Ordering::Relaxed);

        let encrypted = self.cipher.encrypt(data);
        self.stats
            .bytes_encrypted
            .fetch_add(encrypted.len() as u64, Ordering::Relaxed);

        encrypted
    }

    /// 解密数据
    fn decrypt_data(&self, data: &[u8]) -> StorageResult<Vec<u8>> {
        match self.cipher.decrypt(data) {
            Ok(decrypted) => {
                self.stats.decryptions.fetch_add(1, Ordering::Relaxed);
                Ok(decrypted)
            }
            Err(e) => {
                self.stats.decryption_failures.fetch_add(1, Ordering::Relaxed);
                if self.config.fail_on_decrypt_error {
                    Err(StorageError::Other(format!("Decryption failed: {}", e)))
                } else {
                    // 返回原始数据
                    self.stats.decryptions.fetch_add(1, Ordering::Relaxed);
                    Ok(data.to_vec())
                }
            }
        }
    }
}

/// 详细加密统计
#[derive(Debug, Clone)]
pub struct DetailedEncryptionStats {
    /// 加密次数
    pub encryptions: u64,
    /// 解密次数
    pub decryptions: u64,
    /// 加密字节数
    pub bytes_encrypted: u64,
    /// 解密字节数
    pub bytes_decrypted: u64,
    /// 解密失败次数
    pub decryption_failures: u64,
    /// 加密开销比例
    pub encryption_overhead: f64,
    /// 解密成功率
    pub decryption_success_rate: f64,
    /// 加密器名称
    pub cipher_name: String,
    /// 密钥 ID
    pub key_id: String,
    /// 是否加密键名
    pub encrypt_keys: bool,
}

// ============================================================================
// StorageBackend 实现
// ============================================================================

#[async_trait]
impl<B: StorageBackend, C: Cipher + 'static> StorageBackend for EncryptedStorage<B, C> {
    async fn read(&self, key: &str) -> StorageResult<Vec<u8>> {
        let transformed_key = self.transform_key(key);
        let encrypted_data = self.backend.read(&transformed_key).await?;
        self.decrypt_data(&encrypted_data)
    }

    async fn write(&self, key: &str, data: &[u8]) -> StorageResult<()> {
        let transformed_key = self.transform_key(key);
        let encrypted_data = self.encrypt_data(data);
        self.backend.write(&transformed_key, &encrypted_data).await
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        let transformed_key = self.transform_key(key);
        self.backend.delete(&transformed_key).await
    }

    async fn list(&self, prefix: &str) -> StorageResult<Vec<String>> {
        if self.config.encrypt_keys {
            // 如果加密了键名，需要列出所有并解密过滤
            let all_keys = self.backend.list("").await?;
            let mut matching = Vec::new();

            for encrypted_key in all_keys {
                // 尝试解密键名
                use base64::Engine;
                if let Ok(decoded) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(&encrypted_key) {
                    if let Ok(decrypted) = self.cipher.decrypt(&decoded) {
                        if let Ok(key) = String::from_utf8(decrypted) {
                            if key.starts_with(prefix) {
                                matching.push(key);
                            }
                        }
                    }
                }
            }

            Ok(matching)
        } else {
            self.backend.list(prefix).await
        }
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        let transformed_key = self.transform_key(key);
        self.backend.exists(&transformed_key).await
    }

    fn stats(&self) -> StorageStats {
        self.backend.stats()
    }

    fn name(&self) -> &'static str {
        "EncryptedStorage"
    }
}

// ============================================================================
// 多密钥支持
// ============================================================================

/// 多密钥加密器
///
/// 支持密钥轮换，使用最新密钥加密，尝试所有密钥解密
pub struct MultiKeyCipher<C: Cipher> {
    /// 当前密钥（用于加密）
    current: Arc<C>,
    /// 历史密钥（用于解密旧数据）
    historical: Vec<Arc<C>>,
}

impl<C: Cipher> MultiKeyCipher<C> {
    /// 创建多密钥加密器
    pub fn new(current: C) -> Self {
        Self {
            current: Arc::new(current),
            historical: Vec::new(),
        }
    }

    /// 添加历史密钥
    pub fn with_historical(mut self, cipher: C) -> Self {
        self.historical.push(Arc::new(cipher));
        self
    }

    /// 轮换密钥
    pub fn rotate(&mut self, new_cipher: C) {
        let old = std::mem::replace(&mut self.current, Arc::new(new_cipher));
        self.historical.insert(0, old);
    }

    /// 获取当前密钥 ID
    pub fn current_key_id(&self) -> &str {
        self.current.key_id()
    }

    /// 获取所有密钥 ID
    pub fn all_key_ids(&self) -> Vec<&str> {
        let mut ids = vec![self.current.key_id()];
        ids.extend(self.historical.iter().map(|c| c.key_id()));
        ids
    }
}

impl<C: Cipher + 'static> Cipher for MultiKeyCipher<C> {
    fn encrypt(&self, plaintext: &[u8]) -> Vec<u8> {
        self.current.encrypt(plaintext)
    }

    fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CipherError> {
        // 首先尝试当前密钥
        if let Ok(data) = self.current.decrypt(ciphertext) {
            return Ok(data);
        }

        // 尝试历史密钥
        for cipher in &self.historical {
            if let Ok(data) = cipher.decrypt(ciphertext) {
                return Ok(data);
            }
        }

        Err(CipherError::DecryptionFailed(
            "No matching key found".to_string(),
        ))
    }

    fn name(&self) -> &'static str {
        "MultiKeyCipher"
    }

    fn key_id(&self) -> &str {
        self.current.key_id()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[test]
    fn test_xor_cipher_encrypt_decrypt() {
        let cipher = XorCipher::new(b"secret-key-here!");
        let plaintext = b"Hello, World!";

        let encrypted = cipher.encrypt(plaintext);
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_xor_cipher_with_key_id() {
        let cipher = XorCipher::with_key_id(b"my-key", "key-v1");
        assert_eq!(cipher.key_id(), "key-v1");
    }

    #[test]
    fn test_xor_cipher_key_mismatch() {
        let cipher1 = XorCipher::with_key_id(b"key1", "id1");
        let cipher2 = XorCipher::with_key_id(b"key2", "id2");

        let encrypted = cipher1.encrypt(b"test");
        let result = cipher2.decrypt(&encrypted);

        assert!(matches!(result, Err(CipherError::KeyMismatch(_))));
    }

    #[test]
    fn test_null_cipher() {
        let cipher = NullCipher::new();
        let data = b"test data";

        let encrypted = cipher.encrypt(data);
        assert_eq!(encrypted, data);

        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[test]
    fn test_base64_cipher() {
        let cipher = Base64Cipher::new();
        let data = b"Hello, World!";

        let encrypted = cipher.encrypt(data);
        // Base64 编码后应该是 ASCII
        assert!(encrypted.iter().all(|&b| b.is_ascii()));

        let decrypted = cipher.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, data);
    }

    #[tokio::test]
    async fn test_encrypted_storage_new() {
        let storage = MemoryStorage::new();
        let cipher = XorCipher::new(b"test-key-12345!!");
        let encrypted = EncryptedStorage::new(storage, cipher);

        assert_eq!(encrypted.name(), "EncryptedStorage");
        assert_eq!(encrypted.cipher_name(), "XorCipher");
    }

    #[tokio::test]
    async fn test_encrypted_storage_write_read() {
        let storage = MemoryStorage::new();
        let cipher = XorCipher::new(b"test-key-12345!!");
        let encrypted = EncryptedStorage::new(storage, cipher);

        encrypted.write("key1", b"secret data").await.unwrap();
        let data = encrypted.read("key1").await.unwrap();

        assert_eq!(data, b"secret data");
    }

    #[tokio::test]
    async fn test_encrypted_storage_data_is_encrypted() {
        let storage = Arc::new(MemoryStorage::new());
        let cipher = XorCipher::new(b"test-key-12345!!");
        let encrypted = EncryptedStorage::from_arc(Arc::clone(&storage), cipher);

        let plaintext = b"secret data";
        encrypted.write("key1", plaintext).await.unwrap();

        // 直接从后端读取应该是加密的数据
        let raw_data = storage.read("key1").await.unwrap();
        assert_ne!(raw_data, plaintext);
    }

    #[tokio::test]
    async fn test_encrypted_storage_delete() {
        let storage = MemoryStorage::new();
        let cipher = XorCipher::new(b"test-key-12345!!");
        let encrypted = EncryptedStorage::new(storage, cipher);

        encrypted.write("key1", b"data").await.unwrap();
        assert!(encrypted.exists("key1").await.unwrap());

        encrypted.delete("key1").await.unwrap();
        assert!(!encrypted.exists("key1").await.unwrap());
    }

    #[tokio::test]
    async fn test_encrypted_storage_list() {
        let storage = MemoryStorage::new();
        let cipher = XorCipher::new(b"test-key-12345!!");
        let encrypted = EncryptedStorage::new(storage, cipher);

        encrypted.write("test:a", b"1").await.unwrap();
        encrypted.write("test:b", b"2").await.unwrap();
        encrypted.write("other:c", b"3").await.unwrap();

        let keys = encrypted.list("test:").await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"test:a".to_string()));
        assert!(keys.contains(&"test:b".to_string()));
    }

    #[tokio::test]
    async fn test_encrypted_storage_stats() {
        let storage = MemoryStorage::new();
        let cipher = XorCipher::new(b"test-key-12345!!");
        let encrypted = EncryptedStorage::new(storage, cipher);

        encrypted.write("key1", b"data").await.unwrap();
        encrypted.read("key1").await.unwrap();

        let stats = encrypted.stats_snapshot();
        assert_eq!(stats.encryptions, 1);
        assert_eq!(stats.decryptions, 1);
    }

    #[tokio::test]
    async fn test_encrypted_storage_detailed_stats() {
        let storage = MemoryStorage::new();
        let cipher = XorCipher::new(b"test-key-12345!!");
        let encrypted = EncryptedStorage::new(storage, cipher);

        let stats = encrypted.detailed_stats();
        assert_eq!(stats.cipher_name, "XorCipher");
        assert_eq!(stats.key_id, "xor-default");
        assert!(!stats.encrypt_keys);
    }

    #[tokio::test]
    async fn test_encrypted_storage_with_encrypted_keys() {
        let storage = MemoryStorage::new();
        let cipher = XorCipher::new(b"test-key-12345!!");
        let config = EncryptedStorageConfig {
            encrypt_keys: true,
            ..Default::default()
        };
        let encrypted = EncryptedStorage::with_config(storage, cipher, config);

        encrypted.write("secret-key", b"data").await.unwrap();

        // 键名应该被加密
        let data = encrypted.read("secret-key").await.unwrap();
        assert_eq!(data, b"data");
    }

    #[test]
    fn test_multi_key_cipher() {
        let current = XorCipher::with_key_id(b"new-key", "v2");
        let old = XorCipher::with_key_id(b"old-key", "v1");

        let multi = MultiKeyCipher::new(current).with_historical(old.clone());

        // 使用旧密钥加密的数据
        let old_encrypted = old.encrypt(b"old data");

        // 多密钥加密器可以解密
        let decrypted = multi.decrypt(&old_encrypted).unwrap();
        assert_eq!(decrypted, b"old data");
    }

    #[test]
    fn test_multi_key_cipher_current_key_id() {
        let current = XorCipher::with_key_id(b"key", "v2");
        let multi = MultiKeyCipher::new(current);

        assert_eq!(multi.current_key_id(), "v2");
    }

    #[test]
    fn test_multi_key_cipher_all_key_ids() {
        let current = XorCipher::with_key_id(b"key2", "v2");
        let old = XorCipher::with_key_id(b"key1", "v1");

        let multi = MultiKeyCipher::new(current).with_historical(old);

        let ids = multi.all_key_ids();
        assert_eq!(ids, vec!["v2", "v1"]);
    }

    #[test]
    fn test_multi_key_cipher_rotate() {
        let v1 = XorCipher::with_key_id(b"key1", "v1");
        let v2 = XorCipher::with_key_id(b"key2", "v2");

        let mut multi = MultiKeyCipher::new(v1);
        assert_eq!(multi.current_key_id(), "v1");

        multi.rotate(v2);
        assert_eq!(multi.current_key_id(), "v2");
        assert_eq!(multi.all_key_ids(), vec!["v2", "v1"]);
    }

    #[test]
    fn test_encryption_stats_snapshot() {
        let stats = EncryptionStats::default();
        stats.encryptions.store(10, Ordering::Relaxed);
        stats.decryptions.store(8, Ordering::Relaxed);
        stats.decryption_failures.store(2, Ordering::Relaxed);

        let snapshot = stats.snapshot();
        assert_eq!(snapshot.encryptions, 10);
        assert_eq!(snapshot.decryptions, 8);
        assert_eq!(snapshot.decryption_failures, 2);
        assert!((snapshot.decryption_success_rate() - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_encryption_overhead() {
        let snapshot = EncryptionStatsSnapshot {
            encryptions: 10,
            decryptions: 10,
            bytes_encrypted: 1100,
            bytes_decrypted: 1000,
            decryption_failures: 0,
        };

        assert!((snapshot.encryption_overhead() - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_cipher_error_display() {
        let err = CipherError::DecryptionFailed("test".to_string());
        assert!(err.to_string().contains("Decryption failed"));

        let err = CipherError::InvalidFormat("bad".to_string());
        assert!(err.to_string().contains("Invalid format"));

        let err = CipherError::KeyMismatch("wrong key".to_string());
        assert!(err.to_string().contains("Key mismatch"));
    }

    #[tokio::test]
    async fn test_encrypted_storage_fail_on_decrypt_error() {
        let storage = Arc::new(MemoryStorage::new());
        let cipher = XorCipher::new(b"test-key-12345!!");
        let encrypted = EncryptedStorage::from_arc(Arc::clone(&storage), cipher);

        // 写入无效的加密数据
        storage.write("bad", b"not encrypted").await.unwrap();

        // 默认配置下应该失败
        let result = encrypted.read("bad").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_encrypted_storage_no_fail_on_decrypt_error() {
        let storage = Arc::new(MemoryStorage::new());
        let cipher = XorCipher::new(b"test-key-12345!!");
        let config = EncryptedStorageConfig {
            fail_on_decrypt_error: false,
            ..Default::default()
        };
        let encrypted = EncryptedStorage::from_arc_with_config(Arc::clone(&storage), cipher, config);

        // 写入无效的加密数据
        storage.write("bad", b"not encrypted").await.unwrap();

        // 配置为不失败，返回原始数据
        let result = encrypted.read("bad").await.unwrap();
        assert_eq!(result, b"not encrypted");
    }
}
