//! 服务全局状态:聚合 Core 协调器与全部 PlugsCores 模块。

use crate::events::EventLog;
use crate::auth::TokenStore;
use crate::ai::AiClient;
use crate::storage::{MysqlSync, SqliteStore};
use crate::tools::ToolRegistry;
use pilhome_atogrowup::BayesClassifier;
use crate::ha_bridge::HaBridge;
use crate::persistence::Persistence;
use pilhome_atogrowup::{Evolver, Profile};
use pilhome_autotime::CronRule;
use pilhome_core::Coordinator;
use pilhome_handmodel::Controller;
use pilhome_netlinker::{Connector, DeviceRegistry};
use pilhome_netscanear::{DiscoveryReport, ScanConfig};
use pilhome_selflookup::AuditLog;
use pilhome_sideagent::ModelRegistry;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// 当前 Unix 秒。
pub fn now_secs() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// 服务全局状态。
pub struct AppState {
    /// Core 总协调器(模块注册表 + 消息总线)。
    pub coord: Coordinator,
    /// NetLinker:设备台账。
    pub devices: DeviceRegistry,
    /// NetLinker:连接状态。
    pub connector: Connector,
    /// 统一事件日志。
    pub events: EventLog,
    /// ATOGrowUP:行为画像。
    pub profile: Mutex<Profile>,
    /// ATOGrowUP:规则进化器。
    pub evolver: Mutex<Evolver>,
    /// SelflookUP:审计日志。
    pub audit: AuditLog,
    /// SideAgent:边缘模型注册表。
    pub models: ModelRegistry,
    /// HandModel:模块控制器。
    pub controller: Controller,
    /// AUTOTIME:定时规则。
    pub schedules: RwLock<Vec<CronRule>>,
    /// 网络扫描配置。
    pub scan_cfg: RwLock<ScanConfig>,
    /// 最近一次扫描报告。
    pub last_scan: Mutex<Option<DiscoveryReport>>,
    /// 网关启动时间。
    pub started_at: u64,
    /// 持久化层(事件/审计/画像快照)。
    pub persistence: Persistence,
    /// HA 双向桥(全屋设备接管)。
    pub ha: Arc<HaBridge>,
    /// 自动化规则(内置自动化引擎)。
    pub automations: RwLock<Vec<crate::automation::AutomationRule>>,
    /// MQTT 命令通道(自动化/外部 API 下发)。
    pub mqtt_tx: Mutex<Option<tokio::sync::mpsc::Sender<crate::mqtt::MqttCommand>>>,
    /// API 令牌库。
    pub tokens: TokenStore,
    /// 鉴权是否启用。
    pub auth_enabled: bool,
    /// SQLite 事件库。
    pub sqlite: SqliteStore,
    /// MySQL 同步器。
    pub mysql: MysqlSync,
    /// AI 客户端。
    pub ai: AiClient,
    /// 工具注册表(内置 + JS)。
    pub tools: ToolRegistry,
    /// 朴素贝叶斯习惯分类器。
    pub bayes: Mutex<BayesClassifier>,
    /// 工具 KV 状态。
    pub tool_state: RwLock<crate::tools::ToolState>,
    /// TTS 服务地址。
    pub tts_url: String,
    /// ComfyUI 地址。
    pub comfyui_url: String,
    /// 数据目录(tts/sd 输出)。
    pub data_dir: std::path::PathBuf,
    /// AI 服务地址(展示用)。
    pub ai_base_url: String,
    /// AI 模型(展示用)。
    pub ai_model: String,
    /// 视觉模型(展示用)。
    pub ai_vision_model: String,
}

impl AppState {
    /// 构建全局状态(Coordinator 与各模块实例)。
    pub fn new(
        event_capacity: usize,
        data_dir: impl Into<std::path::PathBuf>,
        ha: Arc<HaBridge>,
        api_tokens: &[String],
        auth_enabled: bool,
        mysql_url: &str,
        ai_base_url: &str,
        ai_model: &str,
        ai_vision_model: &str,
        ai_api_key: &str,
        tts_url: &str,
        comfyui_url: &str,
    ) -> Arc<Self> {
        let data_dir2: std::path::PathBuf = data_dir.into();
        let coord = Coordinator::new();
        let state = Arc::new(Self {
            coord,
            devices: DeviceRegistry::new(),
            connector: Connector::new(),
            events: EventLog::new(event_capacity),
            profile: Mutex::new(Profile::default()),
            evolver: Mutex::new(Evolver::new()),
            audit: AuditLog::new(4096),
            models: ModelRegistry::new(),
            controller: Controller::new(),
            schedules: RwLock::new(Vec::new()),
            scan_cfg: RwLock::new(ScanConfig::default()),
            last_scan: Mutex::new(None),
            started_at: now_secs(),
            persistence: Persistence::new(&data_dir2),
            ha,
            automations: RwLock::new(Vec::new()),
            mqtt_tx: Mutex::new(None),
            tokens: TokenStore::new(api_tokens),
            auth_enabled,
            sqlite: SqliteStore::open(data_dir2.join("pilhome.db")),
            mysql: MysqlSync::new(mysql_url.to_string()),
            ai: AiClient::new(ai_base_url, ai_api_key, ai_model, ai_vision_model),
            tools: ToolRegistry::new(),
            bayes: Mutex::new(BayesClassifier::new()),
            tool_state: RwLock::new(crate::tools::ToolState::new()),
            tts_url: tts_url.to_string(),
            comfyui_url: comfyui_url.to_string(),
            data_dir: data_dir2.clone(),
            ai_base_url: ai_base_url.to_string(),
            ai_model: ai_model.to_string(),
            ai_vision_model: ai_vision_model.to_string(),
        });
        state
    }

    /// 运行时长(秒)。
    pub fn uptime_secs(&self) -> u64 {
        now_secs().saturating_sub(self.started_at)
    }
}