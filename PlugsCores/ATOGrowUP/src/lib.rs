//! ATOGrowUP —— 自发成长和变异程序
//!
//! 系统随运行"自发成长":持续记录设备行为形成画像,
//! 规则库按适应度"变异"进化,自动逼近家庭真实作息:
//! - [`profile`] 行为画像(设备 × 时段活跃期望)
//! - [`evolve`] 规则变异(遗传式规则进化)

pub mod bayes;
pub mod evolve;
pub mod profile;

pub use evolve::{Evolver, RuleGene, RuleKind};
pub use bayes::BayesClassifier;
pub use profile::{EventRecord, Profile, Suggestion};