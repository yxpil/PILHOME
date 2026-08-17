//! 服务配置:TOML 文件优先,缺省时使用内置默认值。

use pilhome_netscanear::ScanConfig;
use pilhome_vescaner::ProbeConfig;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// MQTT 接入配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MqttConfig {
    /// Broker 地址,如 `127.0.0.1:1883`。
    pub host: String,
    /// 客户端 ID。
    pub client_id: String,
    /// 订阅主题前缀。
    pub topic_prefix: String,
}

impl Default for MqttConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1:1883".to_string(),
            client_id: format!("pilhome-gateway-{}", std::process::id()),
            topic_prefix: "pilhome".to_string(),
        }
    }
}

/// Home Assistant 集成配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct HaConfig {
    /// 是否启用 MQTT Discovery。
    pub discovery: bool,
    /// Discovery 主题前缀。
    pub discovery_prefix: String,
    /// HA WebSocket 地址(如 `ws://127.0.0.1:8123/api/websocket`;空 = 不启用全屋接管)。
    pub ws_url: String,
    /// HA 长效访问令牌。
    pub token: String,
    /// 全量状态同步间隔(秒)。
    pub sync_interval_secs: u64,
}

impl Default for HaConfig {
    fn default() -> Self {
        Self { discovery: true, discovery_prefix: "homeassistant".to_string(), ws_url: String::new(), token: String::new(), sync_interval_secs: 60 }
    }
}

/// 周期任务配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TaskConfig {
    /// 网络扫描间隔(秒)。
    pub scan_interval_secs: u64,
    /// 定时规则检查间隔(秒)。
    pub cron_tick_secs: u64,
    /// 规则进化间隔(秒)。
    pub evolve_interval_secs: u64,
}

impl Default for TaskConfig {
    fn default() -> Self {
        Self { scan_interval_secs: 600, cron_tick_secs: 30, evolve_interval_secs: 3600 }
    }
}

/// 顶层配置。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// HTTP 监听地址。
    pub listen: String,
    /// WebUI 静态文件目录。
    pub webui_dir: PathBuf,
    /// 数据目录。
    pub data_dir: PathBuf,
    /// 设备离线判定阈值(秒)。
    pub offline_after_secs: u64,
    /// 事件日志保留条数。
    pub event_capacity: usize,
    /// MQTT 配置。
    pub mqtt: MqttConfig,
    /// Home Assistant 配置。
    pub ha: HaConfig,
    /// 网络扫描配置。
    pub scan: ScanConfig,
    /// 视频设备探测配置。
    pub probe: ProbeConfig,
    /// 周期任务配置。
    pub tasks: TaskConfig,
    /// 告警 Webhook 地址(空 = 关闭;仅 http)。
    pub webhook_url: String,
    /// 数据快照间隔(秒)。
    pub snapshot_interval_secs: u64,
    /// 是否启用 API Token 鉴权(默认关闭)。
    pub auth_enabled: bool,
    /// 预置 API 令牌(可选)。
    pub api_tokens: Vec<String>,
    /// 远程 MySQL 连接串(如 `mysql://user:pass@host:3306/pilhome`;空 = 关闭)。
    pub mysql_url: String,
    /// AI 服务:OpenAI 兼容 base_url(如 `https://cidaiji.com/v1`)。
    pub ai_base_url: String,
    /// AI 模型名。
    pub ai_model: String,
    /// 视觉模型名(空 = 复用 ai_model)。
    pub ai_vision_model: String,
    /// AI API 密钥(留空读环境变量 AI_API_KEY;绝不写入仓库)。
    pub ai_api_key: String,
    /// TTS 服务地址(如 `http://127.0.0.1:5000/tts`;空 = 关闭)。
    pub tts_url: String,
    /// ComfyUI 地址(如 `http://127.0.0.1:8188`;空 = 关闭)。
    pub comfyui_url: String,
    /// AI 自我审查间隔(秒,默认 6 小时)。
    pub ai_review_secs: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            listen: "127.0.0.1:8080".to_string(),
            webui_dir: PathBuf::from("../WebUI/dist"),
            data_dir: PathBuf::from("data"),
            offline_after_secs: 120,
            event_capacity: 4096,
            mqtt: MqttConfig::default(),
            ha: HaConfig::default(),
            scan: ScanConfig::default(),
            probe: ProbeConfig::default(),
            tasks: TaskConfig::default(),
            webhook_url: String::new(),
            snapshot_interval_secs: 300,
            auth_enabled: false,
            api_tokens: Vec::new(),
            mysql_url: String::new(),
            ai_base_url: String::new(),
            ai_model: "default".to_string(),
            ai_vision_model: String::new(),
            ai_api_key: String::new(),
            tts_url: String::new(),
            comfyui_url: String::new(),
            ai_review_secs: 21600,
        }
    }
}

impl Config {
    /// 从 TOML 文件加载;文件不存在时返回默认配置。
    pub fn load(path: Option<&Path>) -> Self {
        let Some(path) = path else { return Self::default() };
        let Ok(text) = std::fs::read_to_string(path) else {
            tracing::warn!("配置文件不存在:{path:?},使用默认配置");
            return Self::default();
        };
        match toml::from_str(&text) {
            Ok(cfg) => cfg,
            Err(err) => {
                tracing::error!("配置文件解析失败:{err};使用默认配置");
                Self::default()
            }
        }
    }
}