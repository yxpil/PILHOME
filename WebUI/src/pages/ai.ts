// 智能页:手势推理实验室 + 行为画像可视化 + 规则进化器。

import { ai, fmtTime } from "../api";
import { card, el, empty } from "../components/ui";

export async function renderAI(): Promise<HTMLElement> {
  const [profile, evolve, health] = await Promise.all([
    ai.profile().catch(() => null),
    ai.evolve().catch(() => null),
    ai.health().catch(() => null),
  ]);

  // ---------- 手势推理实验室 ----------
  const sliders: HTMLInputElement[] = [];
  for (let i = 0; i < 8; i++) {
    sliders.push(el("input", { class: "input", type: "range", min: "0", max: "1", step: "0.01", value: "0.5" }) as HTMLInputElement);
  }
  const resultBox = el("div", { class: "muted mt" }, "拖动 8 个特征滑条,点击「推理」查看手势分类结果");

  const gestureCard = card(
    "手势推理实验室(SideAgent 规则引擎)",
    el("div", { class: "grid cols-2" },
      ...sliders.map((s, i) => el("div", { class: "field" }, el("label", {}, `特征 ${i + 1}`), s)),
    ),
    el("button", {
      class: "btn",
      onclick: async () => {
        const features = sliders.map((s) => Number(s.value));
        const res = await ai.gesture(features);
        const top = res.top;
        resultBox.innerHTML = "";
        resultBox.append(
          el("div", { class: "flex mb" },
            el("span", { class: "tag active" }, `识别:${top?.label ?? "未知"}`),
            el("span", { class: "mono muted" }, `置信度 ${(top?.confidence ?? 0).toFixed(3)} · 耗时 ${res.latency_ms.toFixed(2)}ms`),
          ),
          ...res.results.map((r) =>
            el("div", { class: "flex" },
              el("span", { class: "tag" }, r.label),
              el("div", { class: "bar-track" }, el("div", { class: "bar", style: `width:${Math.round(r.confidence * 100)}%` })),
            ),
          ),
        );
      },
    }, "推理"),
    resultBox,
  );

  // ---------- 行为画像可视化 ----------
  const curveCards = Object.entries(profile?.curves ?? {}).length
    ? Object.entries(profile!.curves).map(([device, curve]) =>
        card(
          `活跃画像 · ${device}`,
          el("div", { class: "bars" }, ...curve.map((v) => el("div", { class: "bar-col" }, el("div", { class: "bar", style: `height:${Math.max(4, Math.round(v * 120))}%` })))),
          el("div", { class: "muted" }, "24 小时活跃曲线(自动学习家庭作息)"),
        ),
      )
    : [empty("暂无画像数据:等待设备事件上报后自动生成")];

  const suggestions = (profile?.suggestions ?? []).map((s) =>
    el("div", { class: "event" },
      el("span", { class: "event-source" }, s.severity),
      el("span", { class: "event-msg" }, `${s.title}:${s.detail}`),
    ),
  );

  // ---------- 规则进化器 ----------
  const genBox = el("div", { class: "muted mt" }, evolve ? `第 ${evolve.generation} 代 · ${evolve.total} 条规则` : "尚未初始化");
  const evolveRows = (evolve?.population ?? []).map((g) =>
    el("tr", {},
      el("td", { class: "mono" }, g.id),
      el("td", {}, el("span", { class: "tag" }, g.kind)),
      el("td", { class: "mono" }, `${String(g.start_hour).padStart(2, "0")}:00 → ${String(g.end_hour).padStart(2, "0")}:00`),
      el("td", { class: "mono" }, g.fitness.toFixed(2)),
      el("td", { class: "mono" }, `G${g.generation}`),
    ),
  );

  const evolveCard = card(
    "规则进化器(ATOGrowUP 自发成长和变异)",
    el("div", { class: "flex" },
      el("button", {
        class: "btn",
        onclick: async () => { await ai.evolveSeed(); location.reload(); },
      }, "注入初始种群"),
      el("button", {
        class: "btn ghost",
        onclick: async () => { await ai.evolveTick(); location.reload(); },
      }, "进化一代"),
    ),
    genBox,
    el("table", { class: "table mt" },
      el("thead", {}, el("tr", {}, el("th", {}, "ID"), el("th", {}, "种类"), el("th", {}, "时段"), el("th", {}, "适应度"), el("th", {}, "代数"))),
      el("tbody", {}, ...(evolveRows.length ? evolveRows : [el("tr", {}, el("td", { colSpan: 5 }, empty("先注入初始种群")))])),
    ),
  );

  // ---------- 健康报告 ----------
  const healthRows = (health?.report.items ?? []).map((i) =>
    el("tr", {},
      el("td", { class: "mono" }, i.name),
      el("td", {}, el("span", { class: `tag ${i.status === "ok" ? "active" : ""}` }, i.status)),
      el("td", {}, i.detail),
      el("td", { class: "mono muted" }, fmtTime(i.checked_at)),
    ),
  );

  return el("div", {},
    el("h1", { class: "page-title" }, "智能"),
    el("p", { class: "page-sub" }, "边缘 AI 推理 · 行为自学习 · 自我审计"),
    el("div", { class: "grid cols-2" }, gestureCard, evolveCard),
    el("div", { class: "mt" },
      card("行为画像与异常建议(ATOGrowUP)", el("div", { class: "grid cols-2" }, ...curveCards), el("div", { class: "timeline mt" }, ...suggestions)),
    ),
    el("div", { class: "mt" },
      card("健康自检报告(SelflookUP)",
        el("table", { class: "table" },
          el("thead", {}, el("tr", {}, el("th", {}, "检查项"), el("th", {}, "状态"), el("th", {}, "详情"), el("th", {}, "时间"))),
          el("tbody", {}, ...(healthRows.length ? healthRows : [el("tr", {}, el("td", { colSpan: 4 }, empty("健康服务不可用")))])),
        ),
      ),
    ),
  );
}