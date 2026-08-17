# Home Assistant 集成 —— 全屋设备接管

PILHOME 与 Home Assistant(HA)双向集成,实现**接管全部家居设备**:

- **下行同步**:通过 HA WebSocket API 拉取全屋实体状态,映射为 PILHOME 统一设备台账;
- **上行控制**:在 PILHOME 中直接调用 HA 服务(开关灯、锁门、拉窗帘…),统一接管;
- **自动发现**:MQTT Discovery 保留消息,反向把 PILHOME 设备注册进 HA。

## 一、集成架构

```
                    ┌─────────────── 双向接管 ───────────────┐
                    ▼                                        │
PILHOME 网关 ◀──WebSocket(get_states / call_service)──▶ Home Assistant
   │   ▲                                                    │
   │   └── MQTT Discovery 实体注册 ◀──Mosquitto──▶ 全屋设备 │
   └── 统一台账 + 场景控制(布防/撤防/离家/回家)              │
```

## 二、配置(`config.toml`)

```toml
[ha]
discovery = true
discovery_prefix = "homeassistant"
# HA WebSocket 地址(HA 设置 → 系统 → 网络 可查;默认 http 端口 8123)
ws_url = "ws://127.0.0.1:8123/api/websocket"
# HA 长效访问令牌(个人资料 → 安全 → 长效访问令牌)
token = "eyJhbGciOiJIUzI1NiIs..."
sync_interval_secs = 60
```

## 三、下行同步:实体 → 设备台账

HA `get_states` 全量拉取后,按领域与 device_class 映射:

| HA 领域 | PILHOME 种类 | 说明 |
| --- | --- | --- |
| `light` | light | 灯(含亮度属性) |
| `switch` | switch | 开关 / 插座 |
| `climate` | climate | 空调 / 地暖(含温度属性) |
| `cover` | cover | 窗帘 / 车库门 |
| `fan` | fan | 风扇 / 新风 |
| `siren` | siren | 警报器 |
| `lock` | lock | 门锁 |
| `camera` | camera | 摄像头 |
| `binary_sensor`(motion/presence) | presence | 人体存在 |
| `binary_sensor`(door/window) | contact | 门磁窗磁 |
| `binary_sensor`(smoke/gas/water) | sensor | 安防传感 |
| `sensor` | sensor | 温湿度等 |

同步后设备 `address = ha://<entity_id>`,关键属性(brightness/temperature/device_class)存入 attrs,全部纳入 NetLinker 台账与行为画像。

## 四、上行控制:全屋命令

| API | 说明 |
| --- | --- |
| `GET /api/ha/entities` | 实体缓存 + 连接状态 |
| `POST /api/ha/control` | 单实体控制 `{entity_id, action}` |
| `POST /api/ha/sync` | 触发全量同步 |
| `POST /api/ha/scenes` | 全屋场景一键执行 |

动作映射(按实体领域自动选服务):

| 动作 | 服务 |
| --- | --- |
| `on` / `off` / `toggle` | `turn_on` / `turn_off` / `toggle` |
| `lock` / `unlock`(lock 领域) | `lock` / `unlock` |
| `open` / `close`(cover 领域) | `open_cover` / `close_cover` |

示例:
```bash
# 关掉客厅灯
curl -X POST localhost:8080/api/ha/control -H "Content-Type: application/json" \
  -d '{"entity_id":"light.living_room","action":"off"}'

# 一键布防(全部门锁 + 关窗帘 + 关灯)
curl -X POST localhost:8080/api/ha/scenes -H "Content-Type: application/json" \
  -d '{"scene":"arm"}'
```

## 五、全屋场景

| 场景 | 动作 |
| --- | --- |
| `arm` 布防 | 全部 lock→lock、cover→close、light/switch/fan→off |
| `disarm` 撤防 | 全部 lock→unlock、cover→open |
| `away` 离家 | arm + 全屋灯/开关关闭 |
| `home` 回家 | disarm + 开窗帘 |

## 六、反向:MQTT Discovery 实体注册

PILHOME 设备(MQTT 直连 / 手动注册)自动发布到
`homeassistant/<platform>/pilhome_<id>/config` 保留消息,HA 免配置发现:

| 设备种类 | HA 平台 | payload |
| --- | --- | --- |
| contact | binary_sensor | open / closed |
| presence | binary_sensor | motion / clear |
| lock | lock | LOCK / UNLOCK |

## 七、联动自动化(HA 侧示例)

```yaml
# PILHOME 场景与 HA 联动:人离开家自动布防
alias: "离家自动布防"
triggers:
  - trigger: state
    entity_id: person.owner
    to: "not_home"
actions:
  - action: mqtt.publish
    data:
      topic: "pilhome/gateway/command"
      payload: "arm"
```

## 八、优势

- **全屋接管**:HA 上千种设备(Zigbee/Z-Wave/品牌云/MQTT)全部纳入统一台账;
- **双向闭环**:状态进来、命令出去,WebUI 一个界面控制全屋;
- **本地安全**:WebSocket 走内网,令牌本地配置;
- **场景一键化**:布防/离家等场景批量下发,秒级生效。