# Security Design

## 1. Defense in Depth

```
Internet ── firewall ── gateway(127.0.0.1 by default) ── devices (LAN)
              │
         VPN / trusted intranet for remote access
```

## 2. Network Security

- Gateway binds **127.0.0.1** by default; remote exposure requires VPN/firewall
- Device commands via MQTT with per-topic semantics; HA bridge token stored locally
- WOL/ARP/mDNS discovery are read-only probes (no writes to third parties)

## 3. Access Control (API Token)

- `POST /api/token` issues cryptographically random 32-byte tokens (getrandom)
- Optional middleware auth: `auth_enabled = true` → all `/api/*` need
  `Authorization: Bearer <token>` (static assets, /api/token, /api/ws exempt)
- `POST /api/token/revoke` invalidates tokens; all issue/revoke audited

## 4. Data Security

- SQLite local file (`data/pilhome.db`); MySQL sync only when `mysql_url` configured
- Events include no raw credentials; passwords never logged
- Snapshots (events/audit/profile) JSON — readable, restorable, no binary bloat

## 5. Audit Trail (SelflookUP)

- Append-only `AuditLog`: actor/action/target/detail/ts — every API mutation recorded
  (device upsert/trust, scenes, automation add/toggle, token issue, WOL send…)
- Health checks: device online rate, event buffer level, storage availability

## 6. Device Security

- Unknown devices (not in whitelist) trigger alert events
- MAC OUI vendor identification helps spot rogue devices
- Heartbeat + offline detection (120s) flags dead/removed sensors

## 7. Threat Model Summary

| Threat | Mitigation |
| --- | --- |
| Unauthorized API access | Token auth (optional, audited) |
| Malicious MQTT payloads | Topic whitelist semantics, graceful parse failures |
| Rogue device on LAN | Discovery + fingerprint + alert |
| Data tampering | Append-only audit log, snapshots |
| Dependency risk | Minimal deps; mDNS/WOL/xlsx/webhook hand-written |