# REST API 参考

基础地址:`http://127.0.0.1:8080`。全部请求/响应为 JSON。

## 端点总览

| 方法 | 路径 | 模块 | 说明 |
| --- | --- | --- | --- |
| GET | `/api/status` | Server | 系统信息、概览指标、模块状态 |
| GET | `/api/health` | SelflookUP | 健康自检报告(设备在线率/缓冲水位/模型/审计/模块) |
| GET | `/api/devices` | NetLinker | 设备台账 |
| POST | `/api/devices` | NetLinker | 注册/更新设备 |
| POST | `/api/devices/{id}/trust` | NetLinker | 设置白名单 |
| GET | `/api/events?limit=100` | Server | 事件流 |
| GET | `/api/audit?limit=100` | SelflookUP | 审计日志 |
| POST | `/api/scan/run` | NetScanear | 立即执行网络扫描 |
| GET | `/api/scan/last` | NetScanear | 最近一次扫描报告 |
| GET | `/api/models` | SideAgent | 边缘模型列表 |
| POST | `/api/models` | SideAgent | 注册模型 |
| POST | `/api/gesture` | SideAgent | 手势推理(8 维特征 → 分类) |
| GET | `/api/profile` | ATOGrowUP | 行为画像曲线与异常建议 |
| GET | `/api/evolve` | ATOGrowUP | 规则种群与代数 |
| POST | `/api/evolve/seed` | ATOGrowUP | 注入初始规则种群 |
| POST | `/api/evolve/tick` | ATOGrowUP | 执行一代变异 |
| POST | `/api/video/probe` | VEScaner | 探测网段视频设备(RTSP/ONVIF) |
| POST | `/api/video/audit` | YololookUP | 视频粗略代审计(可注入检测结果) |
| GET | `/api/control` | HandModel | 受控模块列表 |
| POST | `/api/control/{id}/start` | HandModel | 启动模块 |
| POST | `/api/control/{id}/stop` | HandModel | 停止模块 |
| GET | `/api/schedules` | AUTOTIME | 定时规则 |
| POST | `/api/schedules` | AUTOTIME | 新增定时规则 |
| GET | `/api/rhythm` | AUTOTIME | 节律曲线与布防建议 |
| GET | `/api/ha/entities` | HA 桥 | HA 实体缓存与连接状态(全屋接管) |
| POST | `/api/ha/control` | HA 桥 | 单实体控制 `{entity_id, action}` |
| POST | `/api/ha/sync` | HA 桥 | 触发全量状态同步 |
| POST | `/api/ha/scenes` | HA 桥 | 全屋场景:`arm/disarm/away/home` |
| GET | `/api/automations` | 自动化 | 自动化规则列表 |
| POST | `/api/automations` | 自动化 | 新增自动化规则 |
| POST | `/api/automations/{id}/toggle` | 自动化 | 启用/停用规则 |
| DELETE | `/api/automations/{id}` | 自动化 | 删除规则 |
| POST | `/api/mqtt/publish` | Server | 经网关发布 MQTT 消息 |
| WS | `/api/ws` | Server | 实时事件流(对外开放接入) |
| POST | `/api/discover` | NetScanear | 全自动发现(扫描+ARP+mDNS+指纹入库) |
| GET | `/api/arp` | NetScanear | 系统 ARP 表 |
| GET | `/api/vendors` | 厂商 | 厂商生态设备即时发现 |
| POST | `/api/wol` | NetLinker | WOL 网络唤醒 `{mac\|device_id}` |
| POST | `/api/token` | Server | 签发 API 令牌 |
| GET | `/api/token/status` | Server | 令牌库状态 |
| POST | `/api/token/revoke` | Server | 吊销令牌 |
| GET | `/api/export/events` | Server | 导出 Excel(xlsx)/ CSV 事件数据 |

## 示例

### GET /api/status
```json
{
  "system": { "name": "PILHOME 边缘网关", "version": "0.1.0", "uptime_secs": 3600, "now": 1785900000 },
  "overview": {
    "devices_total": 4, "devices_online": 3, "events": 128, "audit": 42,
    "models_ready": 2, "connections": 3
  },
  "modules": [
    { "id": "netlinker", "label": "NetLinker 网络设备连接程序", "status": "运行中", "metric": "3/4 设备在线" },
    { "id": "autotime", "label": "AUTOTIME 定时和节律控制", "status": "运行中", "metric": "2 条定时规则" }
  ]
}
```

### POST /api/gesture
```json
{ "features": [0.05, 0.06, 0.04, 0.05, 0.05, 0.06, 0.05, 0.04] }
```
```json
{
  "ok": true, "latency_ms": 0.012,
  "top": { "label": "fist", "confidence": 0.999 },
  "results": [
    { "label": "fist", "confidence": 0.999, "bbox": null },
    { "label": "ok", "confidence": 0.68, "bbox": null }
  ]
}
```

### GET /api/profile
```json
{
  "total_events": 512,
  "curves": { "contact_front_door": [0.0, 0.0, 0.0, 0.0, 0.0, 0.1, 0.4, 0.9, 0.5, 0.2, 0.3, 0.4, 0.3, 0.2, 0.2, 0.3, 0.5, 0.7, 0.8, 0.6, 0.3, 0.1, 0.0, 0.0] },
  "suggestions": []
}
```

### POST /api/video/audit
```json
{ "camera_id": "cam_living", "motion_ratio": 0.05, "detections": [{ "label": "person", "confidence": 0.82, "bbox": [0.1, 0.2, 0.3, 0.4] }] }
```
```json
{ "ok": true, "event": { "ts": 1785900000, "camera_id": "cam_living", "level": "critical", "summary": "检出关键目标:person(共 1 个)", "detections": [...] } }
```

### GET /api/rhythm
```json
{
  "curve": [0.05, 0.03, 0.02, 0.02, 0.03, 0.06, 0.2, 0.45, 0.7, 0.55, 0.45, 0.5, 0.55, 0.5, 0.45, 0.4, 0.5, 0.65, 0.8, 0.75, 0.55, 0.35, 0.15, 0.08],
  "recommended_arm_windows": [[22, 6]]
}
```

### POST /api/ha/control
```json
{ "entity_id": "light.living_room", "action": "off" }
```
```json
{ "ok": true, "entity_id": "light.living_room", "action": "off" }
```

### POST /api/ha/scenes
```json
{ "scene": "arm" }
```
```json
{ "ok": true, "scene": "arm", "executed": 12 }
```

### GET /api/ha/entities
```json
{
  "enabled": true, "connected": true, "entity_count": 24, "last_sync": 1785900000,
  "entities": [
    { "entity_id": "light.living_room", "name": "客厅灯", "domain": "light", "state": "on",
      "attributes": { "brightness": 200, "device_class": null } }
  ]
}
```

### GET /api/health
```json
{
  "ok": true,
  "report": {
    "generated_at": 1785900000,
    "overall": "ok",
    "items": [
      { "name": "device.online_rate", "status": "ok", "detail": "4/4 台在线", "checked_at": 1785900000 },
      { "name": "models.ready", "status": "ok", "detail": "2 个模型就绪", "checked_at": 1785900000 }
    ]
  }
}
```

## 错误

- `400` 参数/规则非法(如 Cron 表达式错误);
- `404` 资源不存在(设备/模块未注册);
- `500` 内部错误。