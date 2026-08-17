//! 朴素贝叶斯分类器:学习用户习惯,为自动动作放行提供置信度。
//!
//! 特征 = (键, 值) 列表(如 `action=light.on`、`hour=22`、`weekday=mon`);
//! 训练样本来自用户历史操作(正样本:用户手动执行;
//! 负样本:自动化执行后被用户 5 分钟内纠正)。拉普拉斯平滑防零概率。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 朴素贝叶斯二分类器(正类 = 用户接受该动作)。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BayesClassifier {
    /// 正/负样本数。
    pub total_pos: usize,
    pub total_neg: usize,
    /// 特征条件计数: `"k=v" -> (正计数, 负计数)`。
    cond: HashMap<String, (usize, usize)>,
}

/// 单条特征。
pub type Feature<'a> = (&'a str, &'a str);

impl BayesClassifier {
    /// 创建空分类器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 训练一条样本。
    pub fn train(&mut self, features: &[Feature], positive: bool) {
        if positive {
            self.total_pos += 1;
        } else {
            self.total_neg += 1;
        }
        for (k, v) in features {
            let entry = self.cond.entry(format!("{k}={v}")).or_insert((0, 0));
            if positive {
                entry.0 += 1;
            } else {
                entry.1 += 1;
            }
        }
    }

    /// 批量训练。
    pub fn train_batch(&mut self, samples: &[(Vec<Feature>, bool)]) {
        for (feats, label) in samples {
            self.train(feats, *label);
        }
    }

    /// 预测正类(用户接受)概率,0.0 ~ 1.0。
    ///
    /// 使用对数空间累加避免下溢,拉普拉斯平滑(+1)。
    pub fn predict(&self, features: &[Feature]) -> f64 {
        if self.total_pos + self.total_neg == 0 {
            return 0.5; // 无样本时中立。
        }
        let p_pos = self.total_pos as f64 / (self.total_pos + self.total_neg) as f64;
        let p_neg = 1.0 - p_pos;

        // 对数似然。
        let mut log_pos = p_pos.ln();
        let mut log_neg = p_neg.ln();
        for (k, v) in features {
            let key = format!("{k}={v}");
            let (c_pos, c_neg) = self.cond.get(&key).copied().unwrap_or((0, 0));
            // 拉普拉斯平滑:分子 +1,分母 +2(两类)。
            let p_f_pos = (c_pos as f64 + 1.0) / (self.total_pos as f64 + 2.0);
            let p_f_neg = (c_neg as f64 + 1.0) / (self.total_neg as f64 + 2.0);
            log_pos += p_f_pos.ln();
            log_neg += p_f_neg.ln();
        }
        // softmax 归一化。
        let max = log_pos.max(log_neg);
        let exp_pos = (log_pos - max).exp();
        let exp_neg = (log_neg - max).exp();
        exp_pos / (exp_pos + exp_neg)
    }

    /// 预测结果包装(含样本量)。
    pub fn predict_with_meta(&self, features: &[Feature]) -> Prediction {
        Prediction {
            probability: self.predict(features),
            samples: self.total_pos + self.total_neg,
            positive_samples: self.total_pos,
        }
    }

    /// 已训练特征数。
    pub fn feature_count(&self) -> usize {
        self.cond.len()
    }
}

/// 一次预测的结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// 用户接受概率(0.0~1.0)。
    pub probability: f64,
    /// 总样本数。
    pub samples: usize,
    /// 正样本数。
    pub positive_samples: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 构造训练集:用户习惯"晚上 22 点手动关灯"(正),白天从不关灯(负)。
    fn trained() -> BayesClassifier {
        let mut b = BayesClassifier::new();
        // 10 条晚上手动关灯。
        for _ in 0..10 {
            b.train(&[("action", "light.off"), ("hour", "22"), ("weekday", "mon")], true);
        }
        // 8 条白天系统误触发被纠正(负)。
        for _ in 0..8 {
            b.train(&[("action", "light.off"), ("hour", "14"), ("weekday", "tue")], false);
        }
        b
    }

    #[test]
    fn predicts_user_habit() {
        let b = trained();
        let p_night = b.predict(&[("action", "light.off"), ("hour", "22")]);
        let p_day = b.predict(&[("action", "light.off"), ("hour", "14")]);
        assert!(p_night > 0.7, "夜间置信度应高,实际 {p_night}");
        assert!(p_day < 0.3, "白天置信度应低,实际 {p_day}");
    }

    #[test]
    fn neutral_without_data() {
        let b = BayesClassifier::new();
        assert_eq!(b.predict(&[("action", "x")]), 0.5);
    }

    #[test]
    fn train_increases_confidence() {
        let mut b = BayesClassifier::new();
        b.train(&[("device", "washer"), ("hour", "20")], true);
        b.train(&[("device", "washer"), ("hour", "20")], true);
        b.train(&[("device", "washer"), ("hour", "20")], false);
        let p = b.predict(&[("device", "washer"), ("hour", "20")]);
        assert!(p > 0.5);
    }
}