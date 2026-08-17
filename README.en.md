# PILHOME — IoT + AI Edge Computing Smart Home Security Monitoring System

A graduation design project: an intelligent home security monitoring system based on
**IoT + AI edge computing** (supervisor: Prof. Wang Haitao, School of Artificial Intelligence).

## System Overview

```
Smart Home Devices ──► Core (Coordinator) ──► Server Gateway ──► WebUI (Control Panel)
        │                   │                      │
        │            PlugsCores (9 modules)   MQTT / WebSocket / REST
        │                   │                      │
        └── MQTT / TCP / mDNS / ARP ◄──── HA Bridge / Vendor SDK / Automation
```

- **Backend**: Rust (workspace, 12 crates, 77 source files) — static-first, modular `src/`
- **Frontend**: TypeScript (strict mode, zero framework) — black-white minimalist rounded UI
- **Standalone & open**: built-in automation engine, WebSocket event stream, MQTT publish,
  HA bidirectional bridge (full-house takeover), SQLite/MySQL storage, Excel export

## Modules (defined by the user via txt filenames)

| Core | Coordinator of the whole system |
| --- | --- |
| **PlugsCores** | 9 pluggable modules |
| NetLinker | Network device connection program (device registry, connector, WOL wake) |
| NetScanear | Network device discovery (port scan, ARP table, mDNS, fingerprinting) |
| SideAgent | Edge AI model (model registry, inference trait, rule engine) |
| HandModel | Module control program (start/stop, params, state collection) |
| VEScaner | Video device discovery (RTSP/ONVIF probe, motion detection) |
| YololookUP | Video rough-audit module (YOLO detector interface + audit engine) |
| SelflookUP | Self-audit program (health checks, append-only audit log) |
| ATOGrowUP | Self-growth & mutation program (behavior profile, genetic rule evolution) |
| AUTOTIME | Timing & rhythm control (cron-like schedules, circadian rhythm curves) |
| **Server** | Web page + kernel bridging service (REST, MQTT, HA, automation, storage) |
| **WebUI** | Web console & control (black-white rounded, auto refresh) |

## Feature Highlights

- **Full-house takeover**: HA bidirectional bridge (WebSocket) syncs thousands of device
  entities into one registry; control directly (lights/locks/covers/scenes one-click arm)
- **Independent closed loop**: built-in automation engine (event trigger → action),
  scenes (arm/disarm/away/home), Webhook alerts, persistence
- **Open access**: `ws://host:8080/api/ws` real-time event stream, `POST /api/mqtt/publish`,
  40+ REST endpoints, API Token auth (`Authorization: Bearer <token>`)
- **Network discovery**: TCP scan + ARP table + mDNS zero-config discovery + port
  fingerprinting + MAC OUI vendor identification (Xiaomi/Huawei/Midea/Haier/Apple)
- **Vendor ecosystem**: mDNS auto-discovery (zero key); `VendorAdapter` trait reserved
  for cloud SDKs (Mi Home / Hilink / Midea / Haier / HomeKit)
- **WOL wake**: standard magic packet, one-click from WebUI
- **Edge AI**: gesture inference (SideAgent rule engine), frame-difference motion,
  YOLO video audit interface, circadian-rhythm-based behavior learning
- **Storage**: SQLite local event DB + optional MySQL sync + Excel(.xlsx)/CSV export
- **Self-test**: 27+ unit tests green, release build 0 warnings, tsc strict passed

## Quick Start

```powershell
cd C:\Users\Admin\OneDrive\Desktop\PILHOME
.\target\release\pilhome-server.exe config.toml
```

Open http://127.0.0.1:8080 (WebUI pre-built in `WebUI/dist`).

## Documentation

Chinese docs: `docs/` (12 chapters). English docs: `docs/en/` (same set).

| Chinese | English | Content |
| --- | --- | --- |
| docs/01-选题说明.md | docs/en/01-project-overview.md | Project overview & thesis outline |
| docs/02-总体架构.md | docs/en/02-architecture.md | Architecture & data flow |
| docs/03-模块设计.md | docs/en/03-module-design.md | Module design |
| docs/04-HomeAssistant集成.md | docs/en/04-home-assistant.md | HA integration / full-house takeover |
| docs/05-边缘AI推理.md | docs/en/05-edge-ai.md | Edge AI inference |
| docs/06-安全设计.md | docs/en/06-security.md | Security design |
| docs/07-部署指南.md | docs/en/07-deployment.md | Deployment guide |
| docs/08-API参考.md | docs/en/08-api-reference.md | API reference (40+ endpoints) |
| docs/09-测试与验证.md | docs/en/09-testing.md | Testing & verification |
| docs/10-扩展指南.md | docs/en/10-extension-guide.md | Extension guide |
| docs/11-自动化与开放接入.md | docs/en/11-automation.md | Automation & open access |
| docs/12-网络发现与厂商生态.md | docs/en/12-network-vendors.md | Network discovery & vendors |

## Tech Stack

Rust 1.97 (axum / tokio / rumqttc / tokio-tungstenite / rusqlite / zip) ·
TypeScript (Vite, strict) · Home Assistant MQTT Discovery · SQLite / MySQL