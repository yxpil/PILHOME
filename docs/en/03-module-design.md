# Module Design

## 1. Core — Coordinator (负责协调总体)

- `message.rs` — typed module message bus (`Message`, `MessageBus` over tokio broadcast)
- `registry.rs` — `ModuleRegistry` (id/label/version/status, thread-safe)
- `coordinator.rs` — module registration, lifecycle, publish/subscribe

## 2. NetLinker — Network Device Connection Program

| File | Responsibility |
| --- | --- |
| `device.rs` | `Device` model (id/name/kind/state/address/attrs), `DeviceKind` (13 types), `DeviceState` |
| `registry.rs` | `DeviceRegistry` — upsert/get/all/by_kind/heartbeat/online_count |
| `connector.rs` | `Connector` — per-device connection state machine with exponential backoff |
| `wol.rs` | WOL magic packet (6×FF + MAC×16), MAC parsing, UDP broadcast send |

## 3. NetScanear — Network Device Discovery Program

| File | Responsibility |
| --- | --- |
| `scanner.rs` | Concurrent TCP port scan (prefix/ports/timeout/concurrency) |
| `arp.rs` | `arp -a` parsing → IP↔MAC table |
| `mdns.rs` | mDNS zero-config discovery (hand-written DNS parser incl. compression pointers) |
| `fingerprint.rs` | Port-based type guess + MAC OUI vendor identification |
| `discovery.rs` | Scan result vs whitelist → unknown-host alerts |

## 4. SideAgent — Edge AI Model

| File | Responsibility |
| --- | --- |
| `model.rs` | `ModelInfo` (kind/format/status/latency), `Prediction` |
| `registry.rs` | `ModelRegistry` — register/load/unload/ready_count |
| `runtime.rs` | `ModelRuntime` trait + rule-based inference engine (8-dim features → gesture class) |

## 5. HandModel — Module Control Program

| File | Responsibility |
| --- | --- |
| `params.rs` | `ParamValue` enum, parameter definitions |
| `control.rs` | `Controller` — module start/stop, param push, state collection |

## 6. VEScaner — Video Device Discovery Program

| File | Responsibility |
| --- | --- |
| `probe.rs` | RTSP/ONVIF probe across subnet → `VideoDevice` candidates |
| `frame.rs` | Frame-difference motion detection (grayscale diff, threshold) |

## 7. YololookUP — Video Rough-Audit Module

| File | Responsibility |
| --- | --- |
| `detect.rs` | `Detection`/`DetectResult`; `Detector` trait (ONNX YOLO plug-in point) |
| `audit.rs` | `AuditEngine` — motion + detections → audit events; periodic rough audit |

## 8. SelflookUP — Self-Audit Program

| File | Responsibility |
| --- | --- |
| `health.rs` | `CheckItem`/`HealthStatus`, module health checks |
| `audit_log.rs` | Append-only `AuditLog` (actor/action/target/detail/ts) |

## 9. ATOGrowUP — Self-Growth & Mutation Program

| File | Responsibility |
| --- | --- |
| `profile.rs` | Per-device 24h behavior profile, anomaly detection |
| `evolve.rs` | Rule population with fitness; genetic mutation (param drift, elite retention) |

## 10. AUTOTIME — Timing & Rhythm Control

| File | Responsibility |
| --- | --- |
| `schedule.rs` | Cron-like rules (`min hour dom month dow`), UTC-robust matching |
| `rhythm.rs` | 24h circadian curve, default template, `suggest_arm_windows` |

## 11. Server — Web Page + Kernel Bridging Service

| File | Responsibility |
| --- | --- |
| `main.rs` | Assembly: config → state → MQTT/HA/automation/storage tasks → axum server |
| `config.rs` | TOML config (server/mqtt/ha/tasks/storage/auth…) |
| `state.rs` | `AppState` — all modules + SQLite/MySQL/tokens |
| `events.rs` | `EventLog` ring buffer + broadcast subscribe |
| `mqtt.rs` | MQTT client bridge (subscribe device events, publish commands) |
| `ha_bridge.rs` | HA WebSocket bidirectional bridge (get_states / call_service, auto-reconnect) |
| `ha/` | MQTT Discovery entity registration |
| `ws.rs` | WebSocket real-time event stream (open access) |
| `automation.rs` | Automation engine (trigger + window + actions) |
| `discover.rs` | Full auto-discovery orchestration (scan+ARP+mDNS+fingerprint) |
| `vendor.rs` | Vendor ecosystem (mDNS classification + `VendorAdapter` SDK trait) |
| `storage.rs` | SQLite store + MySQL sync |
| `excel.rs` | .xlsx (hand-written OOXML) + CSV export |
| `auth.rs` | API Token issue/verify middleware |
| `webhook.rs` | Zero-dependency HTTP alert push |
| `persistence.rs` | JSON snapshots (events/audit/profile) restore on boot |
| `tasks.rs` | Periodic: cron checks, scans, snapshots, storage sync, webhook push |
| `api/` | 40+ REST endpoints (devices/events/scan/models/gesture/profile/evolve/video/ha/automations/export…) |

## 12. WebUI — Web Console & Control

- `main.ts` — hash router + auto-refresh dashboard
- `pages/` — dashboard (stats/rhythm/modules), devices (control/scenes/WOL/discovery),
  events (timeline), ai (gesture lab/behavior curve/evolution), settings
  (schedules/automations/scan/models/token/export/storage)
- `components/ui.ts` — `el()` DOM helper, card/dot/tag/empty primitives
- Black-white minimalist, rounded (capsule) design, no emoji