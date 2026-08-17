# MCP 接入 —— 让 AI 智能体操控全屋

PILHOME 内置 **MCP (Model Context Protocol)** 端点,支持 Claude Desktop、Cursor、
以及任意 MCP 客户端直接接入,以工具调用方式查询/控制全屋设备。

## 一、端点

```
POST http://127.0.0.1:8080/api/mcp
```

JSON-RPC 2.0 子集:`initialize` / `tools/list` / `tools/call` / `ping`。

## 二、智能体配置

### Claude Desktop(`claude_desktop_config.json`)

```json
{
  "mcpServers": {
    "pilhome": { "url": "http://127.0.0.1:8080/api/mcp" }
  }
}
```

### Cursor / 其他 MCP 客户端

新增 MCP Server,类型选 HTTP(Streamable HTTP),URL 填
`http://127.0.0.1:8080/api/mcp`。

## 三、工具清单

| 工具 | 说明 | 参数 |
| --- | --- | --- |
| `list_devices` | 设备台账(类型/状态/MAC/厂商) | — |
| `get_status` | 系统状态(设备/事件/存储/HA 连接) | — |
| `get_events` | 最近事件 | `limit` |
| `run_scene` | 全屋场景 | `scene`(arm/disarm/away/home) |
| `discover` | 全自动设备发现 | — |
| `wol_wake` | WOL 唤醒 | `mac` 或 `device_id` |
| `ha_control` | 控制 HA 实体 | `entity_id` + `action` |
| `add_automation` | 新增自动化规则 | `name` + `action_type` 等 |

## 四、调用示例(原始 JSON-RPC)

```bash
# 握手
curl -X POST localhost:8080/api/mcp -H "Content-Type: application/json" -d '{
  "jsonrpc":"2.0","id":1,"method":"initialize","params":{}
}'

# 查设备
curl -X POST localhost:8080/api/mcp -H "Content-Type: application/json" -d '{
  "jsonrpc":"2.0","id":2,"method":"tools/call",
  "params":{"name":"list_devices","arguments":{}}
}'

# 布防
curl -X POST localhost:8080/api/mcp -H "Content-Type: application/json" -d '{
  "jsonrpc":"2.0","id":3,"method":"tools/call",
  "params":{"name":"run_scene","arguments":{"scene":"arm"}}
}'
```

## 五、安全

- MCP 端点与其他 `/api/*` 一致,受 `auth_enabled` 令牌鉴权保护;
- 工具执行全部写入审计日志(actor=mcp);
- 建议仅内网暴露。