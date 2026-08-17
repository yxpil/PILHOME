//! 类 Cron 定时规则:分 时 日 月 星期。
//!
//! 语法示例:
//! - `30 8 * * 1-5`   —— 工作日 08:30
//! - `0 23 * * *`     —— 每天 23:00
//! - `0 */2 * * *`    —— 每 2 小时整点

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// 一个字段的取值集合。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct Field {
    values: BTreeSet<u32>,
    step: Option<u32>,
}

impl Field {
    fn contains(&self, v: u32) -> bool {
        match self.step {
            Some(step) if step > 0 => v % step == 0,
            _ => self.values.contains(&v),
        }
    }
}

/// 解析字段:支持 `*`、`*/n`、`数字`、`a,b,c`、`a-b`。
fn parse_field(text: &str, min: u32, max: u32) -> Result<Field, String> {
    let text = text.trim();
    let mut field = Field::default();
    if let Some(step_text) = text.strip_prefix("*/") {
        let step: u32 = step_text.parse().map_err(|_| format!("非法步进: {text}"))?;
        if step == 0 || step > max {
            return Err(format!("步进越界: {text}"));
        }
        field.step = Some(step);
        return Ok(field);
    }
    for part in text.split(',') {
        if part == "*" {
            for v in min..=max {
                field.values.insert(v);
            }
        } else if let Some((lo, hi)) = part.split_once('-') {
            let lo: u32 = lo.trim().parse().map_err(|_| format!("非法数字: {part}"))?;
            let hi: u32 = hi.trim().parse().map_err(|_| format!("非法数字: {part}"))?;
            if lo > hi || lo < min || hi > max {
                return Err(format!("范围越界: {part}"));
            }
            for v in lo..=hi {
                field.values.insert(v);
            }
        } else {
            let v: u32 = part.trim().parse().map_err(|_| format!("非法数字: {part}"))?;
            if v < min || v > max {
                return Err(format!("数字越界: {part}"));
            }
            field.values.insert(v);
        }
    }
    Ok(field)
}

/// 定时规则。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronRule {
    /// 规则名称(如 `夜间布防`)。
    pub name: String,
    /// 触发的动作(如 `arm` / `disarm` / `patrol`)。
    pub action: String,
    minute: Field,
    hour: Field,
    day: Field,
    month: Field,
    weekday: Field,
}

/// 解析类 Cron 规则文本:`分 时 日 月 星期`。
pub fn parse_rule(name: &str, action: &str, expr: &str) -> Result<CronRule, String> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if parts.len() != 5 {
        return Err(format!("规则需要 5 段(分 时 日 月 星期),实际 {} 段", parts.len()));
    }
    Ok(CronRule {
        name: name.to_string(),
        action: action.to_string(),
        minute: parse_field(parts[0], 0, 59)?,
        hour: parse_field(parts[1], 0, 23)?,
        day: parse_field(parts[2], 1, 31)?,
        month: parse_field(parts[3], 1, 12)?,
        weekday: parse_field(parts[4], 0, 6)?,
    })
}

impl CronRule {
    /// 判断给定时刻(Unix 秒)是否命中。
    pub fn matches(&self, ts: u64) -> bool {
        let secs = ts % 3600;
        let minute = (secs / 60) as u32;
        let hour = ((ts / 3600) % 24) as u32;

        let days_since_epoch = (ts / 86400) as u32;
        let (_, month, day) = civil_from_days(days_since_epoch);
        let weekday = ((days_since_epoch + 3) % 7) as u8; // 1970-01-01 为周四

        self.minute.contains(minute)
            && self.hour.contains(hour)
            && self.day.contains(day)
            && self.month.contains(month)
            && self.weekday.contains(weekday.into())
    }
}

/// 儒略日 -> 公历日期(Howard Hinnant 算法)。
fn civil_from_days(z: u32) -> (u32, u32, u32) {
    let z = z as i64 + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y } as u32;
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekday_rule_matches() {
        let rule = parse_rule("工作日布防", "arm", "30 8 * * 1-5").expect("parse ok");
        // 2026-08-17 08:30 UTC(周一)。
        assert!(rule.matches(1_785_832_200));
    }

    #[test]
    fn every_two_hours() {
        let rule = parse_rule("两小时", "patrol", "0 */2 * * *").expect("parse ok");
        assert!(rule.matches(1_785_880_800));
    }
}
