//! 两仪：阴阳二元状态
//!
//! 太极分化为两仪，体现阴阳对立统一

use super::taiji::Taiji;

/// 两仪：阴阳二元状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Liangyyi {
    /// 太阴 ☽ - 极静、深层、内敛、收藏
    Taiyin,

    /// 太阳 ☉ - 极动、表层、外放、发散
    Taiyang,
}

impl Liangyyi {
    /// 从太极分化
    ///
    /// 根据阴阳能量的主导确定两仪
    pub fn from_taiji(taiji: &Taiji) -> Self {
        if taiji.yin_energy > taiji.yang_energy {
            Liangyyi::Taiyin
        } else {
            Liangyyi::Taiyang
        }
    }

    /// 转换到对立面
    pub fn opposite(&self) -> Self {
        match self {
            Liangyyi::Taiyin => Liangyyi::Taiyang,
            Liangyyi::Taiyang => Liangyyi::Taiyin,
        }
    }

    /// 符号表示
    pub fn symbol(&self) -> &'static str {
        match self {
            Liangyyi::Taiyin => "☽",
            Liangyyi::Taiyang => "☉",
        }
    }

    /// 描述
    pub fn description(&self) -> &'static str {
        match self {
            Liangyyi::Taiyin => "太阴·静·收·聚·藏",
            Liangyyi::Taiyang => "太阳·动·放·散·发",
        }
    }

    /// 是否为阴
    pub fn is_yin(&self) -> bool {
        matches!(self, Liangyyi::Taiyin)
    }

    /// 是否为阳
    pub fn is_yang(&self) -> bool {
        matches!(self, Liangyyi::Taiyang)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::liangyyi::taiji::Event;

    #[test]
    fn test_from_taiji_yin() {
        let mut taiji = Taiji::new();
        taiji.update_from_event(&Event::UserRead);

        let liangyyi = Liangyyi::from_taiji(&taiji);
        assert_eq!(liangyyi, Liangyyi::Taiyin);
        assert!(liangyyi.is_yin());
    }

    #[test]
    fn test_from_taiji_yang() {
        let mut taiji = Taiji::new();
        taiji.update_from_event(&Event::UserExecute);

        let liangyyi = Liangyyi::from_taiji(&taiji);
        assert_eq!(liangyyi, Liangyyi::Taiyang);
        assert!(liangyyi.is_yang());
    }

    #[test]
    fn test_opposite() {
        assert_eq!(Liangyyi::Taiyin.opposite(), Liangyyi::Taiyang);
        assert_eq!(Liangyyi::Taiyang.opposite(), Liangyyi::Taiyin);
    }

    #[test]
    fn test_symbol() {
        assert_eq!(Liangyyi::Taiyin.symbol(), "☽");
        assert_eq!(Liangyyi::Taiyang.symbol(), "☉");
    }
}
