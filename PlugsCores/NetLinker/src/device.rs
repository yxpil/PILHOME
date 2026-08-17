//! 统一设备模型。

use serde::{Deserialize, Serialize};
use std::fmt;

/// 设备种类,决定默认图标、接入方式与监控策略。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceKind {
    /// 门磁 / 窗磁等开合传感器
    Contact,
    /// 人体红外、毫米波存在传感器
    Presence,
    /// 门锁 / 指纹锁
    Lock,
    /// 网关 / 中继设备
    Gateway,
    /// 通用传感器(温湿度、烟雾、水浸等)
    Sensor,
    /// 通用执行器(警报器等)
    Actuator,
    /// 灯(可调光/调色)
    Light,
    /// 开关 / 插座
    Switch,
    /// 温控设备(空调 / 地暖 / 热水器)
    Climate,
    /// 窗帘 / 百叶 / 车库门
    Cover,
    /// 风扇 / 新风
    Fan,
    /// 警报器 / 警笛
    Siren,
    /// 摄像头 / 视频设备
    Camera,
    /// 未分类设备
    Unknown,
}

impl fmt::Display for DeviceKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl DeviceKind {
    /// 稳定的字符串标识(与序列化保持一致)。
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Contact => "contact",
            Self::Presence => "presence",
            Self::Camera => "camera",
            Self::Lock => "lock",
            Self::Gateway => "gateway",
            Self::Sensor => "sensor",
            Self::Actuator => "actuator",
            Self::Light => "light",
            Self::Switch => "switch",
            Self::Climate => "climate",
            Self::Cover => "cover",
            Self::Fan => "fan",
            Self::Siren => "siren",
            Self::Unknown => "unknown",
        }
    }
}

/// 设备在线状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceState {
    /// 在线且正常
    Online,
    /// 在线但上报异常(如电量低、通信抖动)
    Degraded,
    /// 离线
    Offline,
}

/// 统一设备实体。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// 全局唯一设备标识(如 `contact_front_door`)。
    pub id: String,
    /// 展示名称(如 `入户门磁`)。
    pub name: String,
    /// 设备种类。
    pub kind: DeviceKind,
    /// 接入地址(MQTT topic / IP:端口 / RTSP URL)。
    pub address: String,
    /// 当前状态。
    pub state: DeviceState,
    /// 最后心跳时间(Unix 秒)。
    pub last_seen: u64,
    /// 附加属性(厂商、固件版本、电量等)。
    pub attrs: std::collections::BTreeMap<String, String>,
    /// 是否白名单设备(白名单外设备不参与告警,仅记录)。
    pub trusted: bool,
}

impl Device {
    /// 构造新设备,初始状态为离线。
    pub fn new(id: impl Into<String>, name: impl Into<String>, kind: DeviceKind, address: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            kind,
            address: address.into(),
            state: DeviceState::Offline,
            last_seen: 0,
            attrs: std::collections::BTreeMap::new(),
            trusted: false,
        }
    }

    /// 心跳续期:更新最后活跃时间并置为在线。
    pub fn heartbeat(&mut self, now: u64) {
        self.last_seen = now;
        self.state = DeviceState::Online;
    }

    /// 按离线阈值判定当前在线状态(不修改内部状态)。
    pub fn effective_state(&self, now: u64, offline_after_secs: u64) -> DeviceState {
        if self.state == DeviceState::Offline {
            return DeviceState::Offline;
        }
        if now.saturating_sub(self.last_seen) > offline_after_secs {
            DeviceState::Offline
        } else {
            self.state
        }
    }
}