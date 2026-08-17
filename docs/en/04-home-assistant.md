# Home Assistant Integration — Full-House Takeover

PILHOME integrates with Home Assistant (HA) bidirectionally:

- **Downlink sync**: pull all HA entities via WebSocket API → unified PILHOME device registry
- **Uplink control**: call HA services directly from PILHOME (lights/locks/covers…)
- **Auto discovery**: MQTT Discovery retained messages register PILHOME devices into HA

## Architecture

```
                    ┌──────────── bidirectional takeover ────────────┐
                    ▼                                                 │
PILHOME Gateway ◀──WebSocket(get_states / call_service)──▶ Home Assistant
   │   ▲                                                             │
   │   └── MQTT Discovery entities ◀──Mosquitto──▶ household devices │
   └── unified registry + scenes (arm/disarm/away/home)              │
```

## Config (`config.toml`)

```toml
[ha]
discovery = true
discovery_prefix = "homeassistant"
ws_url = "ws://127.0.0.1:8123/api/websocket"   # HA WebSocket address
token = "eyJhbGciOiJIUzI1NiIs..."              # HA long-lived access token
sync_interval_secs = 60
```

## Entity → Device Mapping

| HA domain | PILHOME kind | Note |
| --- | --- | --- |
| `light` | light | brightness attr |
| `switch` | switch | outlets |
| `climate` | climate | temperature attr |
| `cover` | cover | curtains/garage |
| `fan` | fan | fans/HVAC |
| `siren` | siren | alarm |
| `lock` | lock | door lock |
| `camera` | camera | camera |
| `binary_sensor` (motion/presence) | presence | human presence |
| `binary_sensor` (door/window) | contact | door/window sensors |
| `binary_sensor` (smoke/gas/water) | sensor | safety sensors |
| `sensor` | sensor | temp/humidity… |

## Control API

| API | Description |
| --- | --- |
| `GET /api/ha/entities` | entity cache + connection status |
| `POST /api/ha/control` | `{entity_id, action}` |
| `POST /api/ha/sync` | force full sync |
| `POST /api/ha/scenes` | `arm/disarm/away/home` batch execution |

Action mapping: `on/off/toggle` → `turn_on/turn_off/toggle`; lock domain → `lock/unlock`;
cover domain → `open_cover/close_cover`.

## Scenes

| Scene | Actions |
| --- | --- |
| `arm` | all locks→lock, covers→close, lights/switches/fans→off |
| `disarm` | all locks→unlock, covers→open |
| `away` | arm + all lights/switches off |
| `home` | disarm + open covers |

## Reverse: MQTT Discovery Registration

PILHOME devices publish retained config to `homeassistant/<platform>/pilhome_<id>/config`;
HA discovers them with zero YAML (contact→binary_sensor, presence→binary_sensor, lock→lock).