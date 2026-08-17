# Edge AI Inference

All AI runs on the edge (local gateway): no cloud, low latency, privacy preserved.

## 1. SideAgent — Edge Model Management

- `ModelRegistry`: register/load/unload models, readiness counts
- `ModelRuntime` trait: `run(features: &[f32; 8]) -> Vec<Prediction>`
- **Rule engine** (built-in, zero-weight): template matching on normalized features,
  used for gesture inference (fist/palm/point/ok) and occupancy heuristics

## 2. Gesture Inference Lab (WebUI)

- 8 feature sliders → `POST /api/gesture` → class + confidence bar chart
- Demonstrates end-to-end edge inference pipeline (feature → model → decision)

## 3. VEScaner — Motion Detection

- Frame-difference on grayscale frames (abs diff → threshold → motion ratio)
- `POST /api/video/audit` accepts motion ratio + optional detections

## 4. YololookUP — Video Rough Audit (YOLO)

- `Detector` trait: `detect(frame) -> Vec<Detection>` — plug in any ONNX YOLO
  (yolov8n.pt → onnx), no gateway code change
- `AuditEngine`: motion + high-confidence detection → audit event (level/summary);
  periodic rough audit (one frame per interval) for cheap perimeter watching

## 5. ATOGrowUP — Behavior Learning & Rule Evolution

- Per-device 24h activity profile (event counts per hour)
- Anomaly detection: current activity vs expected band (μ ± 2σ style)
- Genetic rule evolution: population of rules with fitness (hit-rate based);
  mutation = parameter drift / toggle; elite retention keeps best rules
- `GET /api/profile` 24h curves; `POST /api/evolve/tick` one generation

## 6. Privacy & Efficiency

- All inference local; no video leaves the LAN
- Rule engine cost ≈ 0 CPU; YOLO optional (model file dropped in `models/`)
- Frame-difference cheap enough for always-on motion watching