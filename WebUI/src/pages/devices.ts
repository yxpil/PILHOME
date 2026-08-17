// 设备页:全屋设备台账 + 场景控制 + 白名单。

import { api, Device, fmtTime, system } from "../api";
import { ha } from "../api";
import { card, dot, el, empty } from "../components/ui";

export async function renderDevices(): Promise<HTMLElement> {
  const { devices } = await api.devices();
  const haInfo = await ha.entities().catch(() => null);

  const rows =
    devices.length === 0
      ? [el("tr", {}, el("td", { colSpan: 6 }, empty("暂无设备:等待 MQTT 上报 / HA 同步 / 手动注册"))) ]
      : devices.map((d) => deviceRow(d));

  // 自动发现栏。
  const discoverBar = card(
    "自动发现",
    el("div", { class: "flex" },
      el("button", {
        class: "btn",
        onclick: async () => {
          const res = await system.discover();
          alert(`发现完成:共 ${res.report.total_devices} 台(新增 ${res.report.new_devices})`);
          location.reload();
        },
      }, "扫描局域网(ARP+mDNS+端口指纹)"),
      el("button", {
        class: "btn ghost",
        onclick: async () => {
          const v = await system.vendors();
          alert(`厂商生态:${v.supported.join(" / ")}\n即时发现 ${v.total} 台设备\n${v.discovered.map((d) => `· [${d.vendor}] ${d.name}${d.ip ? " " + d.ip : ""}`).join("\n")}`);
        },
      }, "厂商设备发现"),
      el("span", { class: "spacer" }),
      el("label", { class: "muted" }, "WOL 唤醒:"),
      el("input", { class: "input", id: "wol-mac", placeholder: "MAC 如 aa:bb:cc:dd:ee:ff", style: "width:220px" }),
      el("button", {
        class: "btn ghost",
        onclick: async () => {
          const mac = (document.getElementById("wol-mac") as HTMLInputElement).value.trim();
          if (!mac) { alert("请输入 MAC"); return; }
          const res = await system.wol({ mac });
          alert(res.ok ? "WOL 魔法包已发送" : "发送失败");
        },
      }, "唤醒"),
    ),
  );

  // 全屋场景控制栏。
  const scenes = haInfo?.connected
    ? card(
        "全屋场景(HA 桥接管)",
        el("div", { class: "flex" },
          ...(["arm", "disarm", "away", "home"] as const).map((s) =>
            el("button", {
              class: "btn ghost",
              onclick: async () => {
                const res = await ha.scene(s);
                alert(`场景 ${s} 已执行:${res.executed} 个实体操作`);
              },
            }, sceneLabel(s)),
          ),
          el("span", { class: "spacer" }),
          el("span", { class: "tag active" }, `已接管 ${haInfo.entity_count} 个实体`),
        ),
      )
    : null;

  // 注册表单(保留)。
  const form = card(
    "手动注册设备",
    el("form", { class: "mt" },
      field("设备 ID", input("id", "contact_front_door")),
      field("名称", input("name", "入户门磁")),
      el("div", { class: "flex" },
        el("select", { class: "select", id: "kind" },
          ...["contact", "presence", "camera", "lock", "sensor", "actuator", "light", "switch", "climate", "cover"].map((k) =>
            el("option", { value: k }, k),
          ),
        ),
        el("button", { class: "btn", type: "submit" }, "注册"),
      ),
    ),
  );

  form.querySelector("form")!.addEventListener("submit", async (e) => {
    e.preventDefault();
    const id = (form.querySelector("#id") as HTMLInputElement).value.trim();
    const name = (form.querySelector("#name") as HTMLInputElement).value.trim() || id;
    const kind = (form.querySelector("#kind") as HTMLSelectElement).value;
    await api.upsertDevice({ id, name, kind, address: `manual:${id}` });
    location.reload();
  });

  return el("div", {},
    el("h1", { class: "page-title" }, "设备"),
    el("p", { class: "page-sub" }, `共 ${devices.length} 台设备 · ${haInfo?.connected ? `HA 桥在线(${haInfo.entity_count} 实体)` : "HA 桥未连接"}`),
    discoverBar,
    scenes,
    el("div", { class: "mt" },
      card("全屋设备台账",
        el("table", { class: "table" },
          el("thead", {}, el("tr", {},
            el("th", {}, "状态"), el("th", {}, "ID"), el("th", {}, "名称"),
            el("th", {}, "种类"), el("th", {}, "最后心跳"), el("th", {}, "控制"),
          )),
          el("tbody", {}, ...rows),
        ),
      ),
    ),
    el("div", { class: "mt" }, form),
  );
}

function sceneLabel(s: string): string {
  return ({ arm: "布防", disarm: "撤防", away: "离家", home: "回家" } as Record<string, string>)[s] ?? s;
}

function field(label: string, control: HTMLElement): HTMLElement {
  return el("div", { class: "field" }, el("label", { for: control.id }, label), control);
}

function input(id: string, placeholder: string): HTMLInputElement {
  return el("input", { class: "input", id, placeholder }) as HTMLInputElement;
}

/** 可控制设备生成控制按钮组。 */
function controlButtons(d: Device): HTMLElement | null {
  const act = (action: string) => async () => {
    await ha.control(d.id, action);
    setTimeout(() => location.reload(), 400);
  };
  switch (d.kind) {
    case "lock":
      return el("span", { class: "flex" },
        el("button", { class: "btn ghost", onclick: act("lock") }, "锁定"),
        el("button", { class: "btn ghost", onclick: act("unlock") }, "解锁"),
      );
    case "cover":
      return el("span", { class: "flex" },
        el("button", { class: "btn ghost", onclick: act("open") }, "打开"),
        el("button", { class: "btn ghost", onclick: act("close") }, "关闭"),
      );
    case "light":
    case "switch":
    case "fan":
    case "siren":
      return el("span", { class: "flex" },
        el("button", { class: "btn ghost", onclick: act("on") }, "开"),
        el("button", { class: "btn ghost", onclick: act("off") }, "关"),
        el("button", { class: "btn ghost", onclick: act("toggle") }, "切换"),
      );
    default:
      return null;
  }
}

function deviceRow(d: Device): HTMLElement {
  const stateCls = d.state === "online" ? "online" : d.state === "degraded" ? "warn" : "alert";
  const controls = controlButtons(d);
  const mac = (d.attrs as Record<string, string> | undefined)?.mac;
  const wolBtn = mac
    ? el("button", { class: "btn ghost", onclick: async () => { await system.wol({ device_id: d.id }); alert("WOL 魔法包已发送"); } }, "唤醒")
    : null;
  return el("tr", {},
    el("td", {}, dot(stateCls as "online" | "warn" | "alert"), d.state),
    el("td", { class: "mono" }, d.id),
    el("td", {}, d.name),
    el("td", {}, el("span", { class: "tag" }, d.kind)),
    el("td", { class: "mono" }, d.last_seen > 0 ? fmtTime(d.last_seen) : "—"),
    el("td", {}, el("span", { class: "flex" }, controls ?? el("span", { class: "muted" }, "只读"), wolBtn)),
  );
}