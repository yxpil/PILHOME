//! 推理运行时:统一预测接口 + 规则引擎实现。

use serde::{Deserialize, Serialize};

/// 一次预测结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// 类别 / 标签。
    pub label: String,
    /// 置信度(0.0 ~ 1.0)。
    pub confidence: f32,
    /// 检测框(目标检测类模型;其余为空)。
    pub bbox: Option<BBox>,
}

/// 检测框(归一化坐标 0.0 ~ 1.0)。
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BBox {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

/// 推理运行时抽象:所有边缘模型统一入口。
///
/// 当前提供基于特征模板的 [`RuleRuntime`];
/// ONNX / TFLite 后端可按相同接口接入(加载权重、GPU/CPU 推理)。
pub trait ModelRuntime: Send + Sync {
    /// 运行一次推理,输入 8 维特征向量,输出按置信度降序的预测。
    fn run(&self, features: &[f32; 8]) -> Vec<Prediction>;
}

/// 内置模板(握拳 / 张手 / OK / 比耶,8 维手指开合特征)。
const TEMPLATES: &[(&str, [f32; 8])] = &[
    ("fist",  [0.05, 0.06, 0.04, 0.05, 0.05, 0.06, 0.05, 0.04]),
    ("palm",  [0.95, 0.92, 0.94, 0.90, 0.91, 0.93, 0.90, 0.92]),
    ("ok",    [0.12, 0.90, 0.88, 0.10, 0.11, 0.89, 0.87, 0.09]),
    ("peace", [0.88, 0.85, 0.10, 0.09, 0.87, 0.84, 0.11, 0.90]),
];

/// 规则引擎:基于欧氏距离的模板匹配,零依赖、毫秒级。
#[derive(Debug, Clone, Copy, Default)]
pub struct RuleRuntime;

impl RuleRuntime {
    fn distance(a: &[f32; 8], b: &[f32; 8]) -> f32 {
        a.iter().zip(b.iter()).map(|(x, y)| (x - y) * (x - y)).sum::<f32>().sqrt()
    }
}

impl ModelRuntime for RuleRuntime {
    fn run(&self, features: &[f32; 8]) -> Vec<Prediction> {
        let mut out: Vec<Prediction> = TEMPLATES
            .iter()
            .map(|(label, tmpl)| {
                let d = Self::distance(features, tmpl);
                Prediction {
                    label: (*label).to_string(),
                    confidence: (1.0 - d / 2.0).clamp(0.0, 1.0),
                    bbox: None,
                }
            })
            .collect();
        out.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        out
    }
}

/// 便捷:用规则引擎分类。
pub fn predict_rule(features: &[f32; 8]) -> Vec<Prediction> {
    RuleRuntime.run(features)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fist_feature_classifies_as_fist() {
        let features = [0.05, 0.06, 0.04, 0.05, 0.05, 0.06, 0.05, 0.04];
        let top = predict_rule(&features);
        assert!(!top.is_empty());
        assert_eq!(top[0].label, "fist");
        assert!(top[0].confidence > 0.9, "应接近模板,置信度应很高");
    }

    #[test]
    fn palm_feature_classifies_as_palm() {
        let features = [0.95, 0.92, 0.94, 0.90, 0.91, 0.93, 0.90, 0.92];
        let top = predict_rule(&features);
        assert_eq!(top[0].label, "palm");
    }
}
