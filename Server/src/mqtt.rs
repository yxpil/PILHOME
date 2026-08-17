//! MQTT 接入:订阅设备事件、转发命令、自动重连。
//!
//! 订阅主题:`{prefix}/{device_id}/event`
//! 设备上报格式(JSON):`{"key":"contact","value":"open"}`

use crate::state::{now_secs, AppState};
use crate::Config;
use pilhome_atogrowup::EventRecord;
use pilhome_netlinker::DeviceKind;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// 命令通道消息。
#[derive(Debug, Clone)]
pub struct MqttCommand {
    pub topic: String,
    pub payload: String,
}

/// 启动 MQTT 后台任务(自动重连,永不退出)。
pub fn spawn(state: Arc<AppState>, cfg: Config) -> mpsc::Sender<MqttCommand> {
    let (tx, mut rx) = mpsc::channel::<MqttCommand>(64);
    tokio::spawn(async move {
        loop {
            tracing::info!("MQTT 连接 {} ...", cfg.mqtt.host);
            let mut options = MqttOptions::new(&cfg.mqtt.client_id, &cfg.mqtt.host, 1883);
            options.set_keep_alive(Duration::from_secs(30));
            let (client, mut eventloop) = AsyncClient::new(options, 16);

            let sub_topic = format!("{}/+/event", cfg.mqtt.topic_prefix);
            if let Err(err) = client.subscribe(sub_topic.clone(), QoS::AtMostOnce).await {
                tracing::warn!("订阅失败:{err}");
            } else {
                tracing::info!("已订阅 {sub_topic}");
            }

            // 连接就绪后发布一次 HA Discovery。
            let devices = state.devices.all();
            crate::ha::discovery::publish_devices(&client, &cfg, &devices).await;

            loop {
                tokio::select! {
                    command = rx.recv() => {
                        let Some(cmd) = command else { return };
                        if let Err(err) = client.publish(cmd.topic, QoS::AtMostOnce, false, cmd.payload.as_bytes()).await {
                            tracing::warn!("命令发布失败:{err}");
                        }
                    }
                    event = eventloop.poll() => {
                        match event {
                            Ok(Event::Incoming(Packet::Publish(publish))) => {
                                handle_publish(&state, &cfg, &publish.topic, &publish.payload);
                            }
                            Ok(_) => {}
                            Err(err) => {
                                tracing::warn!("MQTT 连接断开:{err};5 秒后重连");
                                tokio::time::sleep(Duration::from_secs(5)).await;
                                break;
                            }
                        }
                    }
                }
            }
        }
    });
    tx
}

/// 处理一条设备上报。
fn handle_publish(state: &AppState, cfg: &Config, topic: &str, payload: &[u8]) {
    let Some(rest) = topic.strip_prefix(&format!("{}/", cfg.mqtt.topic_prefix)) else { return };
    let Some((device_id, _)) = rest.split_once('/') else { return };

    let Ok(body) = serde_json::from_slice::<serde_json::Value>(payload) else {
        tracing::warn!("设备 {device_id} 上报非法 JSON");
        return;
    };
    let key = body.get("key").and_then(|v| v.as_str()).unwrap_or("event").to_string();
    let value = body.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let now = now_secs();

    // 1) 设备心跳;未注册设备自动建档。
    if !state.devices.heartbeat(&device_id, now) {
        let kind = if key.contains("contact") { DeviceKind::Contact }
            else if key.contains("motion") { DeviceKind::Presence }
            else if key.contains("lock") { DeviceKind::Lock }
            else { DeviceKind::Unknown };
        let mut device = pilhome_netlinker::Device::new(device_id.to_string(), device_id.to_string(), kind, topic.to_string());
        device.heartbeat(now);
        state.devices.upsert(device);
        state.events.record(crate::events::Level::Info, "netlinker", format!("新设备自动建档:{device_id}"), Some(&device_id), None);
    }

    // 2) 事件日志。
    state.events.record(
        crate::events::Level::Info,
        "mqtt",
        format!("{device_id}:{key}={value}"),
        Some(&device_id),
        Some(serde_json::json!({ "key": key, "value": value })),
    );

    // 3) 行为画像。
    if let Ok(mut profile) = state.profile.lock() {
        profile.record(EventRecord { device_id: device_id.to_string(), key, value, ts: now });
    }
}