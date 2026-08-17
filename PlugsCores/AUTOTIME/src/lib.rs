//! AUTOTIME —— 定时和节律控制
//!
//! 双引擎定时控制:
//! - [`schedule`] 类 Cron 定时规则(布防 / 撤防 / 巡检)
//! - [`rhythm`] 昼夜节律曲线(家庭活跃节律,自动推荐布防时段)

pub mod rhythm;
pub mod schedule;

pub use rhythm::{RhythmCurve, default_rhythm, from_profile, suggest_arm_windows};
pub use schedule::{CronRule, parse_rule};