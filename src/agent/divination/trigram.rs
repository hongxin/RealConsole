//! 八卦系统
//!
//! 将 Intent/Tool 类型映射到八卦符号

use serde::{Deserialize, Serialize};

/// 八卦枚举
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Trigram {
    /// 乾（☰）- 天，创造
    Qian,
    /// 坤（☷）- 地，承载
    Kun,
    /// 震（☳）- 雷，启动
    Zhen,
    /// 巽（☴）- 风，传播
    Xun,
    /// 坎（☵）- 水，流动
    Kan,
    /// 离（☲）- 火，明照
    Li,
    /// 艮（☶）- 山，止藏
    Gen,
    /// 兑（☱）- 泽，悦通
    Dui,
}

impl Trigram {
    /// 获取卦象符号（Unicode）
    pub fn symbol(&self) -> &'static str {
        match self {
            Trigram::Qian => "☰",
            Trigram::Kun => "☷",
            Trigram::Zhen => "☳",
            Trigram::Xun => "☴",
            Trigram::Kan => "☵",
            Trigram::Li => "☲",
            Trigram::Gen => "☶",
            Trigram::Dui => "☱",
        }
    }

    /// 获取卦名
    pub fn name(&self) -> &'static str {
        match self {
            Trigram::Qian => "乾",
            Trigram::Kun => "坤",
            Trigram::Zhen => "震",
            Trigram::Xun => "巽",
            Trigram::Kan => "坎",
            Trigram::Li => "离",
            Trigram::Gen => "艮",
            Trigram::Dui => "兑",
        }
    }

    /// 获取属性描述
    pub fn nature(&self) -> &'static str {
        match self {
            Trigram::Qian => "天，创造，刚健",
            Trigram::Kun => "地，承载，厚德",
            Trigram::Zhen => "雷，启动，震动",
            Trigram::Xun => "风，传播，渗透",
            Trigram::Kan => "水，流动，险陷",
            Trigram::Li => "火，明照，附丽",
            Trigram::Gen => "山，止藏，静止",
            Trigram::Dui => "泽，悦通，喜悦",
        }
    }

    /// 根据工具名称映射卦象
    pub fn from_tool_name(tool_name: &str) -> Self {
        // 关键词匹配
        let tool_lower = tool_name.to_lowercase();

        if tool_lower.contains("create") || tool_lower.contains("init") || tool_lower.contains("new") {
            Trigram::Qian  // 创造
        } else if tool_lower.contains("list") || tool_lower.contains("count") || tool_lower.contains("show") {
            Trigram::Kun   // 承载
        } else if tool_lower.contains("start") || tool_lower.contains("run") || tool_lower.contains("exec") {
            Trigram::Zhen  // 启动
        } else if tool_lower.contains("send") || tool_lower.contains("request") || tool_lower.contains("fetch") {
            Trigram::Xun   // 传播
        } else if tool_lower.contains("read") || tool_lower.contains("stream") || tool_lower.contains("download") {
            Trigram::Kan   // 流动
        } else if tool_lower.contains("search") || tool_lower.contains("find") || tool_lower.contains("grep") {
            Trigram::Li    // 明照
        } else if tool_lower.contains("stop") || tool_lower.contains("save") || tool_lower.contains("backup") {
            Trigram::Gen   // 止藏
        } else if tool_lower.contains("interact") || tool_lower.contains("prompt") || tool_lower.contains("ask") {
            Trigram::Dui   // 悦通
        } else {
            Trigram::Kun   // 默认
        }
    }

    /// 获取对应的颜色（用于UI）
    pub fn color(&self) -> &'static str {
        match self {
            Trigram::Qian => "#FFD700",  // 金色
            Trigram::Kun => "#8B4513",   // 土色
            Trigram::Zhen => "#FF4500",  // 橙红
            Trigram::Xun => "#00CED1",   // 青色
            Trigram::Kan => "#1E90FF",   // 蓝色
            Trigram::Li => "#FF6347",    // 火红
            Trigram::Gen => "#696969",   // 灰色
            Trigram::Dui => "#98FB98",   // 淡绿
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigram_symbols() {
        assert_eq!(Trigram::Qian.symbol(), "☰");
        assert_eq!(Trigram::Kun.symbol(), "☷");
        assert_eq!(Trigram::Zhen.symbol(), "☳");
        assert_eq!(Trigram::Li.symbol(), "☲");
    }

    #[test]
    fn test_trigram_from_tool_name() {
        assert_eq!(Trigram::from_tool_name("create_file"), Trigram::Qian);
        assert_eq!(Trigram::from_tool_name("list_directory"), Trigram::Kun);
        assert_eq!(Trigram::from_tool_name("search_text"), Trigram::Li);
        assert_eq!(Trigram::from_tool_name("find_file"), Trigram::Li);
        assert_eq!(Trigram::from_tool_name("start_process"), Trigram::Zhen);
    }

    #[test]
    fn test_trigram_nature() {
        assert_eq!(Trigram::Qian.nature(), "天，创造，刚健");
        assert_eq!(Trigram::Li.nature(), "火，明照，附丽");
    }

    #[test]
    fn test_trigram_colors() {
        assert_eq!(Trigram::Qian.color(), "#FFD700");
        assert_eq!(Trigram::Li.color(), "#FF6347");
    }
}
