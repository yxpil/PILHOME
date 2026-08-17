//! 昼夜节律:24 小时活跃曲线与布防时段推荐。

use serde::{Deserialize, Serialize};

/// 24 小时节律曲线(每小时活跃期望,0.0 ~ 1.0)。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RhythmCurve(pub [f32; 24]);

impl RhythmCurve {
    /// 当前小时的活动水平。
    pub fn level(&self, hour: u8) -> f32 {
        self.0[(hour % 24) as usize]
    }

    /// 全时段均值。
    pub fn mean(&self) -> f32 {
        self.0.iter().sum::<f32>() / 24.0
    }

    /// 归一化到 0~1。
    pub fn normalized(&self) -> Self {
        let max = self.0.iter().cloned().fold(0.0f32, f32::max).max(1e-6);
        let mut out = [0.0f32; 24];
        for (i, v) in self.0.iter().enumerate() {
            out[i] = v / max;
        }
        RhythmCurve(out)
    }
}

/// 默认作息模板:日间活跃、夜间安静(适用于典型家庭)。
pub fn default_rhythm() -> RhythmCurve {
    RhythmCurve([
        0.05, 0.03, 0.02, 0.02, 0.03, 0.06, // 00-05 深睡
        0.20, 0.45, 0.70, 0.55, 0.45, 0.50, // 06-11 起床/上午
        0.55, 0.50, 0.45, 0.40, 0.50, 0.65, // 12-17 午后/傍晚
        0.80, 0.75, 0.55, 0.35, 0.15, 0.08, // 18-23 晚间/入睡
    ])
}

/// 从行为画像(ATOGrowUP 输出)构建归一化节律曲线。
pub fn from_profile(activity: [f32; 24]) -> RhythmCurve {
    RhythmCurve(activity).normalized()
}

/// 推荐布防时段:连续低活跃窗口(活跃度低于阈值)。
///
/// 返回形如 `[(22, 6)]` 的跨日窗口列表(小时区间,开区间右端)。
pub fn suggest_arm_windows(curve: &RhythmCurve, threshold: f32) -> Vec<(u8, u8)> {
    let low: Vec<bool> = (0..24).map(|h| curve.level(h as u8) < threshold).collect();
    let mut windows = Vec::new();
    let mut start: Option<u8> = None;

    // 从 0 点向后扫描,处理跨日窗口。
    for h in 0..24u8 {
        let is_low = low[h as usize];
        match (is_low, start) {
            (true, None) => start = Some(h),
            (false, Some(s)) => {
                windows.push((s, h));
                start = None;
            }
            _ => {}
        }
    }
    // 末尾的低活跃段与开头的低活跃段合并(跨日)。
    if let Some(s) = start {
        let mut head_end = 24u8;
        for h in 0..24u8 {
            if !low[h as usize] {
                head_end = h;
                break;
            }
        }
        windows.push((s, head_end));
    }
    windows
}