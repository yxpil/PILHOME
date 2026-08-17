# Automation Engine & Open Access

PILHOME is a complete closed loop without any external platform: **built-in automation
engine** (equivalent to HA Automation) + **open interfaces** (WebSocket event stream /
MQTT publish / REST) — external systems (HA, custom scripts, mobile apps, other
gateways) can all connect in.

## 1. Automation Engine

### Rule Model

```
Rule = Trigger (event match) + Condition (time window) + Actions (list)
```

| Part | Field | Description |
| --- | --- | --- |
| Trigger | `device_id` | device id (empty = any) |
| | `key` | event key (`contact` / `motion`) |
| | `value` | event value (`open`) |
| | `min_level` | minimum level (info/warn/alert) |
| Condition | `window` | hour window `(start, end)` (empty = all day) |
| Actions | `log` | record a system event |
| | `mqtt` | publish MQTT (topic + payload) |
| | `ha` | call HA service (entity_id + action) |
| | `webhook` | send HTTP alert (url) |

### Examples

```bash
# Night door open → turn on living room light
curl -X POST localhost:8080/api/automations -H "Content-Type: application/json" -d '{
  "name": "night door alert", "device_id": "contact_front_door",
  "key": "contact", "value": "open",
  "action_type": "ha", "entity_id": "light.living_room", "action": "on"
}'

# Any alert event → MQTT siren
curl -X POST localhost:8080/api/automations -H "Content-Type: application/json" -d '{
  "name": "alert→siren", "min_level": "alert",
  "action_type": "mqtt", "topic": "pilhome/siren/command", "payload": "on"
}'
```

### Rules API

| Method | Path | Description |
| --- | --- | --- |
| GET | `/api/automations` | list |
| POST | `/api/automations` | add |
| POST | `/api/automations/{id}/toggle` | enable/disable |
| DELETE | `/api/automations/{id}` | delete |

Engine scans the event stream incrementally every second and executes actions in order.

## 2. WebSocket Real-Time Event Stream

**Any client can connect `ws://<host>:8080/api/ws` and receive all events in real time.**

```bash
node -e '
const ws = new WebSocket("ws://127.0.0.1:8080/api/ws");
ws.onmessage = (m) => console.log(m.data);
'
```

Handshake:
```json
{ "type": "auth_ok", "system": { "name": "PILHOME 边缘网关", "version": "0.1.0" } }
```
Events:
```json
{ "type": "event", "data": { "ts": 1785900000, "level": "alert", "source": "netscanear", "message": "陌生设备", "device_id": null, "data": null } }
```
Keep-alive: client sends `ping` → server replies `pong`.

Use cases: mobile app push, HA automations reading PILHOME events, downstream gateways.

## 3. MQTT Publish via Gateway

```bash
curl -X POST localhost:8080/api/mqtt/publish -H "Content-Type: application/json" \
  -d '{"topic":"pilhome/siren/command","payload":"on"}'
```

External systems avoid direct broker access; all publishes are audited.

## 4. Relationship with Home Assistant

| Capability | PILHOME built-in | HA side (optional) |
| --- | --- | --- |
| Device access | MQTT / discovery / video | bidirectional bridge (thousands) |
| Automation | ✅ built-in engine | reuse complex scenes |
| Scenes | ✅ arm/disarm/away/home | linkable |
| History | ✅ persistence | can read event stream |
| Real-time | ✅ WebSocket stream | subscribable |
| Control | ✅ REST / MQTT | callable |

**Two modes**:
1. **Standalone**: no HA — full closed loop;
2. **HA coexisting**: PILHOME takes over HA devices; HA is an optional device layer.

## 5. Security Notes

- Default bind 127.0.0.1; expose remotely only behind VPN/firewall
- Enable API Token auth for open networks