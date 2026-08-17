// 工具:搜索 / 折叠列表 / 创建 JS 工具 / 执行 / 报错分析。

import { tools, Tool } from "../api";
import { card, el, empty } from "../components/ui";

export async function renderTools(): Promise<HTMLElement> {
  const toolList = await tools.list().catch(() => ({ tools: [] as Tool[], total: 0 }));
  const listEl = el("div", {});

  async function refresh() {
    const q = (document.getElementById("tool-q") as HTMLInputElement)?.value ?? "";
    const res = q ? await tools.search(q) : await tools.list();
    render(res.tools);
  }

  function render(ts: Tool[]) {
    listEl.replaceChildren();
    if (!ts.length) { listEl.append(empty("无匹配工具")); return; }
    for (const t of ts) {
      listEl.append(toolItem(t, refresh));
    }
  }

  const createForm = card(
    "创建 JS 工具(AI 自己写代码定流程)",
    el("form", { class: "mt" },
      el("div", { class: "flex" },
        el("div", { class: "field", style: "flex:1" }, el("label", { for: "t-name" }, "工具名"), inputOf("t-name", "夜间安防巡检")),
        el("div", { class: "field", style: "flex:1" }, el("label", { for: "t-desc" }, "描述(供搜索)"), inputOf("t-desc", "晚上 10 点后门磁触发时分析并上报")),
      ),
      el("div", { class: "field" },
        el("label", { for: "t-code" }, "JS 代码(可用 API:log/listDevices/deviceCommand/publishMqtt/wol/bayesPredict/httpGet/sendEvent/tts/generateImage/getState/setState,参数 ARGS)"),
        el("textarea", { class: "input code", id: "t-code", rows: 7 },
          'const conf = bayesPredict(JSON.stringify([["action","contact.open"],["hour",String(new Date().getHours())]]));\nif (parseFloat(conf) > 0.6) {\n  sendEvent("alert", "js.tool", "夜间门磁触发,用户习惯置信度 " + conf);\n  log("巡检完成");\n} else {\n  log("置信度不足,仅记录");\n}',
        ),
      ),
      el("button", { class: "btn", type: "submit" }, "创建工具"),
    ),
  );
  createForm.querySelector("form")!.addEventListener("submit", async (e) => {
    e.preventDefault();
    const g = (id: string) => (createForm.querySelector(id) as HTMLInputElement | HTMLTextAreaElement).value.trim();
    await tools.create({ name: g("#t-name"), description: g("#t-desc"), code: g("#t-code") });
    refresh();
  });

  return el("div", {},
    el("h1", { class: "page-title" }, "工具"),
    el("p", { class: "page-sub" }, `共 ${toolList.total} 个工具 · 内置 + JS 动态工具 · 点击展开执行`),
    card("搜索工具",
      el("div", { class: "flex" },
        inputOf("tool-q", "搜索名称/描述(如 设备/唤醒/AI)"),
        el("button", { class: "btn", onclick: refresh }, "搜索"),
      ),
    ),
    el("div", { class: "mt" }, listEl),
    el("div", { class: "mt" }, createForm),
  );
}

function toolItem(t: Tool, refresh: () => void): HTMLElement {
  const body = el("div", { class: "tool-body" },
    el("p", { class: "muted" }, t.description),
    t.js_code ? el("pre", { class: "code" }, t.js_code) : null,
    el("p", { class: "muted" }, `执行 ${t.run_count} 次${t.last_error ? ` · 最近报错:${t.last_error}` : ""}`),
    el("div", { class: "flex mt" },
      el("button", { class: "btn", onclick: async () => {
        const args = JSON.parse((body.querySelector("#t-args") as HTMLInputElement)?.value || "{}");
        const res = await tools.run(t.id, args);
        alert(res.ok ? `结果:\n${JSON.stringify(res.result, null, 2)}` : `执行失败:\n${res.error}`);
      } }, "执行"),
      el("input", { class: "input mono", id: "t-args", placeholder: '参数 JSON,如 {"id":"light.living_room","action":"off"}', style: "flex:1" }),
      t.kind === "js" ? el("button", { class: "btn ghost", onclick: async () => { const r = await tools.debug(t.id); alert(r.ok ? `AI 修复建议:\n${r.advice}` : "调试失败:无报错记录"); } }, "AI 报错分析") : null,
      t.kind === "js" ? el("button", { class: "btn ghost danger", onclick: async () => { await tools.remove(t.id); refresh(); } }, "删除") : null,
    ),
  );
  body.style.display = "none";
  const head = el("button", {
    class: "btn ghost tool-head",
    onclick: () => { body.style.display = body.style.display === "none" ? "" : "none"; },
  }, `${t.name} · ${t.id} · ${t.kind === "js" ? "JS" : "内置"}`);
  return el("div", { class: "tool-item" }, head, body);
}

function inputOf(id: string, placeholder: string): HTMLInputElement {
  return el("input", { class: "input", id, placeholder }) as HTMLInputElement;
}