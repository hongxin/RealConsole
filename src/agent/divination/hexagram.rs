//! 六十四卦系统
//!
//! 由上下两个八卦组合而成

use super::trigram::Trigram;
use serde::{Deserialize, Serialize};

/// 六十四卦
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hexagram {
    /// 上卦（外卦）
    pub upper: Trigram,
    /// 下卦（内卦）
    pub lower: Trigram,
    /// 卦名
    pub name: String,
    /// 卦辞
    pub judgement: String,
}

impl Hexagram {
    /// 创建卦象
    pub fn new(upper: Trigram, lower: Trigram) -> Self {
        let name = Self::derive_name(&upper, &lower);
        let judgement = Self::derive_judgement(&upper, &lower);

        Self {
            upper,
            lower,
            name,
            judgement,
        }
    }

    /// 推导卦名（简化实现）
    fn derive_name(upper: &Trigram, lower: &Trigram) -> String {
        // 简化：使用上下卦名组合
        // 完整版应该映射到真实的六十四卦名
        use Trigram::*;

        match (upper, lower) {
            (Qian, Qian) => "乾为天".to_string(),
            (Kun, Kun) => "坤为地".to_string(),
            (Kan, Li) => "水火既济".to_string(),
            (Li, Kan) => "火水未济".to_string(),
            (Zhen, Kun) => "雷地豫".to_string(),
            (Gen, Kan) => "山水蒙".to_string(),
            (Qian, Kun) => "天地否".to_string(),
            (Kun, Qian) => "地天泰".to_string(),
            (Li, Li) => "离为火".to_string(),
            (Kan, Kan) => "坎为水".to_string(),
            (Zhen, Zhen) => "震为雷".to_string(),
            (Xun, Xun) => "巽为风".to_string(),
            (Gen, Gen) => "艮为山".to_string(),
            (Dui, Dui) => "兑为泽".to_string(),
            _ => format!("{}{}", upper.name(), lower.name()),
        }
    }

    /// 推导卦辞（简化实现）
    fn derive_judgement(upper: &Trigram, lower: &Trigram) -> String {
        use Trigram::*;

        match (upper, lower) {
            (Qian, Qian) => "元亨利贞。刚健中正，大通顺达。".to_string(),
            (Kun, Kun) => "元亨，利牝马之贞。顺承天时，厚德载物。".to_string(),
            (Kan, Li) => "既济：亨小，利贞。初吉终乱。事已成就，当保持警惕。".to_string(),
            (Li, Kan) => "未济：亨。小狐汔济，濡其尾。事未完成，当循序渐进。".to_string(),
            (Zhen, Kun) => "豫：利建侯行师。顺势而动，万物和悦。".to_string(),
            (Gen, Kan) => "蒙：亨。匪我求童蒙，童蒙求我。启蒙之时，当顺势开导。".to_string(),
            (Qian, Kun) => "否：否之匪人，不利君子贞。天地不交，当守正待时。".to_string(),
            (Kun, Qian) => "泰：小往大来，吉亨。天地交泰，通达顺畅。".to_string(),
            (Li, Li) => "离：利贞，亨。畜牝牛，吉。光明普照，当附丽中正。".to_string(),
            (Kan, Kan) => "坎：习坎，有孚，维心亨。险中求通，守信笃行。".to_string(),
            (Zhen, Zhen) => "震：亨。震来虩虩，笑言哑哑。震动警醒，有惊无险。".to_string(),
            (Xun, Xun) => "巽：小亨，利有攸往。柔顺谦逊，渗透前进。".to_string(),
            (Gen, Gen) => "艮：艮其背，不获其身。当止则止，守静待时。".to_string(),
            (Dui, Dui) => "兑：亨，利贞。喜悦和顺，以刚健居中。".to_string(),
            _ => "亨。循序而进，顺应时势，可获成功。".to_string(),
        }
    }

    /// 获取卦象符号（上下组合）
    pub fn symbol(&self) -> String {
        format!("{}\n{}", self.upper.symbol(), self.lower.symbol())
    }

    /// 获取完整描述
    pub fn full_description(&self) -> String {
        format!(
            "【{}】{}{}\n{}",
            self.name,
            self.upper.symbol(),
            self.lower.symbol(),
            self.judgement
        )
    }

    /// 获取卦象的完整信息（用于UI显示）
    pub fn display_info(&self) -> String {
        format!(
            "卦象：【{}】\n上卦：{} ({})\n下卦：{} ({})\n卦辞：{}",
            self.name,
            self.upper.name(),
            self.upper.nature(),
            self.lower.name(),
            self.lower.nature(),
            self.judgement
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Trigram::*;

    #[test]
    fn test_hexagram_creation() {
        let hexagram = Hexagram::new(Qian, Qian);
        assert_eq!(hexagram.name, "乾为天");
        assert!(hexagram.judgement.contains("元亨利贞"));
    }

    #[test]
    fn test_hexagram_symbol() {
        let hexagram = Hexagram::new(Qian, Kun);
        let symbol = hexagram.symbol();
        assert!(symbol.contains("☰"));
        assert!(symbol.contains("☷"));
    }

    #[test]
    fn test_various_hexagrams() {
        let hexagrams = vec![
            (Qian, Qian, "乾为天"),
            (Kun, Kun, "坤为地"),
            (Kan, Li, "水火既济"),
            (Li, Kan, "火水未济"),
        ];

        for (upper, lower, expected_name) in hexagrams {
            let hexagram = Hexagram::new(upper, lower);
            assert_eq!(hexagram.name, expected_name);
        }
    }

    #[test]
    fn test_hexagram_description() {
        let hexagram = Hexagram::new(Qian, Qian);
        let desc = hexagram.full_description();
        assert!(desc.contains("乾为天"));
        assert!(desc.contains("☰"));
    }
}
