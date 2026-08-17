# Project Overview & Thesis Outline

## 1. Topic

**Design and Implementation of an Intelligent Home Security Monitoring System Based on IoT and AI Edge Computing**

- Supervisor: Prof. Wang Haitao (School of Artificial Intelligence)
- Nature: Graduation design (thesis)

## 2. Background & Significance

- Home IoT devices grow rapidly (sensors, cameras, locks, lights, HVAC…); heterogeneous protocols and closed vendor ecosystems fragment control.
- Cloud-centric smart homes suffer latency, privacy leakage and offline failures.
- **Edge computing** keeps sensing, inference and control local: low latency, offline capability, privacy by design.
- This project builds a local gateway that unifies heterogeneous devices, runs edge AI (gesture/motion/YOLO audit), learns household behavior, automates security scenes and opens interfaces for external systems.

## 3. Goals

1. **Unified device takeover**: MQTT, LAN discovery (TCP/ARP/mDNS), HA bidirectional bridge, vendor ecosystem — all devices in one registry, controllable in one UI.
2. **Edge AI**: gesture inference, motion detection, YOLO-based video audit, all on-device.
3. **Self-learning**: behavior profiles (24h rhythm), genetic rule evolution (ATOGrowUP).
4. **Self-audit**: health checks, append-only audit log, alert webhooks.
5. **Automation & scenes**: built-in automation engine (event → action), arm/disarm scenes.
6. **Openness**: WebSocket event stream, MQTT publish, REST API, API Token auth.
7. **Data**: SQLite persistence, optional MySQL sync, Excel/CSV export.

## 4. Research Content

| # | Content | Module |
| --- | --- | --- |
| 1 | Device modeling & registry, connection state machine, WOL wake | NetLinker |
| 2 | Port scan, ARP table, mDNS discovery, fingerprint analysis | NetScanear |
| 3 | Edge model registry, inference trait, rule engine | SideAgent |
| 4 | Module control (start/stop/params) | HandModel |
| 5 | Video device discovery, frame-difference motion | VEScaner |
| 6 | YOLO video audit engine | YololookUP |
| 7 | Health checks & audit log | SelflookUP |
| 8 | Behavior profile & rule mutation (genetic) | ATOGrowUP |
| 9 | Cron schedules & circadian rhythm | AUTOTIME |
| 10 | Gateway: REST/MQTT/HA/automation/storage/export | Server |
| 11 | Web console (black-white rounded UI) | WebUI |

## 5. Innovations

- **Full-house takeover via HA bidirectional bridge** (WebSocket): thousands of devices, zero-config; scenes batch control.
- **Independent closed loop**: no external platform required; HA becomes an optional device source, not a dependency.
- **Self-growth rules**: genetic mutation of automation rules driven by event feedback.
- **Circadian rhythm security**: recommended arm windows derived from learned behavior.
- **Zero-dependency discovery**: hand-written mDNS/DNS compression-pointer parser in std Rust.

## 6. Technical Route

```
Layer 4 UI:      WebUI (TS, rounded black-white)
Layer 3 Service: Server gateway (REST/WS/MQTT/HA bridge/automation/storage/export)
Layer 2 Cores:   Core coordinator + 9 PlugsCores crates
Layer 1 Device:  MQTT / TCP / ARP / mDNS / vendor cloud SDK / HA entities
```

## 7. Schedule

| Phase | Content |
| --- | --- |
| 1 | Requirement analysis, architecture, device model |
| 2 | Cores modules (discovery/connection/AI/time) |
| 3 | Server gateway + HA bridge + automation |
| 4 | WebUI + storage (SQLite/MySQL/Excel) |
| 5 | Testing, docs, thesis writing |