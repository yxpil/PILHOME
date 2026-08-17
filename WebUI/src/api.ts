// 网关 REST API 封装(全部类型静态声明,严格模式)。

export interface Device {
  id: string;
  name: string;
  kind: string;
  address: string;
  state: "online" | "degraded" | "offline";
  last_seen: number;
  attrs: Record<string, string>;
  trusted: boolean;
}

export interface AppEvent {
  ts: number;
  level: "info" | "warn" | "alert";
  source: string;
  message: string;
  device_id: string | null;
  data: Record<string, unknown> | null;
}

export interface ModuleStatus {
  id: string;
  label: string;
  version: string;
  status: string;
  metric: string;
}

export interface SystemStatus {
  system: { name: string; version: string; uptime_secs: number; now: number };
  overview: {
    devices_total: number;
    devices_online: number;
    events: number;
    audit: number;
    models_ready: number;
  };
  modules: ModuleStatus[];
}

export interface ScanReport {
  elapsed_ms: number;
  alive_total: number;
  known_total: number;
  unknown_total: number;
  unknown: { ip: string; open_ports: number[]; hostname: string }[];
}

export interface AuditEntry {
  ts: number;
  actor: string;
  action: string;
  target: string;
  detail: string;
}

export interface ModelInfo {
  id: string;
  name: string;
  kind: string;
  format: string;
  path: string;
  status: string;
  avg_latency_ms: number;
}

export interface ScheduleRule {
  name: string;
  action: string;
}

export interface RhythmResp {
  curve: number[];
  recommended_arm_windows: [number, number][];
}

async function req<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(path, {
    headers: { "Content-Type": "application/json" },
    ...init,
  });
  if (!res.ok) {
    const body = await res.json().catch(() => ({}));
    throw new Error((body as { error?: string }).error ?? `HTTP ${res.status}`);
  }
  return res.json() as Promise<T>;
}

export const api = {
  status: () => req<SystemStatus>("/api/status"),
  devices: () => req<{ devices: Device[]; total: number }>("/api/devices"),
  upsertDevice: (body: { id: string; name: string; kind?: string; address?: string }) =>
    req<{ ok: boolean }>("/api/devices", { method: "POST", body: JSON.stringify(body) }),
  setTrust: (id: string, trusted: boolean) =>
    req<{ ok: boolean }>(`/api/devices/${id}/trust`, { method: "POST", body: JSON.stringify({ trusted }) }),
  events: (limit = 100) => req<{ events: AppEvent[]; total: number }>(`/api/events?limit=${limit}`),
  scanRun: () => req<{ ok: boolean; report?: ScanReport; error?: string }>("/api/scan/run", { method: "POST" }),
  models: () => req<{ models: ModelInfo[]; total: number }>("/api/models"),
  schedules: () => req<{ schedules: ScheduleRule[]; total: number }>("/api/schedules"),
  addSchedule: (body: { name: string; action: string; expr: string }) =>
    req<{ ok: boolean }>("/api/schedules", { method: "POST", body: JSON.stringify(body) }),
  rhythm: () => req<RhythmResp>("/api/rhythm"),
  audit: (limit = 100) => req<{ audit: AuditEntry[]; total: number }>(`/api/audit?limit=${limit}`),
};

/** Unix 秒 -> 本地时间字符串。 */
export function fmtTime(ts: number): string {
  const d = new Date(ts * 1000);
  const p = (n: number) => String(n).padStart(2, "0");
  return `${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
}

/** 秒 -> 可读时长。 */
export function fmtUptime(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0) return `${h} 小时 ${m} 分`;
  return `${m} 分钟`;
}
/** 手势推理结果。 */
export interface GestureResult {
  ok: boolean;
  latency_ms: number;
  top: { label: string; confidence: number } | null;
  results: { label: string; confidence: number; bbox: null }[];
}

/** 行为画像响应。 */
export interface ProfileResp {
  total_events: number;
  curves: Record<string, number[]>;
  suggestions: { severity: string; device_id: string; title: string; detail: string }[];
}

/** 规则种群。 */
export interface EvolveResp {
  generation: number;
  population: { id: string; kind: string; start_hour: number; end_hour: number; fitness: number; generation: number }[];
  total: number;
}

/** 健康报告。 */
export interface HealthResp {
  ok: boolean;
  report: {
    generated_at: number;
    overall: "ok" | "warn" | "critical";
    items: { name: string; status: string; detail: string; checked_at: number }[];
  };
}
export const ai = {
  gesture: (features: number[]) =>
    req<GestureResult>("/api/gesture", { method: "POST", body: JSON.stringify({ features }) }),
  profile: () => req<ProfileResp>("/api/profile"),
  evolve: () => req<EvolveResp>("/api/evolve"),
  evolveSeed: () => req<{ ok: boolean }>("/api/evolve/seed", { method: "POST" }),
  evolveTick: () => req<{ ok: boolean; population: number; generation: number }>("/api/evolve/tick", { method: "POST" }),
  health: () => req<HealthResp>("/api/health"),
  videoProbe: (body: { prefix: string; rtsp_port?: number; onvif_port?: number }) =>
    req<{ ok: boolean; total: number; devices: { id: string; name: string; rtsp_url: string }[] }>("/api/video/probe", { method: "POST", body: JSON.stringify(body) }),
};
/** HA 实体。 */
export interface HaEntityInfo {
  entity_id: string;
  name: string;
  domain: string;
  state: string;
  attributes: Record<string, unknown>;
}

/** HA 桥状态。 */
export interface HaEntitiesResp {
  enabled: boolean;
  connected: boolean;
  entity_count: number;
  last_sync: number;
  entities: HaEntityInfo[];
}

export const ha = {
  entities: () => req<HaEntitiesResp>("/api/ha/entities"),
  control: (entity_id: string, action: string) =>
    req<{ ok: boolean }>("/api/ha/control", { method: "POST", body: JSON.stringify({ entity_id, action }) }),
  sync: () => req<{ ok: boolean }>("/api/ha/sync", { method: "POST" }),
  scene: (scene: "arm" | "disarm" | "away" | "home") =>
    req<{ ok: boolean; executed: number }>("/api/ha/scenes", { method: "POST", body: JSON.stringify({ scene }) }),
};
/** 自动化规则。 */
export interface AutomationRule {
  id: string;
  name: string;
  enabled: boolean;
  trigger: { device_id: string | null; key: string | null; value: string | null; min_level: string | null };
  window: [number, number] | null;
  actions: { type: string; args: Record<string, unknown> }[];
}

export const automation = {
  list: () => req<{ automations: AutomationRule[]; total: number }>("/api/automations"),
  add: (body: Record<string, unknown>) => req<{ ok: boolean; rule: AutomationRule }>("/api/automations", { method: "POST", body: JSON.stringify(body) }),
  toggle: (id: string) => req<{ ok: boolean; enabled: boolean }>(`/api/automations/${id}/toggle`, { method: "POST" }),
  remove: (id: string) => req<{ ok: boolean }>(`/api/automations/${id}`, { method: "DELETE" }),
};
/** 发现/唤醒/导出接口。 */
export const system = {
  discover: () => req<{ ok: boolean; report: Record<string, unknown> }>("/api/discover", { method: "POST" }),
  arp: () => req<{ entries: { ip: string; mac: string; kind: string }[]; total: number }>("/api/arp"),
  vendors: () => req<{ supported: string[]; discovered: { name: string; vendor: string; ip: string | null }[]; total: number }>("/api/vendors"),
  wol: (body: { mac?: string; device_id?: string }) => req<{ ok: boolean }>("/api/wol", { method: "POST", body: JSON.stringify(body) }),
  issueToken: () => req<{ ok: boolean; token: string; total: number }>("/api/token", { method: "POST" }),
  tokenStatus: () => req<{ enabled: boolean; total: number }>("/api/token/status"),
  exportUrl: (fmt: "xlsx" | "csv") => `/api/export/events?format=${fmt}`,
};
/** AI / 工具 / 媒体接口。 */
export interface ChatMsg { role: string; content: string }
export const aiChat = {
  chat: (message: string, history: ChatMsg[]) => req<{ ok: boolean; reply: string; compressed?: boolean; error?: string }>("/api/ai/chat", { method: "POST", body: JSON.stringify({ message, history }) }),
  analyze: (event: Record<string, unknown>, context?: string) => req<{ ok: boolean; analysis: string }>("/api/ai/analyze", { method: "POST", body: JSON.stringify({ event, context: context ?? "" }) }),
  compress: (history: ChatMsg[]) => req<{ ok: boolean; summary: string }>("/api/ai/compress", { method: "POST", body: JSON.stringify(history) }),
  review: () => req<{ ok: boolean; review: string }>("/api/ai/review", { method: "POST" }),
  status: () => req<{ configured: boolean; base_url: string; model: string; vision_model: string }>("/api/ai/status"),
};

export interface Tool { id: string; name: string; description: string; kind: "builtin" | "js"; js_code?: string | null; last_error?: string | null; last_run: number; run_count: number }
export const tools = {
  list: () => req<{ tools: Tool[]; total: number }>("/api/tools"),
  search: (q: string) => req<{ tools: Tool[]; total: number }>(`/api/tools/search?q=${encodeURIComponent(q)}`),
  create: (body: { name: string; description: string; code: string }) => req<{ ok: boolean; tool: Tool }>("/api/tools", { method: "POST", body: JSON.stringify(body) }),
  run: (id: string, args: Record<string, unknown>) => req<{ ok: boolean; result: unknown; error?: string }>(`/api/tools/${id}/run`, { method: "POST", body: JSON.stringify({ args }) }),
  debug: (id: string) => req<{ ok: boolean; advice: string }>(`/api/tools/${id}/debug`, { method: "POST" }),
  remove: (id: string) => req<{ ok: boolean }>(`/api/tools/${id}`, { method: "DELETE" }),
};

export const media = {
  discover: () => req<{ devices: { name: string; vendor: string; class: string; service: string; ip: string | null; port: number | null }[]; audio_count: number; cast_count: number; stb_count: number }>("/api/media/discover"),
  dial: (body: { ip: string; port: number; app: string; payload?: string }) => req<{ ok: boolean; status?: string; error?: string }>("/api/media/dial", { method: "POST", body: JSON.stringify(body) }),
};

export const bayes = {
  status: () => req<{ samples: number; positive: number; negative: number; features: number }>("/api/bayes/status"),
  predict: (features: [string, string][]) => req<{ probability: number; verdict: string; samples: number }>("/api/bayes/predict", { method: "POST", body: JSON.stringify({ features }) }),
};