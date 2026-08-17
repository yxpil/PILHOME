// 媒体:音响 / 机顶盒 / 投屏发现 + DIAL 投屏。

import { media } from "../api";
import { card, el, empty } from "../components/ui";

export async function renderMedia(): Promise<HTMLElement> {
  const info = await media.discover().catch(() => null);
  const devices = info?.devices ?? [];

  const cls = (c: string, label: string) => {
    const list = devices.filter((d) => d.class === c);
    const rows = list.map((d) =>
      el("div", { class: "flex mb" },
        el("span", { class: "tag" }, d.vendor),
        el("span", {}, d.name),
        el("span", { class: "muted mono" }, `${d.ip ?? "?"}:${d.port ?? "?"} · ${d.service}`),
        el("span", { class: "spacer" }),
        d.class === "stb" || d.class === "cast"
          ? el("button", {
              class: "btn ghost",
              onclick: () => {
                const app = prompt("DIAL 启动应用名(如 YouTube):", "YouTube");
                if (app && d.ip && d.port) media.dial({ ip: d.ip, port: d.port, app });
              },
            }, "投屏")
          : null,
      ),
    );
    return card(label, rows.length ? el("div", {}, ...rows) : empty("未发现,请确认设备与网关同网段且已上电"));
  };

  return el("div", {},
    el("h1", { class: "page-title" }, "媒体设备"),
    el("p", { class: "page-sub" }, `音响 ${info?.audio_count ?? 0} · 投屏 ${info?.cast_count ?? 0} · 机顶盒 ${info?.stb_count ?? 0}(mDNS 自动发现)`),
    el("div", { class: "mt" }, cls("audio", "音响(小爱 / 剑桥 / Sonos 等)")),
    el("div", { class: "mt" }, cls("cast", "投屏设备(AirPlay / Chromecast / DIAL)")),
    el("div", { class: "mt" }, cls("stb", "机顶盒 / 电视(DLNA / DIAL)")),
  );
}