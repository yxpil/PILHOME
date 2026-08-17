# Overall Architecture

## 1. Layers

```
┌──────────────────────────────────────────────────────────┐
│ WebUI — TypeScript SPA (rounded, black-white minimal)     │
│ dashboard / devices / events / AI lab / settings          │
├──────────────────────────────────────────────────────────┤
│ Server Gateway (Rust)                                     │
│ REST API · WebSocket event stream · MQTT bridge           │
│ HA bidirectional bridge · automation engine               │
│ SQLite/MySQL · Excel export · API Token auth · Webhook    │
├──────────────────────────────────────────────────────────┤
│ Core — Coordinator (module bus, registry, lifecycle)      │
│ PlugsCores × 9                                           │
│ NetLinker · NetScanear · SideAgent · HandModel · VEScaner │
│ YololookUP · SelflookUP · ATOGrowUP · AUTOTIME            │
├──────────────────────────────────────────────────────────┤
│ Devices: MQTT / TCP / ARP / mDNS / vendor cloud / HA      │
│ sensors · locks · cameras · lights · HVAC · PCs (WOL)     │
└──────────────────────────────────────────────────────────┘
```

## 2. Workspace Layout

```
PILHOME/
├── Cargo.toml            # workspace manifest (12 crates)
├── config.toml           # gateway config
├── README.md / README.en.md
├── Core/src/             # coordinator, message bus, module registry
├── PlugsCores/
│   ├── NetLinker/src/    # device model/registry/connector/WOL
│   ├── NetScanear/src/   # scanner/arp/mdns/fingerprint/discovery
│   ├── SideAgent/src/    # models/registry/runtime(rule engine)
│   ├── HandModel/src/    # control/params
│   ├── VEScaner/src/     # probe/frame(motion)
│   ├── YololookUP/src/   # detect/audit
│   ├── SelflookUP/src/   # health/audit_log
│   ├── ATOGrowUP/src/    # profile/evolve
│   └── AUTOTIME/src/     # schedule/rhythm
├── Server/src/           # gateway (main, api, mqtt, ha_bridge, ws,
│                         #  automation, discover, vendor, storage, excel, auth)
├── WebUI/                # TS front-end (Vite, strict)
└── docs/  +  docs/en/    # 12 chapters, Chinese & English
```

## 3. Data Flow

- **Inbound**: device events (MQTT `pilhome/+/event`) → EventLog (ring buffer + broadcast) → WebSocket push / automation engine / SQLite write / MySQL sync.
- **Outbound**: REST/WS/MQTT commands → HA services / device topics / webhooks.
- **Discovery**: ARP + TCP scan + mDNS → fingerprint (type/vendor) → device registry.
- **Learning**: events → ATOGrowUP profile (24h rhythm) → AUTOTIME arm suggestions; evolution mutates rules by fitness.

## 4. Key Design Choices

- **Rust workspace**: static-first, each module = crate (modular `src/`).
- **Event broadcast**: tokio broadcast — one write, many subscribers (WS stream, automation, storage).
- **Everything degrade-gracefully**: DB/MySQL/HA offline never blocks the gateway.
- **Zero-dependency where possible**: mDNS, WOL, Excel xlsx, webhook HTTP are all hand-written on std.
- **Auth optional**: `auth_enabled` off by default; token issued locally, verified by axum middleware.

## 5. Module Counts

| Item | Count |
| --- | --- |
| Rust source files | 77 |
| TS source files | 8+ |
| Crates | 12 |
| Unit tests (green) | 27+ |
| REST endpoints | 40+ |
| Docs (CN/EN) | 12 + 12 |