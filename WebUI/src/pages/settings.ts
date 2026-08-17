// 设置页:定时规则、网络扫描、边缘模型。

import { api, automation, fmtUptime, system } from "../api";
import { card, el, empty } from "../components/ui";

export async function renderSettings(): Promise<HTMLElement> {
  const [schedules, models, status] = await Promise.all([
    api.schedules().catch(() => ({ schedules: [], total: 0 })),
    api.models().catch(() => ({ models: [], total: 0 })),
    api.status().catch(() => null),
  ]);

  const ruleRows = schedules.schedules.map((r) =>
    el("tr", {}, el("td", {}, r.name), el("td", { class: "mono" }, r.action)),
  );

  const modelRows = models.models.map((m) =>
    el("tr", {},
      el("td", {}, m.name),
      el("td", { class: "mono" }, m.kind),
      el("td", { class: "mono" }, m.format),
      el("td", {}, el("span", { class: `tag ${m.status === "ready" ? "active" : ""}` }, m.status)),
    ),
  );

  // 定时规则表单。
  const ruleForm = card(
    "新增定时规则(AUTOTIME)",
    el("form", { class: "mt" },
      field("名称", textInput("rule-name", "夜间布防")),
      field("动作", textInput("rule-action", "arm")),
      field("表达式(分 时 日 月 星期)", textInput("rule-expr", "0 23 * * *")),
      el("button", { class: "btn", type: "submit" }, "添加"),
    ),
  );
  ruleForm.querySelector("form")!.addEventListener("submit", async (e) => {
    e.preventDefault();
    const g = (id: string) => (ruleForm.querySelector(id) as HTMLInputElement).value.trim();
    await api.addSchedule({ name: g("#rule-name"), action: g("#rule-action"), expr: g("#rule-expr") });
    location.reload();
  });

  // 网络扫描卡片。
  const scanCard = card(
    "网络扫描(NetScanear)",
    el("p", { class: "muted mb" }, status ? `网关已运行 ${fmtUptime(status.system.uptime_secs)}` : ""),
    el("button", {
      class: "btn",
      onclick: async (e: Event) => {
        const btn = e.target as HTMLButtonElement;
        btn.disabled = true;
        btn.textContent = "扫描中…";
        try {
          const res = await api.scanRun();
          alert(res.report ? `扫描完成:活跃 ${res.report.alive_total} 台,陌生设备 ${res.report.unknown_total} 台` : `扫描失败:${res.error}`);
        } finally {
          btn.disabled = false;
          btn.textContent = "立即扫描";
        }
      },
    }, "立即扫描"),
  );

  const autoRows = (await automation.list().catch(() => ({ automations: [], total: 0 }))).automations.map((a) => {
    const act = a.actions[0];
    const actDesc = act ? `${act.type} ${JSON.stringify(act.args)}` : "";
    return el("tr", {},
      el("td", {}, a.name),
      el("td", { class: "mono" }, a.id),
      el("td", { class: "muted" }, `${a.trigger.device_id ?? "*"} / ${a.trigger.key ?? "*"} / ${a.trigger.value ?? "*"}`),
      el("td", { class: "muted" }, actDesc),
      el("td", {},
        el("span", { class: "flex" },
          el("button", {
            class: `btn ghost ${a.enabled ? "active" : ""}`,
            onclick: async () => { await automation.toggle(a.id); location.reload(); },
          }, a.enabled ? "启用" : "停用"),
          el("button", {
            class: "btn ghost danger",
            onclick: async () => { await automation.remove(a.id); location.reload(); },
          }, "删除"),
        ),
      ),
    );
  });

  const autoForm = card(
    "新增自动化规则(内置引擎)",
    el("form", { class: "mt" },
      field("名称", textInput("auto-name", "夜间门磁告警")),
      el("div", { class: "flex" },
        el("div", { class: "field", style: "flex:1" }, el("label", { for: "auto-device" }, "设备 ID(可空)"), textInput("auto-device", "contact_front_door")),
        el("div", { class: "field", style: "flex:1" }, el("label", { for: "auto-key" }, "事件键(可空)"), textInput("auto-key", "contact")),
      ),
      el("div", { class: "flex" },
        el("div", { class: "field", style: "flex:1" }, el("label", { for: "auto-value" }, "事件值(可空)"), textInput("auto-value", "open")),
        el("div", { class: "field", style: "flex:1" },
          el("label", { for: "auto-act" }, "动作类型"),
          el("select", { class: "select", id: "auto-act" },
            ...["log", "mqtt", "ha", "webhook"].map((k) => el("option", { value: k }, k)),
          ),
        ),
      ),
      field("动作参数(JSON,如 {\"topic\":\"x\",\"payload\":\"y\"})", textInput("auto-args", '{"entity_id":"light.living_room","action":"off"}')),
      el("button", { class: "btn", type: "submit" }, "添加规则"),
    ),
  );
  autoForm.querySelector("form")!.addEventListener("submit", async (e) => {
    e.preventDefault();
    const g = (id: string) => (autoForm.querySelector(id) as HTMLInputElement).value.trim();
    const args = JSON.parse(g("#auto-args") || "{}") as Record<string, unknown>;
    await automation.add({
      name: g("#auto-name"),
      device_id: g("#auto-device") || null,
      key: g("#auto-key") || null,
      value: g("#auto-value") || null,
      action_type: (autoForm.querySelector("#auto-act") as HTMLSelectElement).value,
      ...args,
    });
    location.reload();
  });

  const tokenCard = card(
    "API Token(开放接入鉴权)",
    el("div", { class: "flex" },
      el("button", {
        class: "btn",
        onclick: async () => {
          const res = await system.issueToken();
          await navigator.clipboard?.writeText(res.token).catch(() => {});
          alert(`新令牌已签发(已复制):\n${res.token}\n\n启用鉴权后在 config.toml 设 auth_enabled = true`);
        },
      }, "签发新令牌"),
      el("button", {
        class: "btn ghost",
        onclick: async () => { const s = await system.tokenStatus(); alert(`鉴权${s.enabled ? "已启用" : "未启用"} · 已签发 ${s.total} 个令牌`); },
      }, "令牌状态"),
      el("span", { class: "spacer" }),
      el("a", { class: "btn ghost", href: system.exportUrl("xlsx") }, "导出 Excel"),
      el("a", { class: "btn ghost", href: system.exportUrl("csv") }, "导出 CSV"),
    ),
    el("p", { class: "muted mt" }, "外部系统接入:POST /api/token 签发,请求头带 Authorization: Bearer <token>"),
  );

  const storageCard = card(
    "存储(SQLite + MySQL + 导出)",
    el("div", { class: "flex" },
      el("span", { class: "tag" }, "SQLite 本地事件库"),
      el("span", { class: "tag active" }, "MySQL 可配置"),
      el("span", { class: "tag" }, "Excel/CSV 一键导出"),
    ),
    el("p", { class: "muted mt" }, "事件每 10 秒增量落 SQLite(data/pilhome.db);config.toml 配 mysql_url 后自动同步远程 MySQL;导出走上方按钮(浏览器直接下载)。"),
  );

  return el("div", {},
    el("h1", { class: "page-title" }, "设置"),
    el("p", { class: "page-sub" }, "定时规则、自动化、网络扫描与边缘模型管理"),
    el("div", { class: "mt" }, tokenCard),
    el("div", { class: "mt" }, storageCard),
    el("div", { class: "grid cols-2" }, ruleForm, scanCard),
    el("div", { class: "mt" },
      card("自动化规则(事件触发 → 动作)",
        el("table", { class: "table" },
          el("thead", {}, el("tr", {}, el("th", {}, "名称"), el("th", {}, "ID"), el("th", {}, "触发器"), el("th", {}, "动作"), el("th", {}, "操作"))),
          el("tbody", {}, ...(autoRows.length ? autoRows : [el("tr", {}, el("td", { colSpan: 5 }, empty("暂无规则")))])),
        ),
      ),
    ),
    el("div", { class: "mt" }, autoForm),
    el("div", { class: "mt" },
      card("定时规则列表",
        el("table", { class: "table" },
          el("thead", {}, el("tr", {}, el("th", {}, "名称"), el("th", {}, "动作"))),
          el("tbody", {}, ...(ruleRows.length ? ruleRows : [el("tr", {}, el("td", { colSpan: 2 }, empty("暂无规则")))]),
          ),
        ),
      ),
    ),
    el("div", { class: "mt" },
      card("边缘模型(SideAgent)",
        el("table", { class: "table" },
          el("thead", {}, el("tr", {}, el("th", {}, "名称"), el("th", {}, "种类"), el("th", {}, "格式"), el("th", {}, "状态"))),
          el("tbody", {}, ...(modelRows.length ? modelRows : [el("tr", {}, el("td", { colSpan: 4 }, empty("暂无模型")))]),
          ),
        ),
      ),
    ),
  );
}

function field(label: string, control: HTMLElement): HTMLElement {
  return el("div", { class: "field" }, el("label", { for: control.id }, label), control);
}

function textInput(id: string, placeholder: string): HTMLInputElement {
  return el("input", { class: "input", id, placeholder }) as HTMLInputElement;
}