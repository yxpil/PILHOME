# Network Discovery, WOL, Vendors, Token Auth & Storage

## 1. LAN Device Discovery (auto-discover / auto-analyze)

One full round runs at startup; manual trigger anytime:

| Method | Description |
| --- | --- |
| TCP port scan | concurrent scan of 22/80/443/554/1883/5353/8123/9100 |
| ARP table query | `arp -a` parse → IP↔MAC |
| mDNS zero-config | multicast query `_hap/_miio/_easylink/_midea/...` |
| Port fingerprinting | 554→camera, 8123→HA, 9100→printer… |
| MAC OUI vendor ID | Xiaomi/Huawei/Midea/Haier/Apple/TP-Link prefix table |

All results auto-inserted into the registry (with `ports/fingerprint/confidence/mac/vendor`
attrs); unknown hosts raise alert events.

```bash
curl -X POST localhost:8080/api/discover   # manual round
curl localhost:8080/api/arp                # ARP table
curl localhost:8080/api/vendors            # vendor devices
```

## 2. WOL Wake

Standard magic packet (6×0xFF + MAC×16, UDP broadcast :9):

```bash
curl -X POST localhost:8080/api/wol -d '{"mac":"aa:bb:cc:dd:ee:ff"}'
curl -X POST localhost:8080/api/wol -d '{"device_id":"net_192.168.1.105"}'
```

WebUI: devices with a MAC attr get a "Wake" button; manual MAC input at the top bar.

## 3. Vendor Ecosystem (Xiaomi/Huawei/Midea/Haier/Apple)

**Auto-discovery (real, zero-key)**: mDNS service-type identification —
`_hap`→Apple, `_miio`→Xiaomi, `_easylink`→Huawei, `_midea`→Midea, `_hiaircon`→Haier.

**Cloud SDK adapters (interface reserved)**: `VendorAdapter` trait in
`Server/src/vendor.rs` (discover/control) — fill in developer keys to enable
Mi Home / Hilink / Midea / Haier / HomeKit without gateway changes.

## 4. API Token Auth

```bash
curl -X POST localhost:8080/api/token          # issue → {"token":"<64hex>"}
# config.toml: auth_enabled = true
curl -H "Authorization: Bearer <token>" localhost:8080/api/status
```

Exempt: static pages, `/api/token`, `/api/ws`. Revoke: `POST /api/token/revoke`.

## 5. Storage: SQLite + MySQL + Excel

| Layer | Description |
| --- | --- |
| SQLite | `data/pilhome.db`, incremental write every 10s, survives reboot |
| MySQL | `mysql_url` in config → auto table + incremental sync (graceful fail) |
| Excel | `GET /api/export/events?format=xlsx` — standard .xlsx (OOXML) |
| CSV | `?format=csv` — UTF-8 BOM, opens directly in Excel |

```toml
mysql_url = "mysql://user:pass@192.168.1.10:3306/pilhome"
```

## 6. Self-Test Results

- `cargo test --workspace`: ARP parse, mDNS encode/compression-pointer/response parse,
  WOL packet, MAC parse, fingerprint/OUI, CSV/XLSX structure — 27+ cases green
- `cargo build --release`: 0 errors, 0 warnings
- `npm run build`: tsc strict + Vite passed