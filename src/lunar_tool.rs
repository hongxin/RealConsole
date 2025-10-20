//! 农历工具实现
//!
//! 提供公历-农历双向转换、八字计算等功能
//! 基于 chinese-lunisolar-calendar crate (1901-2100)
//!
//! 功能：
//! - 公历 ↔ 农历互转
//! - 干支、生肖、星座查询
//! - 八字（四柱）计算

use crate::tool::{Parameter, ParameterType, Tool, ToolRegistry};
use chinese_lunisolar_calendar::{
    ChineseVariant, EarthlyBranch, HeavenlyStems, LunisolarDate, SolarDate,
};
use chrono::{Datelike, Local, NaiveDate, NaiveTime};
use serde_json::{json, Value as JsonValue};

/// 注册农历工具
pub fn register_lunar_tool(registry: &mut ToolRegistry) {
    let tool = Tool::new(
        "lunar_calendar",
        "农历查询与八字计算。支持公历↔农历双向转换、干支生肖星座查询、八字（四柱）计算。日期格式: YYYY-MM-DD（公历）或 lunar:YYYY-MM-DD（农历）。提供出生时间可计算完整八字（四柱八字）。支持1901-02-19到2101-01-28。",
        vec![
            Parameter {
                name: "date".to_string(),
                param_type: ParameterType::String,
                description: "日期，格式: today（今天）, YYYY-MM-DD（公历）, lunar:YYYY-MM-DD（农历，支持闰月如lunar:2023-闰02-15）"
                    .to_string(),
                required: false,
                default: Some(json!("today")),
            },
            Parameter {
                name: "time".to_string(),
                param_type: ParameterType::String,
                description: "时间（可选），格式: HH:MM，用于八字时柱计算。提供此参数会显示完整八字（四柱）"
                    .to_string(),
                required: false,
                default: None,
            },
            Parameter {
                name: "detail_level".to_string(),
                param_type: ParameterType::String,
                description: "详细程度: simple（仅日期）, standard（含干支生肖）, full（完整信息+八字）"
                    .to_string(),
                required: false,
                default: Some(json!("standard")),
            },
        ],
        |args: JsonValue| {
            let date_str = args["date"].as_str().unwrap_or("today");
            let time_str = args["time"].as_str();
            let detail_level = args["detail_level"].as_str().unwrap_or("standard");

            // 解析日期（支持公历和农历）
            let (solar_date, lunisolar_date, is_lunar_input) = parse_date_bidirectional(date_str)?;

            // 解析时间（用于八字计算）
            let time_opt = if let Some(ts) = time_str {
                Some(parse_time(ts)?)
            } else {
                None
            };

            // 根据详细程度返回不同信息
            match detail_level {
                "simple" => format_simple(&solar_date, &lunisolar_date, is_lunar_input),
                "standard" => {
                    format_standard(&solar_date, &lunisolar_date, is_lunar_input, time_opt)
                }
                "full" => format_full(&solar_date, &lunisolar_date, is_lunar_input, time_opt),
                _ => Err(format!(
                    "不支持的详细程度: {}。支持: simple, standard, full",
                    detail_level
                )),
            }
        },
    );

    registry.register(tool);
}

/// 双向解析日期（公历或农历）
fn parse_date_bidirectional(date_str: &str) -> Result<(NaiveDate, LunisolarDate, bool), String> {
    if let Some(lunar_str) = date_str.strip_prefix("lunar:") {
        // 农历输入：lunar:YYYY-MM-DD 或 lunar:YYYY-闰MM-DD
        parse_lunar_date(lunar_str)
    } else {
        // 公历输入
        let solar_naive = if date_str == "today" || date_str.is_empty() {
            Local::now().date_naive()
        } else {
            NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                .map_err(|e| format!("日期格式错误: {}。请使用 YYYY-MM-DD 格式", e))?
        };

        let lunisolar = LunisolarDate::from_naive_date(solar_naive)
            .map_err(|e| format!("农历转换失败: {:?}。支持范围: 1901-02-19 到 2101-01-28", e))?;

        Ok((solar_naive, lunisolar, false))
    }
}

/// 解析农历日期字符串
fn parse_lunar_date(lunar_str: &str) -> Result<(NaiveDate, LunisolarDate, bool), String> {
    // 支持格式：2024-01-01 或 2024-闰02-15
    let parts: Vec<&str> = lunar_str.split('-').collect();
    if parts.len() != 3 {
        return Err("农历日期格式错误。格式: YYYY-MM-DD 或 YYYY-闰MM-DD".to_string());
    }

    let year: u16 = parts[0]
        .parse()
        .map_err(|_| "农历年份解析失败".to_string())?;

    // 检查是否是闰月
    let (month, is_leap) = if parts[1].starts_with('闰') {
        let month_str = &parts[1][3..]; // "闰" 是3字节
        let month: u8 = month_str
            .parse()
            .map_err(|_| "农历月份解析失败".to_string())?;
        (month, true)
    } else {
        let month: u8 = parts[1]
            .parse()
            .map_err(|_| "农历月份解析失败".to_string())?;
        (month, false)
    };

    let day: u8 = parts[2]
        .parse()
        .map_err(|_| "农历日期解析失败".to_string())?;

    // 从农历创建LunisolarDate
    let lunisolar = LunisolarDate::from_ymd(year, month, is_leap, day)
        .map_err(|e| format!("农历日期无效: {:?}", e))?;

    // 转换到公历
    let solar = lunisolar.to_solar_date();
    let solar_naive = solar.to_naive_date();

    Ok((solar_naive, lunisolar, true))
}

/// 解析时间字符串（HH:MM）
fn parse_time(time_str: &str) -> Result<NaiveTime, String> {
    NaiveTime::parse_from_str(time_str, "%H:%M")
        .map_err(|e| format!("时间格式错误: {}。请使用 HH:MM 格式（如 14:30）", e))
}

/// Simple 格式：仅日期转换
fn format_simple(
    solar: &NaiveDate,
    lunar: &LunisolarDate,
    is_lunar_input: bool,
) -> Result<String, String> {
    let chinese_str = lunar.to_chinese_string(ChineseVariant::Simple);

    if is_lunar_input {
        // 农历 → 公历
        Ok(format!(
            "农历 {} → 公历 {}",
            chinese_str,
            solar.format("%Y-%m-%d")
        ))
    } else {
        // 公历 → 农历
        Ok(format!(
            "公历 {} → 农历 {}",
            solar.format("%Y-%m-%d"),
            chinese_str
        ))
    }
}

/// Standard 格式：含干支生肖
fn format_standard(
    solar: &NaiveDate,
    lunar: &LunisolarDate,
    is_lunar_input: bool,
    time_opt: Option<NaiveTime>,
) -> Result<String, String> {
    let chinese_str = lunar.to_chinese_string(ChineseVariant::Simple);
    let parts: Vec<&str> = chinese_str.split('　').collect();

    let mut result = if is_lunar_input {
        format!(
            "📅 农历 {}\n公历 {}",
            chinese_str.replace('　', " "),
            solar.format("%Y年%m月%d日")
        )
    } else {
        format!(
            "📅 公历 {}\n农历 {}",
            solar.format("%Y年%m月%d日"),
            chinese_str.replace('　', " ")
        )
    };

    // 解析干支和生肖
    if parts.len() >= 2 {
        let zodiac_info = parts[1];
        if zodiac_info.contains('、') {
            let zodiac_parts: Vec<&str> = zodiac_info.split('、').collect();
            if zodiac_parts.len() >= 2 {
                result.push_str(&format!("\n干支: {}", zodiac_parts[0]));
                result.push_str(&format!(
                    "\n生肖: {}",
                    zodiac_parts[1].trim_end_matches('年')
                ));
            }
        }
    }

    // 如果提供了时间，显示简化八字
    if let Some(time) = time_opt {
        result.push_str(&format!("\n\n⏰ 出生时间: {}", time.format("%H:%M")));
        result.push_str(&format!("\n{}", format_bazi_simple(lunar, time)?));
    }

    Ok(result)
}

/// Full 格式：完整信息+八字
fn format_full(
    solar: &NaiveDate,
    lunar: &LunisolarDate,
    is_lunar_input: bool,
    time_opt: Option<NaiveTime>,
) -> Result<String, String> {
    let chinese_str = lunar.to_chinese_string(ChineseVariant::Simple);
    let parts: Vec<&str> = chinese_str.split('　').collect();

    let mut result = String::from("═══ 农历信息 ═══\n\n");

    // 公历信息
    result.push_str("📅 公历\n");
    result.push_str(&format!("  日期: {}\n", solar.format("%Y年%m月%d日")));
    result.push_str(&format!("  星期: {}\n", format_weekday(solar.weekday())));

    // 农历完整信息
    result.push_str("\n🌙 农历\n");
    result.push_str(&format!("  {}\n", chinese_str.replace('　', " ")));

    // 解析干支和生肖
    if parts.len() >= 2 {
        let zodiac_info = parts[1];
        if zodiac_info.contains('、') {
            let zodiac_parts: Vec<&str> = zodiac_info.split('、').collect();
            if zodiac_parts.len() >= 2 {
                result.push_str(&format!("  干支: {}\n", zodiac_parts[0]));
                result.push_str(&format!(
                    "  生肖: {}\n",
                    zodiac_parts[1].trim_end_matches('年')
                ));
            }
        }
    }

    // 八字（四柱）
    if let Some(time) = time_opt {
        result.push_str(&format!("\n⏰ 出生时间: {}\n", time.format("%H:%M")));
        result.push_str(&format!("\n{}", format_bazi_full(lunar, time)?));
    }

    if is_lunar_input {
        result.push_str(&format!(
            "\n💡 农历输入，转换结果: 公历 {}",
            solar.format("%Y年%m月%d日")
        ));
    }

    result.push_str("\n══════════════════");

    Ok(result)
}

/// 格式化星期
fn format_weekday(weekday: chrono::Weekday) -> &'static str {
    match weekday {
        chrono::Weekday::Mon => "星期一",
        chrono::Weekday::Tue => "星期二",
        chrono::Weekday::Wed => "星期三",
        chrono::Weekday::Thu => "星期四",
        chrono::Weekday::Fri => "星期五",
        chrono::Weekday::Sat => "星期六",
        chrono::Weekday::Sun => "星期日",
    }
}

/// 计算并格式化八字（简化版）
fn format_bazi_simple(lunar: &LunisolarDate, time: NaiveTime) -> Result<String, String> {
    let bazi = calculate_bazi(lunar, time)?;
    Ok(format!(
        "八字: {} {} {} {}",
        bazi.year, bazi.month, bazi.day, bazi.hour
    ))
}

/// 计算并格式化八字（完整版）
fn format_bazi_full(lunar: &LunisolarDate, time: NaiveTime) -> Result<String, String> {
    let bazi = calculate_bazi(lunar, time)?;

    let mut result = String::from("🔮 八字（四柱）\n");
    result.push_str(&format!(
        "  年柱: {} （{}{}）\n",
        bazi.year, bazi.year_stem, bazi.year_branch
    ));
    result.push_str(&format!(
        "  月柱: {} （{}{}）\n",
        bazi.month, bazi.month_stem, bazi.month_branch
    ));
    result.push_str(&format!(
        "  日柱: {} （{}{}）\n",
        bazi.day, bazi.day_stem, bazi.day_branch
    ));
    result.push_str(&format!(
        "  时柱: {} （{}{}）\n",
        bazi.hour, bazi.hour_stem, bazi.hour_branch
    ));

    // 八字重量（称骨算命）
    let weight = lunar.get_ba_zi_weight_by_time(time);
    result.push_str(&format!("\n💎 八字重量: {} 两", weight));

    Ok(result)
}

/// 八字数据结构
struct BaZi {
    year: String,
    year_stem: String,
    year_branch: String,
    month: String,
    month_stem: String,
    month_branch: String,
    day: String,
    day_stem: String,
    day_branch: String,
    hour: String,
    hour_stem: String,
    hour_branch: String,
}

/// 计算八字（四柱）
fn calculate_bazi(lunar: &LunisolarDate, time: NaiveTime) -> Result<BaZi, String> {
    // 获取农历年份信息
    let lunar_year = lunar.get_lunar_year();
    let year_stem = lunar_year.get_heavenly_stems();
    let year_branch = lunar_year.get_earthly_branch();

    // 时辰地支
    let hour_branch = EarthlyBranch::from_time(time);

    // 时辰天干的计算：根据日天干和时辰地支推算
    // 使用日干求时干的口诀：
    // 甲己还加甲，乙庚丙作初，丙辛从戊起，丁壬庚子居，戊癸何方发，壬子是真途
    let solar_date = lunar.to_solar_date();
    let naive_date = solar_date.to_naive_date();

    // 获取日天干地支（通过NaiveDate计算）
    let (day_stem, day_branch) = calculate_day_pillar(naive_date)?;

    // 获取月柱
    let (month_stem, month_branch) = calculate_month_pillar(lunar, year_stem)?;

    // 计算时干
    let hour_stem = calculate_hour_stem(day_stem, hour_branch);

    Ok(BaZi {
        year: format!("{}{}", year_stem, year_branch),
        year_stem: format!("{}", year_stem),
        year_branch: format!("{}", year_branch),
        month: format!("{}{}", month_stem, month_branch),
        month_stem: format!("{}", month_stem),
        month_branch: format!("{}", month_branch),
        day: format!("{}{}", day_stem, day_branch),
        day_stem: format!("{}", day_stem),
        day_branch: format!("{}", day_branch),
        hour: format!("{}{}", hour_stem, hour_branch),
        hour_stem: format!("{}", hour_stem),
        hour_branch: format!("{}", hour_branch),
    })
}

/// 计算日柱（基于公元纪年的天干地支算法）
fn calculate_day_pillar(date: NaiveDate) -> Result<(HeavenlyStems, EarthlyBranch), String> {
    // 使用公历日期计算日干支
    // 基准：公元1年1月1日是甲辰日
    let base_date = NaiveDate::from_ymd_opt(1, 1, 1).unwrap();
    let days = date.signed_duration_since(base_date).num_days();

    // 天干周期10，地支周期12
    let stem_index = (days + 9) % 10; // +9是因为基准日是甲（索引0），但要调整到正确位置
    let branch_index = (days + 9) % 12;

    let stem = unsafe { HeavenlyStems::from_ordinal_unsafe(stem_index as i8) };
    let branch = unsafe { EarthlyBranch::from_ordinal_unsafe(branch_index as i8) };

    Ok((stem, branch))
}

/// 计算月柱
fn calculate_month_pillar(
    lunar: &LunisolarDate,
    year_stem: HeavenlyStems,
) -> Result<(HeavenlyStems, EarthlyBranch), String> {
    // 获取农历月份
    let lunar_month = lunar.get_lunar_month();
    let month_num = lunar_month.to_u8();

    // 月地支：寅月（正月）、卯月（二月）...
    let branch_index = (month_num as i8 + 1) % 12; // 正月是寅（索引2）
    let month_branch = unsafe { EarthlyBranch::from_ordinal_unsafe(branch_index) };

    // 月天干：根据年干推算
    // 甲己之年丙作首，乙庚之岁戊为头，丙辛必定寻庚上，丁壬壬位顺行流，若问戊癸何方起，甲寅之上好追求
    let year_stem_index = year_stem as usize;
    let base_stem_index = match year_stem_index % 5 {
        0 => 2, // 甲、己年从丙开始
        1 => 4, // 乙、庚年从戊开始
        2 => 6, // 丙、辛年从庚开始
        3 => 8, // 丁、壬年从壬开始
        4 => 0, // 戊、癸年从甲开始
        _ => unreachable!(),
    };

    let month_stem_index = (base_stem_index + month_num as usize - 1) % 10;
    let month_stem = unsafe { HeavenlyStems::from_ordinal_unsafe(month_stem_index as i8) };

    Ok((month_stem, month_branch))
}

/// 计算时干
fn calculate_hour_stem(day_stem: HeavenlyStems, hour_branch: EarthlyBranch) -> HeavenlyStems {
    // 日干支推时干：
    // 甲己还加甲，乙庚丙作初，丙辛从戊起，丁壬庚子居，戊癸何方发，壬子是真途
    let day_stem_index = day_stem as usize;
    let hour_branch_index = hour_branch as usize;

    let base_stem_index = match day_stem_index % 5 {
        0 => 0, // 甲、己日从甲开始
        1 => 2, // 乙、庚日从丙开始
        2 => 4, // 丙、辛日从戊开始
        3 => 6, // 丁、壬日从庚开始
        4 => 8, // 戊、癸日从壬开始
        _ => unreachable!(),
    };

    let hour_stem_index = (base_stem_index + hour_branch_index) % 10;
    unsafe { HeavenlyStems::from_ordinal_unsafe(hour_stem_index as i8) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_solar_date() {
        let result = parse_date_bidirectional("2024-01-01");
        assert!(result.is_ok());
        let (solar, _, is_lunar) = result.unwrap();
        assert_eq!(solar.to_string(), "2024-01-01");
        assert!(!is_lunar);
    }

    #[test]
    fn test_parse_lunar_date() {
        // 农历2024年正月初一 = 公历2024-02-10
        let result = parse_date_bidirectional("lunar:2024-01-01");
        assert!(result.is_ok());
        let (solar, _, is_lunar) = result.unwrap();
        assert_eq!(solar.to_string(), "2024-02-10");
        assert!(is_lunar);
    }

    #[test]
    fn test_parse_time() {
        let result = parse_time("14:30");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().format("%H:%M").to_string(), "14:30");
    }

    #[test]
    fn test_bazi_calculation() {
        // 测试八字计算
        let solar = NaiveDate::from_ymd_opt(2024, 2, 10).unwrap();
        let lunar = LunisolarDate::from_naive_date(solar).unwrap();
        let time = NaiveTime::from_hms_opt(14, 30, 0).unwrap();

        let bazi = calculate_bazi(&lunar, time);
        assert!(bazi.is_ok());

        let bazi = bazi.unwrap();
        // 验证八字格式正确（每柱两个汉字）
        assert_eq!(bazi.year.chars().count(), 2); // 两个汉字
        assert_eq!(bazi.month.chars().count(), 2);
        assert_eq!(bazi.day.chars().count(), 2);
        assert_eq!(bazi.hour.chars().count(), 2);
    }

    #[test]
    fn test_day_pillar_calculation() {
        // 测试日柱计算
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let result = calculate_day_pillar(date);
        assert!(result.is_ok());
    }
}
