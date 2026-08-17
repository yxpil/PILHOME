# PILHOME

**基于物联网 + AI 边缘计算的智能家居安全监控系统**

> 选题审核状态:审核通过(指导教师:王海涛 / 人工智能学院)

## 一句话

一套**本地优先、可自进化、Home Assistant 深度联动**的智能家居安全监控系统:
异构设备统一接入 → 边缘 AI 推理(检测/手势/视频审计)→ 行为画像与规则变异 → 自我审计闭环 → 黑白简约 Web 控制端。

**全屋接管**:通过 Home Assistant 双向桥(WebSocket),把 HA 上千种设备全量同步进统一台账,并在 PILHOME 里直接控制(开关/锁门/窗帘/场景一键布防)。

**独立闭环 + 开放接入**:内置自动化引擎(事件触发 → 动作)、WebSocket 实时事件流、MQTT 发布接口、**MCP 端点(智能体直连操控全屋)**——HA 有的能力 PILHOME 自己也有,并允许外部系统接入。

## 架构

```
Core(负责协调总体)
  ├── PlugsCores/NetLinker    网络设备连接程序
  ├── PlugsCores/NetScanear   网络设备发现程序
  ├── PlugsCores/SideAgent    边缘AI模型
  ├── PlugsCores/HandModel    模块控制程序
  ├── PlugsCores/VEScaner     视频设备发现程序
  ├── PlugsCores/YololookUP   视频粗略代审计模块
  ├── PlugsCores/SelflookUP   自我审计程序
  ├── PlugsCores/ATOGrowUP    自发成长和变异程序
  └── PlugsCores/AUTOTIME     定时和节律控制
Server(负责管理网页和内核桥接的服务)+ WebUI(网页和控制端)
```

## 技术栈

| 端 | 技术 |
| --- | --- |
| 内核/服务 | Rust(静态编译)· tokio · axum · MQTT · serde |
| 控制端 | TypeScript(严格模式)· Vite · 零框架 · 黑白圆角设计 |
| 数据 | JSON 快照持久化(事件/审计/画像)· Webhook 告警推送 |
| 集成 | Home Assistant 双向桥:全屋实体同步 + 服务调用 + MQTT Discovery |

## 快速开始

```powershell
# WebUI
cd WebUI
npm install
npm run build

# 内核测试与构建
cd ..
cargo test --workspace
cargo build --release

# 运行网关(默认 127.0.0.1:8080)
.\target\release\pilhome-server.exe config.toml
```

打开 http://127.0.0.1:8080 即可进入控制端。

控制端页面:**总览**(指标/模块/节律)、**设备**(台账/白名单)、**事件**(事件流/审计)、**设置**(定时规则/扫描/模型)、**智能**(手势推理实验室/画像可视化/规则进化/健康自检)。

## 文档

**中英双语**:中文版见 `docs/`(12 篇);英文版见 `docs/en/`(同名 12 篇)+ `README.en.md`。

| 文档 | 内容 |
| --- | --- |
| [docs/01-选题说明.md](docs/01-选题说明.md) | 毕设选题、目标、创新点、进度 |
| [docs/02-总体架构.md](docs/02-总体架构.md) | 分层架构、技术选型、数据流 |
| [docs/03-模块设计.md](docs/03-模块设计.md) | 各模块职责与关键接口 |
| [docs/04-HomeAssistant集成.md](docs/04-HomeAssistant集成.md) | MQTT Discovery、自动化联动 |
| [docs/05-边缘AI推理.md](docs/05-边缘AI推理.md) | 模型管理、YOLO 审计、规则变异 |
| [docs/06-安全设计.md](docs/06-安全设计.md) | 威胁模型、纵深防御、审计 |
| [docs/07-部署指南.md](docs/07-部署指南.md) | 构建、配置、运行、排障 |
| [docs/08-API参考.md](docs/08-API参考.md) | REST 端点与示例 |
| [docs/09-测试与验证.md](docs/09-测试与验证.md) | 单测清单、构建验证、手动验证 |
| [docs/10-扩展指南.md](docs/10-扩展指南.md) | 接入设备/模型/端点/任务的扩展方法 |
| [docs/11-自动化与开放接入.md](docs/11-自动化与开放接入.md) | 内置自动化引擎、WebSocket 实时流、开放接入 |
| [docs/12-网络发现与厂商生态.md](docs/12-网络发现与厂商生态.md) | 局域网发现、WOL、厂商 SDK、Token、存储 |
| [docs/13-MCP接入.md](docs/13-MCP接入.md) | MCP 智能体接入(Claude/Cursor 直连操控全屋) |