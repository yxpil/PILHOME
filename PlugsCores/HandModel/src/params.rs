//! 模块参数定义与取值。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// 参数值(强类型,静态检查友好)。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum ParamValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    Text(String),
}

impl ParamValue {
    /// 参数类型名。
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::Bool(_) => "bool",
            Self::Text(_) => "text",
        }
    }
}

/// 参数定义(供 WebUI 生成表单)。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDef {
    /// 参数键。
    pub key: String,
    /// 说明。
    pub desc: String,
    /// 默认值。
    pub default: ParamValue,
    /// 取值范围(可空)。
    pub range: Option<(f64, f64)>,
}

impl ParamDef {
    /// 构造参数定义。
    pub fn new(key: impl Into<String>, desc: impl Into<String>, default: ParamValue) -> Self {
        Self { key: key.into(), desc: desc.into(), default, range: None }
    }
}

/// 一组模块参数。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ParamSet {
    inner: BTreeMap<String, ParamValue>,
}

impl ParamSet {
    /// 空参数集。
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置参数。
    pub fn set(&mut self, key: impl Into<String>, value: ParamValue) {
        self.inner.insert(key.into(), value);
    }

    /// 读取参数。
    pub fn get(&self, key: &str) -> Option<&ParamValue> {
        self.inner.get(key)
    }

    /// 全部参数快照。
    pub fn snapshot(&self) -> BTreeMap<String, ParamValue> {
        self.inner.clone()
    }
}