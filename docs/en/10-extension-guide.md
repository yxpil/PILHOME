# Extension Guide

## 1. Add a New Device Type

1. `PlugsCores/NetLinker/src/device.rs` — add `DeviceKind` variant + `as_str()`
2. `Server/src/ha_bridge.rs` — map HA domain → new kind
3. WebUI `devices.ts` — control buttons for the new kind

## 2. Add a Real YOLO Model

1. Export model: `yolo export model=yolov8n.pt format=onnx`
2. Put `yolov8n.onnx` under `models/`
3. Implement `Detector` trait (`PlugsCores/YololookUP/src/detect.rs`) loading the ONNX
4. Wire into `AuditEngine` — no gateway changes needed

## 3. Add a New REST Endpoint

1. `Server/src/api/xxx.rs` — `pub async fn handler(State(state): State<Arc<AppState>>) -> …`
2. `Server/src/api/mod.rs` — `pub mod xxx;` + `.route("/api/xxx", get(xxx::handler))`
3. `WebUI/src/api.ts` — add typed client; page consumes it

## 4. Add a New Periodic Task

1. `Server/src/tasks.rs` — new `spawn_xxx(state, …)` with `tokio::time::interval`
2. Call it in `main.rs` section 4
3. Log via `state.events.record(...)` for visibility

## 5. Vendor Cloud SDK (Xiaomi/Huawei/Midea/Haier/HomeKit)

1. Implement `VendorAdapter` (`Server/src/vendor.rs`):
   `vendor() / discover() / control(device_id, action)`
2. Instantiate in `main.rs` and merge discovered devices into the registry
3. That's it — WebUI/API/automation treat them as normal devices

## 6. Extend Home Assistant Integration

- New platform mapping → `ha_bridge.rs` `kind_from_domain`
- New scene → `api/ha.rs` `scenes` handler
- HA automations can consume PILHOME via WS stream or MQTT

## 7. Add Storage Backend

- Implement like `MysqlSync` (`Server/src/storage.rs`):
  a struct with `enabled()` + `sync_events(&[AppEvent]) -> Result<usize, String>`
- Register in `tasks.rs` storage task

## 8. UI Design Rules (project-wide)

- Black-white minimalist, rounded (capsule) — no emoji
- Use `el()` + `card()` primitives from `components/ui.ts`
- Keep pages data-driven via `api.ts` typed clients