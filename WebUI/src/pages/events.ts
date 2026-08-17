// 事件页:统一事件时间线 + 审计日志。

import { api, fmtTime } from "../api";
import { card, el, empty } from "../components/ui";

export async function renderEvents(): Promise<HTMLElement> {
  const { events } = await api.events(200);
  const { audit } = await api.audit(50).catch(() => ({ audit: [], total: 0 }));

  const timeline: HTMLElement[] =
    events.length === 0
      ? [empty("暂无事件")]
      : events.map((e) =>
          el("div", { class: `event level-${e.level}` },
            el("span", { class: "event-time" }, fmtTime(e.ts)),
            el("span", { class: "event-source" }, e.source),
            el("span", { class: "event-msg" }, e.message),
          ),
        );

  const auditRows = audit.map((a) =>
    el("tr", {},
      el("td", { class: "mono" }, fmtTime(a.ts)),
      el("td", { class: "mono" }, a.actor),
      el("td", { class: "mono" }, a.action),
      el("td", {}, a.target),
      el("td", { class: "muted" }, a.detail),
    ),
  );

  return el("div", {},
    el("h1", { class: "page-title" }, "事件"),
    el("p", { class: "page-sub" }, "设备事件与系统审计流水"),
    card("实时事件流", el("div", { class: "timeline" }, ...timeline)),
    el("div", { class: "mt" },
      card("自我审计日志(SelflookUP)",
        el("table", { class: "table" },
          el("thead", {}, el("tr", {},
            el("th", {}, "时间"), el("th", {}, "操作者"), el("th", {}, "动作"), el("th", {}, "目标"), el("th", {}, "详情"),
          )),
          el("tbody", {}, ...(auditRows.length ? auditRows : [el("tr", {}, el("td", { colSpan: 5 }, empty("暂无审计记录")))]),
          ),
        ),
      ),
    ),
  );
}