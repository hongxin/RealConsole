//! 语音播报器
//!
//! 提供异步语音播报功能，支持队列管理

use super::platform::PlatformVoice;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

/// 播报配置
#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    /// 是否启用语音播报
    pub enabled: bool,
    /// 语音名称（可选）
    pub voice: Option<String>,
    /// 最大队列长度
    pub max_queue_size: usize,
}

impl Default for BroadcastConfig {
    fn default() -> Self {
        Self {
            enabled: false, // 默认禁用
            voice: None,
            max_queue_size: 10,
        }
    }
}

/// 语音播报器
///
/// 使用后台线程处理语音播报队列，避免阻塞主线程
pub struct VoiceBroadcaster {
    /// 播报配置
    config: Arc<Mutex<BroadcastConfig>>,
    /// 消息发送通道
    tx: mpsc::UnboundedSender<String>,
}

impl VoiceBroadcaster {
    /// 创建新的语音播报器
    ///
    /// # 参数
    /// - `config`: 播报配置
    ///
    /// # 返回
    /// 语音播报器实例
    pub fn new(config: BroadcastConfig) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let config_arc = Arc::new(Mutex::new(config));

        // 启动后台任务处理播报队列
        let config_clone = Arc::clone(&config_arc);
        tokio::spawn(async move {
            Self::broadcast_worker(rx, config_clone).await;
        });

        Self {
            config: config_arc,
            tx,
        }
    }

    /// 检查当前平台是否支持语音播报
    pub fn is_platform_supported() -> bool {
        PlatformVoice::is_supported()
    }

    /// 启用语音播报
    pub async fn enable(&self) {
        let mut config = self.config.lock().await;
        config.enabled = true;
    }

    /// 禁用语音播报
    pub async fn disable(&self) {
        let mut config = self.config.lock().await;
        config.enabled = false;
    }

    /// 检查是否已启用
    pub async fn is_enabled(&self) -> bool {
        let config = self.config.lock().await;
        config.enabled
    }

    /// 设置语音
    pub async fn set_voice(&self, voice: Option<String>) {
        let mut config = self.config.lock().await;
        config.voice = voice;
    }

    /// 播报文本（异步，非阻塞）
    ///
    /// # 参数
    /// - `text`: 要播报的文本
    ///
    /// # 返回
    /// - `Ok(())`: 成功加入播报队列
    /// - `Err(String)`: 加入队列失败
    pub async fn speak(&self, text: impl Into<String>) -> Result<(), String> {
        let config = self.config.lock().await;

        // 如果未启用，直接返回
        if !config.enabled {
            return Ok(());
        }

        let text = text.into();

        // 发送到播报队列
        self.tx
            .send(text)
            .map_err(|e| format!("发送到播报队列失败: {}", e))
    }

    /// 播报工作线程
    ///
    /// 从队列中取出文本并播报
    async fn broadcast_worker(
        mut rx: mpsc::UnboundedReceiver<String>,
        config: Arc<Mutex<BroadcastConfig>>,
    ) {
        while let Some(text) = rx.recv().await {
            // 在独立的任务中执行播报，避免阻塞队列
            let config_clone = Arc::clone(&config);
            tokio::task::spawn_blocking(move || {
                let config = tokio::runtime::Handle::current()
                    .block_on(config_clone.lock());

                let voice = config.voice.as_deref();

                if let Err(e) = PlatformVoice::speak(&text, voice) {
                    eprintln!("⚠ 语音播报失败: {}", e);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_broadcaster_creation() {
        let config = BroadcastConfig::default();
        let broadcaster = VoiceBroadcaster::new(config);

        assert!(!broadcaster.is_enabled().await);
    }

    #[tokio::test]
    async fn test_enable_disable() {
        let config = BroadcastConfig::default();
        let broadcaster = VoiceBroadcaster::new(config);

        assert!(!broadcaster.is_enabled().await);

        broadcaster.enable().await;
        assert!(broadcaster.is_enabled().await);

        broadcaster.disable().await;
        assert!(!broadcaster.is_enabled().await);
    }

    #[tokio::test]
    async fn test_set_voice() {
        let config = BroadcastConfig::default();
        let broadcaster = VoiceBroadcaster::new(config);

        broadcaster.set_voice(Some("Ting-Ting".to_string())).await;

        let config = broadcaster.config.lock().await;
        assert_eq!(config.voice, Some("Ting-Ting".to_string()));
    }

    #[tokio::test]
    async fn test_speak_when_disabled() {
        let config = BroadcastConfig::default();
        let broadcaster = VoiceBroadcaster::new(config);

        // 未启用时也应该返回 Ok，只是不播报
        let result = broadcaster.speak("测试").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // 忽略此测试，因为会播放声音
    async fn test_speak_when_enabled() {
        if !VoiceBroadcaster::is_platform_supported() {
            println!("跳过测试：当前平台不支持语音播报");
            return;
        }

        let mut config = BroadcastConfig::default();
        config.enabled = true;
        let broadcaster = VoiceBroadcaster::new(config);

        let result = broadcaster.speak("测试语音播报").await;
        assert!(result.is_ok());

        // 等待播报完成
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }
}
