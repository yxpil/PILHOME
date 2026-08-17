//! AI 接入层(OpenAI 兼容 API)。
//!
//! 支持任意 OpenAI 兼容端点(本地可配 cidaiji.com 等):
//! - [`AiClient::chat`]      日常聊天 / 工具决策
//! - [`AiClient::analyze`]   传感器阈值事件告知 AI 分析
//! - [`AiClient::compress`]  对话上下文压缩
//! - [`AiClient::vision`]    图片分析(视觉模型)
//! - [`AiClient::review`]    定期自我审查
//!
//! **密钥安全**:`api_key` 优先读环境变量 `AI_API_KEY`,其次 config;
//! 密钥绝不写入任何随仓库提交的文件。

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

/// 一条对话消息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMsg {
    pub role: String,
    pub content: String,
}

/// AI 客户端。
#[derive(Debug, Clone)]
pub struct AiClient {
    base_url: String,
    api_key: String,
    model: String,
    vision_model: String,
}

impl AiClient {
    /// 创建客户端。api_key 为空时回退环境变量 `AI_API_KEY`。
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>, model: impl Into<String>, vision_model: impl Into<String>) -> Self {
        let api_key = api_key.into();
        let api_key = if api_key.is_empty() {
            std::env::var("AI_API_KEY").unwrap_or_default()
        } else {
            api_key
        };
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key,
            model: model.into(),
            vision_model: vision_model.into(),
        }
    }

    /// 是否已配置(有 key 与地址)。
    pub fn configured(&self) -> bool {
        !self.base_url.is_empty() && !self.api_key.is_empty()
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// 基础聊天补全。
    fn complete(&self, model: &str, messages: &[ChatMsg], max_tokens: u32) -> Result<String, String> {
        let resp = ureq::post(&self.endpoint("/chat/completions"))
            .timeout(Duration::from_secs(120))
            .set("Content-Type", "application/json")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .send_json(json!({
                "model": model,
                "messages": messages,
                "max_tokens": max_tokens,
                "temperature": 0.7,
            }))
            .map_err(|e| format!("AI 请求失败:{e}"))?;
        let body: Value = resp.into_json().map_err(|e| format!("AI 响应解析失败:{e}"))?;
        body["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| format!("AI 响应缺少内容:{body}"))
    }

    /// 日常聊天。
    pub fn chat(&self, messages: &[ChatMsg]) -> Result<String, String> {
        self.complete(&self.model, messages, 2048)
    }

    /// 事件分析:传感器触发阈值等事件告知 AI,返回研判。
    pub fn analyze(&self, event: &Value, context: &str) -> Result<String, String> {
        let msg = ChatMsg {
            role: "user".into(),
            content: format!(
                "你是家庭安防边缘网关的 AI 研判助手。请分析以下事件,判断风险等级(低/中/高),\
                 给出简短结论与建议动作(200 字内)。\n\n事件: {}\n\n上下文: {}",
                serde_json::to_string_pretty(event).unwrap_or_default(),
                context
            ),
        };
        self.complete(&self.model, &[msg], 600)
    }

    /// 上下文压缩:把长对话摘要为系统提示,保留关键信息。
    pub fn compress(&self, messages: &[ChatMsg]) -> Result<String, String> {
        let transcript: String = messages
            .iter()
            .map(|m| format!("[{}] {}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n");
        let msg = ChatMsg {
            role: "user".into(),
            content: format!(
                "请压缩以下多轮对话为一份结构化摘要,保留:用户身份/偏好、已执行的设备操作、\
                 关键结论与待办、未解决的问题。用中文,300 字内。\n\n{}",
                &transcript[..transcript.len().min(8000)]
            ),
        };
        self.complete(&self.model, &[msg], 500)
    }

    /// 图片分析(视觉模型)。
    pub fn vision(&self, prompt: &str, image_url: &str, image_b64: Option<&str>) -> Result<String, String> {
        let mut content = json!([
            { "type": "text", "text": prompt }
        ]);
        if let Some(b64) = image_b64 {
            content.as_array_mut().unwrap().push(json!({
                "type": "image_url",
                "image_url": { "url": format!("data:image/jpeg;base64,{b64}") },
            }));
        } else {
            content.as_array_mut().unwrap().push(json!({
                "type": "image_url",
                "image_url": { "url": image_url },
            }));
        }
        let model = if self.vision_model.is_empty() { &self.model } else { &self.vision_model };
        let resp = ureq::post(&self.endpoint("/chat/completions"))
            .timeout(Duration::from_secs(180))
            .set("Content-Type", "application/json")
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .send_json(json!({
                "model": model,
                "messages": [ { "role": "user", "content": content } ],
                "max_tokens": 1024,
            }))
            .map_err(|e| format!("AI 视觉请求失败:{e}"))?;
        let body: Value = resp.into_json().map_err(|e| format!("AI 响应解析失败:{e}"))?;
        body["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| format!("AI 响应缺少内容:{body}"))
    }

    /// 自我审查:给定系统运行摘要,输出改进建议。
    pub fn review(&self, context: &str) -> Result<String, String> {
        let msg = ChatMsg {
            role: "user".into(),
            content: format!(
                "你是家庭智能网关的运维审计 AI。请审查以下运行快照,列出:风险点、配置问题、\
                 自动化改进建议、安全隐患。用中文,分条列出,300 字内。\n\n{}",
                &context[..context.len().min(8000)]
            ),
        };
        self.complete(&self.model, &[msg], 600)
    }
}

#[allow(dead_code)] // 预留:对话编排辅助
/// 构建"系统提示 + 历史 + 新消息"。
pub fn system_messages(system: &str, history: &[ChatMsg]) -> Vec<ChatMsg> {
    let mut msgs = vec![ChatMsg { role: "system".into(), content: system.into() }];
    msgs.extend_from_slice(history);
    msgs
}