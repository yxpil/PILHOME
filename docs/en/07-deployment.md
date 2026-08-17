# Deployment Guide

## 1. Build

```powershell
cd C:\Users\Admin\OneDrive\Desktop\PILHOME
cargo build --release            # Rust workspace (12 crates)
cd WebUI && npm install && npm run build   # tsc strict + Vite → dist/
```

## 2. Run

```powershell
.\target\release\pilhome-server.exe config.toml
```

- WebUI: http://127.0.0.1:8080
- API: `http://127.0.0.1:8080/api/*`
- WebSocket event stream: `ws://127.0.0.1:8080/api/ws`

## 3. Config (`config.toml`)

```toml
[server]
listen = "127.0.0.1:8080"
data_dir = "data"

[mqtt]
broker = "127.0.0.1:1883"
client_id = "pilhome-gateway"

[ha]
discovery = true
discovery_prefix = "homeassistant"
ws_url = "ws://127.0.0.1:8123/api/websocket"   # empty = disabled
token = ""
sync_interval_secs = 60

[tasks]
scan_interval_secs = 600
cron_tick_secs = 1

webhook_url = ""                  # alert push, empty = disabled
auth_enabled = false              # API Token auth
api_tokens = []                   # preconfigured tokens
mysql_url = ""                    # MySQL sync, empty = disabled
```

## 4. First Boot Behavior

1. Opens SQLite `data/pilhome.db` (creates `events` table)
2. Runs one full auto-discovery round (TCP scan + ARP + mDNS + fingerprint)
3. Connects MQTT (subscribe `pilhome/+/event`), HA bridge (if configured)
4. Restores JSON snapshots (events/audit/profile)
5. Starts periodic tasks (cron check, scan, snapshot, storage sync, webhook)

## 5. Verification Checklist

| Item | How |
| --- | --- |
| WebUI loads | open http://127.0.0.1:8080 |
| Devices appear | after auto-discovery, or `POST /api/discover` |
| Events flow | any MQTT event → dashboard timeline / `GET /api/events` |
| Persistence | restart → `data/events.json` restored; `data/pilhome.db` grows |
| Full-house takeover | set `[ha] ws_url+token`, restart, `GET /api/ha/entities` |
| One-click arm | `POST /api/ha/scenes` body `{"scene":"arm"}` |
| WOL wake | `POST /api/wol` with a PC MAC |
| Export | WebUI settings → Export Excel/CSV |
| Real-time stream | connect `ws://127.0.0.1:8080/api/ws`, see events push |

## 6. Troubleshooting

| Symptom | Fix |
| --- | --- |
| MQTT no events | broker address wrong? check broker running; gateway logs |
| HA bridge not connected | ws_url/token wrong; HA WebSocket enabled? |
| Scan finds nothing | subnet prefix wrong in `config.toml` [tasks] scan config; firewall |
| SQLite unavailable | check `data/` writable; bundled sqlite needs no install |
| MySQL sync failed | check `mysql_url`, user grants, network |
| Auth 401 | token expired/revoked → issue new via `POST /api/token` |