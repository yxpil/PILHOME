//! 设备注册中心:并发安全的设备台账。

use super::device::{Device, DeviceKind, DeviceState};
use std::collections::HashMap;
use std::sync::RwLock;

/// 设备注册中心。
///
/// 内部使用 `RwLock<HashMap>` 保证多线程(发现线程、连接线程、API 线程)
/// 并发读写安全。读多写少的场景下读写锁优于互斥锁。
#[derive(Debug, Default)]
pub struct DeviceRegistry {
    inner: RwLock<HashMap<String, Device>>,
}

impl DeviceRegistry {
    /// 创建空注册中心。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册或更新设备(存在则覆盖,不存在则插入)。
    pub fn upsert(&self, device: Device) {
        self.inner.write().expect("registry lock poisoned").insert(device.id.clone(), device);
    }

    /// 按 ID 移除设备。
    pub fn remove(&self, id: &str) -> Option<Device> {
        self.inner.write().expect("registry lock poisoned").remove(id)
    }

    /// 按 ID 查询设备。
    pub fn get(&self, id: &str) -> Option<Device> {
        self.inner.read().expect("registry lock poisoned").get(id).cloned()
    }

    /// 全部设备(按 ID 排序,输出稳定)。
    pub fn all(&self) -> Vec<Device> {
        let mut list: Vec<Device> = self.inner.read().expect("registry lock poisoned").values().cloned().collect();
        list.sort_by(|a, b| a.id.cmp(&b.id));
        list
    }

    /// 设备总数。
    pub fn len(&self) -> usize {
        self.inner.read().expect("registry lock poisoned").len()
    }

    /// 注册中心是否为空。
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 设备心跳:存在则续期,不存在则忽略。
    pub fn heartbeat(&self, id: &str, now: u64) -> bool {
        let mut guard = self.inner.write().expect("registry lock poisoned");
        match guard.get_mut(id) {
            Some(dev) => {
                dev.heartbeat(now);
                true
            }
            None => false,
        }
    }

    /// 按种类过滤设备。
    pub fn by_kind(&self, kind: DeviceKind) -> Vec<Device> {
        self.all().into_iter().filter(|d| d.kind == kind).collect()
    }

    /// 在线设备数量(按离线阈值判定)。
    pub fn online_count(&self, now: u64, offline_after_secs: u64) -> usize {
        self.all()
            .into_iter()
            .filter(|d| d.effective_state(now, offline_after_secs) != DeviceState::Offline)
            .count()
    }

    /// 标记设备状态(如审计模块发现异常时)。
    pub fn set_state(&self, id: &str, state: DeviceState) {
        if let Some(dev) = self.inner.write().expect("registry lock poisoned").get_mut(id) {
            dev.state = state;
        }
    }
}