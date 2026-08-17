// 总览页:系统指标 + 模块状态 + 节律建议。

import { api, fmtUptime } from "../api";
import { card, el, empty } from "../components/ui";

export async function renderDashboard(): Promise<HTMLElement> {
  const status = await api.status();

  const stats = [
    { label: "设备总数", value: String(status.overview.devices_total) },
    { label: "在线设备", value: String(status.overview.devices_online) },
    { label: "事件记录", value: String(status.overview.events) },
    { label: "就绪模型", value: String(status.overview.models_ready) },
  ].map((s) =>
    el("div", { class: "card" }, el("div", { class: "stat-value" }, s.value), el("div", { class: "stat-label" }, s.label)),
  );

  const rhythm = await api.rhythm().catch(() => null);
  const rhythmCard = rhythm
    ? card(
        "昼夜节律 · 推荐布防时段",
        el("div", {}, ...(rhythm.recommended_arm_windows.length > 0
            ? rhythm.recommended_arm_windows
                .map(([s, e]) => el("div", { class: "flex mb" }, el("span", { class: "tag active" }, `布防 ${String(s).padStart(2, "0")}:00 → ${String(e).padStart(2, "0")}:00`)))
            : [empty("样本不足,暂无推荐")])),
        el("div", { class: "muted mt" }, "节律曲线由 AUTOTIME 依据家庭活跃画像自动生成"),
      )
    : empty("节律服务不可用");

  const moduleRows = status.modules.map((m) =>
    el("tr", {},
      el("td", {}, el("span", { class: "flex" }, dotFor(m.status), m.label)),
      el("td", { class: "mono" }, m.metric),
    ),
  );

  return el("div", {},
    el("h1", { class: "page-title" }, "总览"),
    el("p", { class: "page-sub" }, `${status.system.name} v${status.system.version} · 已运行 ${fmtUptime(status.system.uptime_secs)}`),
    el("div", { class: "grid" }, ...stats),
    el("div", { class: "grid cols-2" }, rhythmCard,
      card("核心模块状态",
        el("table", { class: "table" },
          el("thead", {}, el("tr", {}, el("th", {}, "模块"), el("th", {}, "指标"))),
          el("tbody", {}, ...moduleRows),
        ),
      ),
    ),
  );
}

function dotFor(status: string): HTMLElement {
  const { dot } = { dot: (cls: string) => el("span", { class: `dot ${cls}` }) };
  return dot(status.includes("运行") || status.includes("就绪") ? "online" : "warn");
}