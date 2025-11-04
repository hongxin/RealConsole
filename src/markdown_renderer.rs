//! Markdown 渲染器 - LLM 输出美化
//!
//! 使用 termimad 将 LLM 输出的 Markdown 格式文本在终端中美化显示。
//!
//! ## 核心特性
//!
//! - **可选性**: 支持原始输出 ⇄ 美化输出切换
//! - **流式友好**: 逐行渲染,不破坏实时流式体验
//! - **极简实现**: 单一依赖,最小侵入
//! - **样式自定义**: 适配 RealConsole 配色方案
//!
//! ## 设计原则
//!
//! 遵循 RealConsole 的极简主义和易变哲学:
//! - 配置驱动: 文件配置 + 环境变量
//! - 运行时切换: `/render` 命令
//! - 平滑降级: 不支持时回退原始输出
//!
//! ## 使用示例
//!
//! ```rust
//! use realconsole::markdown_renderer::MarkdownRenderer;
//!
//! let mut renderer = MarkdownRenderer::new(true)?;
//!
//! // 流式添加内容
//! renderer.push("## 标题\n");
//! renderer.push("这是 **粗体** 文本\n");
//!
//! // 刷新并渲染
//! renderer.flush()?;
//! ```

use anyhow::Result;
use termimad::{crossterm::style::Color, MadSkin};

/// Markdown 渲染器
///
/// 支持将 LLM 输出的 Markdown 格式文本美化渲染到终端。
pub struct MarkdownRenderer {
    /// 是否启用渲染
    enabled: bool,

    /// 行缓冲区
    buffer: String,

    /// termimad 样式
    skin: MadSkin,
}

impl MarkdownRenderer {
    /// 创建新的 Markdown 渲染器
    ///
    /// # 参数
    ///
    /// * `enabled` - 是否启用渲染。false 时将直接输出原始文本
    ///
    /// # 返回
    ///
    /// 返回配置好的渲染器实例
    pub fn new(enabled: bool) -> Result<Self> {
        let mut skin = MadSkin::default();

        // 自定义样式以适配 RealConsole 配色
        Self::customize_skin(&mut skin);

        Ok(Self {
            enabled,
            buffer: String::new(),
            skin,
        })
    }

    /// 自定义 termimad 样式
    ///
    /// 🎨 Claude Code 风格配色方案（优雅、专业、易读）:
    /// - 标题: 柔和的浅蓝色（类似 Claude Code 主题色）
    /// - 粗体: 明亮的白色（清晰但不刺眼）
    /// - 斜体: 浅灰色（优雅的次要强调）
    /// - 代码块: 柔和的绿色文字 + 深灰色背景
    /// - 内联代码: 浅蓝色（与标题呼应）
    /// - 列表: 柔和的蓝色 bullet（统一主题色）
    /// - 引用: 中等灰色（适度区分）
    fn customize_skin(skin: &mut MadSkin) {
        // 🎨 标题 - 柔和的浅蓝色（RGB: 100, 180, 255）
        // 类似 Claude Code 的主题色，优雅且易读
        let header_color = Color::Rgb {
            r: 100,
            g: 180,
            b: 255,
        };
        skin.headers[0].set_fg(header_color);
        skin.headers[1].set_fg(header_color);
        skin.headers[2].set_fg(header_color);

        // 🎨 粗体 - 明亮的白色（RGB: 255, 255, 255）
        // 清晰强调，但不像黄色那样刺眼
        skin.bold.set_fg(Color::Rgb {
            r: 255,
            g: 255,
            b: 255,
        });

        // 🎨 斜体 - 浅灰色（RGB: 180, 180, 180）
        // 优雅的次要强调，与粗体形成层次
        skin.italic.set_fg(Color::Rgb {
            r: 180,
            g: 180,
            b: 180,
        });

        // 🎨 代码块 - 柔和的绿色文字（RGB: 150, 220, 150）+ 深灰色背景（RGB: 40, 40, 40）
        // 类似专业 IDE 的配色，护眼且清晰
        skin.code_block.set_fg(Color::Rgb {
            r: 150,
            g: 220,
            b: 150,
        });
        skin.code_block.set_bg(Color::Rgb { r: 40, g: 40, b: 40 });

        // 🎨 内联代码 - 浅蓝色（RGB: 130, 200, 255）
        // 与标题呼应，统一主题色
        skin.inline_code.set_fg(Color::Rgb {
            r: 130,
            g: 200,
            b: 255,
        });

        // 🎨 列表 bullet - 柔和的蓝色（RGB: 100, 180, 255）
        // 与标题相同，保持主题一致性
        skin.bullet = termimad::StyledChar::from_fg_char(
            Color::Rgb {
                r: 100,
                g: 180,
                b: 255,
            },
            '•',
        );

        // 🎨 引用块 - 中等灰色（RGB: 120, 120, 120）
        // 适度区分，不会太暗或太亮
        skin.quote_mark = termimad::StyledChar::from_fg_char(
            Color::Rgb {
                r: 120,
                g: 120,
                b: 120,
            },
            '│',
        );

        // 🎨 段落 - 稍微偏暖的白色（RGB: 240, 240, 240）
        // 柔和的白色，长时间阅读更舒适
        skin.paragraph.set_fg(Color::Rgb {
            r: 240,
            g: 240,
            b: 240,
        });
    }

    /// 添加文本到缓冲区
    ///
    /// 流式接收 LLM 输出的文本片段。
    ///
    /// # 参数
    ///
    /// * `text` - 要添加的文本片段
    pub fn push(&mut self, text: &str) {
        self.buffer.push_str(text);
    }

    /// 刷新缓冲区并渲染
    ///
    /// 将缓冲区中的所有内容渲染到终端，然后清空缓冲区。
    ///
    /// # 返回
    ///
    /// 成功返回 Ok(())，失败返回错误
    pub fn flush(&mut self) -> Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }

        if self.enabled {
            // 使用 termimad 渲染
            self.render_markdown()?;
        } else {
            // 直接输出原始文本
            print!("{}", self.buffer);
        }

        // 清空缓冲区
        self.buffer.clear();

        Ok(())
    }

    /// 渲染 markdown 文本
    ///
    /// 使用 termimad 将缓冲区中的 markdown 渲染到终端。
    fn render_markdown(&self) -> Result<()> {
        // 使用 termimad 的 print_text 方法
        // 这会自动处理换行和终端宽度
        self.skin.print_text(&self.buffer);

        Ok(())
    }

    /// 设置启用状态
    ///
    /// 运行时切换渲染开关。
    ///
    /// # 参数
    ///
    /// * `enabled` - 是否启用渲染
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 获取启用状态
    ///
    /// # 返回
    ///
    /// 当前是否启用渲染
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 立即渲染文本（不缓冲）
    ///
    /// 用于一次性渲染完整的 markdown 文本。
    ///
    /// # 参数
    ///
    /// * `text` - 要渲染的完整 markdown 文本
    ///
    /// # 返回
    ///
    /// 成功返回 Ok(())，失败返回错误
    pub fn render(&self, text: &str) -> Result<()> {
        if self.enabled {
            self.skin.print_text(text);
        } else {
            print!("{}", text);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_markdown_renderer_creation() {
        let renderer = MarkdownRenderer::new(true);
        assert!(renderer.is_ok());

        let renderer = renderer.unwrap();
        assert!(renderer.is_enabled());
    }

    #[test]
    fn test_markdown_renderer_disabled() {
        let renderer = MarkdownRenderer::new(false);
        assert!(renderer.is_ok());

        let renderer = renderer.unwrap();
        assert!(!renderer.is_enabled());
    }

    #[test]
    fn test_push_and_flush() {
        let mut renderer = MarkdownRenderer::new(false).unwrap();

        renderer.push("Hello ");
        renderer.push("World\n");

        // flush 应该成功
        let result = renderer.flush();
        assert!(result.is_ok());

        // 缓冲区应该被清空
        assert!(renderer.buffer.is_empty());
    }

    #[test]
    fn test_set_enabled() {
        let mut renderer = MarkdownRenderer::new(true).unwrap();
        assert!(renderer.is_enabled());

        renderer.set_enabled(false);
        assert!(!renderer.is_enabled());

        renderer.set_enabled(true);
        assert!(renderer.is_enabled());
    }

    #[test]
    fn test_render_immediate() {
        let renderer = MarkdownRenderer::new(false).unwrap();

        // 应该能渲染不报错
        let result = renderer.render("# Test\n\nThis is a **test**.");
        assert!(result.is_ok());
    }
}
