# MCP Access — Let AI Agents Control the Whole Home

PILHOME ships a built-in **MCP (Model Context Protocol)** endpoint so Claude Desktop,
Cursor, or any MCP client can query and control the home via tool calls.

## Endpoint

```
POST http://127.0.0.1:8080/api/mcp
```

JSON-RPC 2.0 subset: `initialize` / `tools/list` / `tools/call` / `ping`.

## Agent Configuration

### Claude Desktop (`claude_desktop_config.json`)

```json
{
  "mcpServers": {
    "pilhome": { "url": "http://127.0.0.1:8080/api/mcp" }
  }
}
```

### Cursor / other MCP clients

Add an MCP server, transport type HTTP (Streamable HTTP), URL
`http://127.0.0.1:8080/api/mcp`.

## Tools

| Tool | Description | Params |
| --- | --- | --- |
| `list_devices` | device registry (kind/state/MAC/vendor) | — |
| `get_status` | system state (devices/events/storage/HA) | — |
| `get_events` | recent events | `limit` |
| `run_scene` | whole-house scene | `scene` (arm/disarm/away/home) |
| `discover` | full auto-discovery | — |
| `wol_wake` | WOL wake | `mac` or `device_id` |
| `ha_control` | control HA entity | `entity_id` + `action` |
| `add_automation` | add automation rule | `name` + `action_type` … |

## Example (raw JSON-RPC)

```bash
# handshake
curl -X POST localhost:8080/api/mcp -H "Content-Type: application/json" -d '{
  "jsonrpc":"2.0","id":1,"method":"initialize","params":{}
}'

# list devices
curl -X POST localhost:8080/api/mcp -H "Content-Type: application/json" -d '{
  "jsonrpc":"2.0","id":2,"method":"tools/call",
  "params":{"name":"list_devices","arguments":{}}
}'

# arm the house
curl -X POST localhost:8080/api/mcp -H "Content-Type: application/json" -d '{
  "jsonrpc":"2.0","id":3,"method":"tools/call",
  "params":{"name":"run_scene","arguments":{"scene":"arm"}}
}'
```

## Security

- The MCP endpoint is protected by the same `auth_enabled` token middleware as other `/api/*`.
- All tool executions are written to the audit log (actor=mcp).
- Expose only on trusted networks.