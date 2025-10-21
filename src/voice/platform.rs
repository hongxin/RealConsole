//! 平台相关的语音实现
//!
//! 根据操作系统选择合适的 TTS 引擎

use std::process::Command;

/// 平台语音接口
pub struct PlatformVoice;

impl PlatformVoice {
    /// 检测当前平台是否支持语音播报
    pub fn is_supported() -> bool {
        #[cfg(target_os = "macos")]
        {
            Self::check_macos_say()
        }
        #[cfg(target_os = "linux")]
        {
            Self::check_linux_tts()
        }
        #[cfg(target_os = "windows")]
        {
            true // Windows 默认支持 PowerShell TTS
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            false
        }
    }

    /// 播报文本（同步版本）
    ///
    /// # 参数
    /// - `text`: 要播报的文本
    /// - `voice`: 语音名称（可选，使用系统默认）
    ///
    /// # 返回
    /// - `Ok(())`: 播报成功
    /// - `Err(String)`: 播报失败
    pub fn speak(text: &str, voice: Option<&str>) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            Self::macos_say(text, voice)
        }
        #[cfg(target_os = "linux")]
        {
            Self::linux_espeak(text, voice)
        }
        #[cfg(target_os = "windows")]
        {
            Self::windows_powershell_tts(text)
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Err("当前平台不支持语音播报".to_string())
        }
    }

    // ========== macOS 实现 ==========

    #[cfg(target_os = "macos")]
    fn check_macos_say() -> bool {
        Command::new("which")
            .arg("say")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[cfg(target_os = "macos")]
    fn macos_say(text: &str, voice: Option<&str>) -> Result<(), String> {
        let mut cmd = Command::new("say");

        if let Some(v) = voice {
            cmd.arg("-v").arg(v);
        }

        cmd.arg(text);

        cmd.output()
            .map_err(|e| format!("执行 say 命令失败: {}", e))
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(format!(
                        "say 命令执行失败: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ))
                }
            })
    }

    // ========== Linux 实现 ==========

    #[cfg(target_os = "linux")]
    fn check_linux_tts() -> bool {
        // 检查 espeak 或 festival
        Command::new("which")
            .arg("espeak")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
            || Command::new("which")
                .arg("festival")
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
    }

    #[cfg(target_os = "linux")]
    fn linux_espeak(text: &str, _voice: Option<&str>) -> Result<(), String> {
        let mut cmd = Command::new("espeak");
        cmd.arg(text);

        cmd.output()
            .map_err(|e| format!("执行 espeak 命令失败: {}", e))
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(format!(
                        "espeak 命令执行失败: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ))
                }
            })
    }

    // ========== Windows 实现 ==========

    #[cfg(target_os = "windows")]
    fn windows_powershell_tts(text: &str) -> Result<(), String> {
        let script = format!(
            "Add-Type -AssemblyName System.Speech; \
             $speak = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
             $speak.Speak('{}')",
            text.replace('\'', "''") // 转义单引号
        );

        let mut cmd = Command::new("powershell");
        cmd.arg("-Command").arg(&script);

        cmd.output()
            .map_err(|e| format!("执行 PowerShell 命令失败: {}", e))
            .and_then(|output| {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(format!(
                        "PowerShell TTS 执行失败: {}",
                        String::from_utf8_lossy(&output.stderr)
                    ))
                }
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_support() {
        // 至少应该能检测到当前平台
        let supported = PlatformVoice::is_supported();
        println!("当前平台语音支持: {}", supported);
    }

    #[test]
    #[ignore] // 忽略此测试，因为会播放声音
    fn test_speak() {
        if PlatformVoice::is_supported() {
            let result = PlatformVoice::speak("测试语音播报", None);
            assert!(result.is_ok());
        }
    }
}
