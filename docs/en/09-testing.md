# PILHOME 测试说明

- 测试完成：部分（2026-10-04）
- 测试日期：2026-10-04
- 测试内容：纯 PlugsCore crate 单元测试（netscanear ARP/mDNS 解析、netlinker MAC/WOL、sideagent 手势规则、atogrowup、vescaner、autotime 等）；本次新增 netlinker MAC 注入测试（shell 元字符/路径穿越/XSS/空白注入被拒或净化不 panic）。**未完成**：pilhome-server 因预存在缺 `Server/src/api/token.rs` 无法编译，其 REST/自动化钩子/MQTT 事件层测试未覆盖。
- 运行命令：`cargo test -p <crate>`（workspace 整体 `cargo test --workspace` 因 server 缺文件失败）
- 测试框架：Rust `#[cfg(test)]`
- 模型：豆包（Doubao）生成

# Testing & Verification

## 1. Automated Tests (all green)

```powershell
cargo test --workspace
```

| Crate | Cases | Coverage |
| --- | --- | --- |
| pilhome-netscanear | 10 | ARP parsing, MAC normalization, mDNS name encode/compression-pointer/response parse, fingerprint, OUI vendor |
| pilhome-sideagent | 4 | rule-engine gesture classification |
| pilhome-atogrowup | 4 | profile anomaly, evolution |
| pilhome-vescaner | 3 | frame-difference motion |
| pilhome-autotime | 2 | cron matching (weekday/star) |
| pilhome-netlinker | 2 | WOL packet structure, MAC parsing |
| pilhome-server | 4 | CSV BOM/rows, XLSX zip validity, column names, vendor classification |

## 1.1 Injection tests added

- `pilhome-netlinker` (`wol::tests::hostile_mac_inputs_are_rejected_or_sanitized_never_panic`):
  MAC strings come from untrusted API/CLI input. Shell-metacharacter payloads
  (`; rm -rf /`, `&& calc.exe`), path traversal (`../../etc/passwd`), XSS
  (`<script>`), empty/whitespace/newline input are all rejected (`None`) or
  sanitized to a clean 6-byte MAC — never panic, never out-of-bounds.

## 1.2 Known pre-existing build blocker (NOT introduced by tests)

- `pilhome-server` does **not** compile as committed on `main`:
  `Server/src/api/mod.rs` declares `pub mod token;` and routes `/api/token*` to
  `token::{issue,status,revoke}`, but `Server/src/api/token.rs` was never
  committed. Fixing it requires writing that source module (out of scope for a
  test-only change). The pure `plugs-core` crates (netlinker/netscanear/...) build
  and test independently with `cargo test -p <crate>`.

## 2. Build Verification

```powershell
cargo build --release    # 0 errors, 0 warnings (12 crates)
cd WebUI && npm run build # tsc --noEmit strict + Vite
```

## 3. Manual Verification (on the machine)

| Item | Steps |
| --- | --- |
| Boot | run exe → logs show auto-discovery + module startup |
| Discovery | `POST /api/discover` → devices appear with fingerprint/vendor attrs |
| ARP | `GET /api/arp` → IP/MAC list |
| mDNS | `GET /api/vendors` → vendor-classified devices |
| WOL | `POST /api/wol` → packet sent (audit log records it) |
| Events | MQTT publish → event appears in `/api/events` and WS stream |
| Persistence | reboot → `data/events.json` + `data/pilhome.db` restored |
| SQLite | `data/pilhome.db` exists, `sqlite3 pilhome.db "select count(*) from events"` grows |
| MySQL | configure `mysql_url` → table `pilhome_events` auto-created, rows sync |
| Export | `/api/export/events?format=xlsx` → opens in Excel with all rows |
| HA bridge | configure token → `/api/ha/entities` returns entities; scene executes |
| Automation | add rule → trigger event → action fires (log/mqtt/ha/webhook) |
| Auth | `auth_enabled=true` → requests without Bearer get 401 |

## 4. Real Execution Evidence

- `cargo test --workspace`: 27+ cases, 0 failed
- `cargo build --release`: Finished, 0 warnings (own code)
- `npm run build`: tsc strict passed, dist generated