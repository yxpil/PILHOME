//! 规则变异:遗传式规则进化。
//!
//! 每条规则携带"适应度";周期性地按适应度选择、交叉与变异,
//! 使规则库自发适应家庭行为变化(成长),并在扰动中探索新策略(变异)。

use serde::{Deserialize, Serialize};

/// 规则作用种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    /// 布防时段规则
    Arm,
    /// 撤防时段规则
    Disarm,
    /// 巡检规则
    Patrol,
    /// 告警抑制规则
    Suppress,
}

/// 一条可进化的规则基因。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleGene {
    /// 规则 ID。
    pub id: String,
    /// 作用种类。
    pub kind: RuleKind,
    /// 起始小时(0~23)。
    pub start_hour: u8,
    /// 结束小时(0~23)。
    pub end_hour: u8,
    /// 适用设备模式(如 `contact_*`;`*` 表示全部)。
    pub device_pattern: String,
    /// 适应度(0.0 ~ 1.0,越高越好)。
    pub fitness: f32,
    /// 生效代数。
    pub generation: u32,
}

impl RuleGene {
    /// 构造规则基因。
    pub fn new(id: impl Into<String>, kind: RuleKind, start_hour: u8, end_hour: u8) -> Self {
        Self {
            id: id.into(),
            kind,
            start_hour: start_hour.min(23),
            end_hour: end_hour.min(23),
            device_pattern: "*".to_string(),
            fitness: 0.5,
            generation: 0,
        }
    }

    /// 规则是否覆盖给定小时。
    pub fn covers(&self, hour: u8) -> bool {
        self.start_hour <= hour && hour <= self.end_hour
    }
}

/// 规则进化器。
#[derive(Debug)]
pub struct Evolver {
    /// 当前种群。
    population: Vec<RuleGene>,
    /// 代数计数。
    generation: u32,
    /// 变异概率(0.0 ~ 1.0)。
    pub mutation_rate: f32,
}

impl Default for Evolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Evolver {
    /// 创建进化器。
    pub fn new() -> Self {
        Self { population: Vec::new(), generation: 0, mutation_rate: 0.15 }
    }

    /// 注入初始种群。
    pub fn seed(&mut self, genes: Vec<RuleGene>) {
        self.population = genes;
    }

    /// 当前种群。
    pub fn population(&self) -> &[RuleGene] {
        &self.population
    }

    /// 记录一次命中(提升适应度)或误报(降低适应度)。
    pub fn reward(&mut self, id: &str, delta: f32) {
        if let Some(g) = self.population.iter_mut().find(|g| g.id == id) {
            g.fitness = (g.fitness + delta).clamp(0.0, 1.0);
        }
    }

    /// 执行一代进化:
    /// 1. 按适应度降序排序;
    /// 2. 保留头部(精英);
    /// 3. 对尾部按变异率扰动参数(小时漂移 ±2、模式切换)。
    pub fn evolve(&mut self) -> usize {
        if self.population.is_empty() {
            return 0;
        }
        self.generation += 1;
        self.population.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap_or(std::cmp::Ordering::Equal));

        let keep = (self.population.len() / 2).max(1);
        let mut next: Vec<RuleGene> = self.population[..keep].to_vec();

        for gene in &self.population[keep..] {
            let mut child = gene.clone();
            child.generation = self.generation;
            // 变异:小时漂移 ±2。
            let drift = (rand01() * 4.0) as i8 - 2;
            child.start_hour = shift_hour(child.start_hour, drift);
            child.end_hour = shift_hour(child.end_hour, drift);
            if rand01() < self.mutation_rate {
                child.kind = match child.kind {
                    RuleKind::Arm => RuleKind::Disarm,
                    RuleKind::Disarm => RuleKind::Arm,
                    other => other,
                };
            }
            child.fitness *= 0.9; // 变异后代略降适应度,等待验证。
            next.push(child);
        }
        self.population = next;
        self.population.len()
    }

    /// 当前代数。
    pub fn generation(&self) -> u32 {
        self.generation
    }
}

/// 伪随机 0~1(线性同余,足够进化扰动使用)。
fn rand01() -> f32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_nanos()).unwrap_or(0);
    ((t % 100_000) as f32) / 100_000.0
}

/// 小时漂移并回卷。
fn shift_hour(hour: u8, drift: i8) -> u8 {
    (((hour as i16) + drift as i16).rem_euclid(24)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evolve_grows_generation() {
        let mut evolver = Evolver::new();
        evolver.seed(vec![
            RuleGene::new("a", RuleKind::Arm, 23, 6),
            RuleGene::new("b", RuleKind::Disarm, 7, 8),
            RuleGene::new("c", RuleKind::Patrol, 12, 13),
        ]);
        let n = evolver.evolve();
        assert!(n >= 3, "种群应保留或增长");
        assert_eq!(evolver.generation(), 1);
    }

    #[test]
    fn reward_updates_fitness() {
        let mut evolver = Evolver::new();
        evolver.seed(vec![RuleGene::new("a", RuleKind::Arm, 23, 6)]);
        let before = evolver.population()[0].fitness;
        evolver.reward("a", 0.2);
        let after = evolver.population()[0].fitness;
        assert!(after > before);
        assert!(after <= 1.0);
    }
}
