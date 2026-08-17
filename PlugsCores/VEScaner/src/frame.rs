//! 帧分析:灰度帧差运动检测(视频审计的输入基础)。

use serde::{Deserialize, Serialize};

/// 运动检测报告。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionReport {
    /// 变化像素占比(0.0 ~ 1.0)。
    pub changed_ratio: f32,
    /// 是否判定为运动。
    pub moving: bool,
    /// 运动区域包围盒(未运动时为空)。
    pub bbox: Option<BBox>,
}

/// 图像区域包围盒。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BBox {
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
}

/// 帧差检测器:比较相邻灰度帧,统计差异像素。
///
/// 帧格式:灰度图,每像素一字节(0~255),行优先排列。
pub struct FrameDiff {
    /// 变化像素判定阈值(0~255)。
    pub diff_threshold: u8,
    /// 运动判定比例阈值(0.0 ~ 1.0)。
    pub ratio_threshold: f32,
}

impl Default for FrameDiff {
    fn default() -> Self {
        Self { diff_threshold: 24, ratio_threshold: 0.01 }
    }
}

impl FrameDiff {
    /// 检测两帧之间的运动;输入不合法时返回 None。
    pub fn detect(&self, prev: &[u8], curr: &[u8], width: usize, height: usize) -> Option<MotionReport> {
        if prev.len() != curr.len() || prev.len() != width * height || width == 0 || height == 0 {
            return None;
        }
        let total = prev.len();
        let mut changed = 0usize;
        let mut min_x = width;
        let mut min_y = height;
        let mut max_x = 0usize;
        let mut max_y = 0usize;

        for (i, (&a, &b)) in prev.iter().zip(curr.iter()).enumerate() {
            if a.abs_diff(b) > self.diff_threshold {
                changed += 1;
                let x = i % width;
                let y = i / width;
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
            }
        }

        let ratio = changed as f32 / total as f32;
        let moving = ratio >= self.ratio_threshold;
        let bbox = if moving {
            Some(BBox { x: min_x, y: min_y, w: max_x - min_x + 1, h: max_y - min_y + 1 })
        } else {
            None
        };
        Some(MotionReport { changed_ratio: ratio, moving, bbox })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_frames_no_motion() {
        let frame = vec![100u8; 64];
        let report = FrameDiff::default().detect(&frame, &frame, 8, 8).expect("valid input");
        assert!(!report.moving);
        assert_eq!(report.changed_ratio, 0.0);
        assert!(report.bbox.is_none());
    }

    #[test]
    fn changed_frames_detect_motion() {
        let prev = vec![100u8; 64];
        let mut curr = prev.clone();
        for px in curr.iter_mut().skip(10).take(20) {
            *px = 200; // 制造明显差异
        }
        let report = FrameDiff::default().detect(&prev, &curr, 8, 8).expect("valid input");
        assert!(report.moving);
        assert!(report.bbox.is_some());
    }

    #[test]
    fn invalid_size_returns_none() {
        assert!(FrameDiff::default().detect(&[1u8; 4], &[1u8; 5], 2, 2).is_none());
    }
}
