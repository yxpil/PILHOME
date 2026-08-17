# API Reference (40+ endpoints)

Base: `http://127.0.0.1:8080` · Auth (optional): `Authorization: Bearer <token>`

## Devices & Network

| Method | Path | Description |
| --- | --- | --- |
| GET | `/api/devices` | device registry |
| POST | `/api/devices` | upsert device |
| POST | `/api/devices/{id}/trust` | whitelist toggle |
| POST | `/api/devices/{id}/command` | send MQTT command |
| POST | `/api/discover` | full auto-discovery (scan+ARP+mDNS+fingerprint) |
| GET | `/api/arp` | system ARP table |
| GET | `/api/vendors` | vendor ecosystem instant discovery |
| POST | `/api/wol` | WOL wake `{mac\|device_id}` |
| GET | `/api/scan/last` | last scan report |
| POST | `/api/scan/run` | run TCP scan |

## Events & Analytics

| Method | Path | Description |
| --- | --- | --- |
| GET | `/api/events` | event log (limit) |
| GET | `/api/status` | system overview + module states |
| GET | `/api/health` | self-audit report |
| GET | `/api/audit` | audit log |
| GET | `/api/profile` | behavior profile 24h curves |
| GET | `/api/rhythm` | circadian rhythm + arm suggestions |

## AI & Video

| Method | Path | Description |
| --- | --- | --- |
| GET | `/api/models` | model registry |
| POST | `/api/models/load` | load model |
| POST | `/api/gesture` | gesture inference (8-dim features) |
| POST | `/api/evolve/seed` | inject rule population |
| POST | `/api/evolve/tick` | evolve one generation |
| GET | `/api/evolve` | population view |
| POST | `/api/video/probe` | RTSP/ONVIF video device probe |
| POST | `/api/video/audit` | video rough audit |

## Control & Automation

| Method | Path | Description |
| --- | --- | --- |
| POST | `/api/control/{module}/start` | start module |
| POST | `/api/control/{module}/stop` | stop module |
| GET | `/api/control` | module states |
| GET | `/api/schedules` | timing rules |
| POST | `/api/schedules` | add timing rule |
| GET | `/api/automations` | automation rules |
| POST | `/api/automations` | add automation |
| POST | `/api/automations/{id}/toggle` | enable/disable |
| DELETE | `/api/automations/{id}` | delete rule |

## HA Takeover

| Method | Path | Description |
| --- | --- | --- |
| GET | `/api/ha/entities` | HA entity cache + connection |
| POST | `/api/ha/control` | single entity control |
| POST | `/api/ha/sync` | force sync |
| POST | `/api/ha/scenes` | `arm/disarm/away/home` |

## Open Access & Data

| Method | Path | Description |
| --- | --- | --- |
| POST | `/api/token` | issue API token |
| GET | `/api/token/status` | token store status |
| POST | `/api/token/revoke` | revoke token |
| POST | `/api/mqtt/publish` | publish MQTT via gateway |
| GET | `/api/export/events` | Excel(.xlsx)/CSV export |
| WS | `/api/ws` | real-time event stream |

## Examples

```bash
# Discover
curl -X POST localhost:8080/api/discover

# WOL
curl -X POST localhost:8080/api/wol -H "Content-Type: application/json" -d '{"mac":"aa:bb:cc:dd:ee:ff"}'

# HA control
curl -X POST localhost:8080/api/ha/control -H "Content-Type: application/json" -d '{"entity_id":"light.living_room","action":"off"}'

# Scene
curl -X POST localhost:8080/api/ha/scenes -H "Content-Type: application/json" -d '{"scene":"arm"}'

# Automation
curl -X POST localhost:8080/api/automations -H "Content-Type: application/json" -d '{"name":"night door alert","device_id":"contact_front_door","key":"contact","value":"open","action_type":"log","message":"door opened"}'

# Export
curl -OJ "localhost:8080/api/export/events?format=xlsx"

# Token auth
curl -H "Authorization: Bearer <token>" localhost:8080/api/status
```