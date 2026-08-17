// 入口:hash 路由 + 顶栏时钟。

import "./style.css";
import { empty } from "./components/ui";
import { renderDashboard } from "./pages/dashboard";
import { renderChat } from "./pages/chat";
import { renderDevices } from "./pages/devices";
import { renderEvents } from "./pages/events";
import { renderMedia } from "./pages/media";
import { renderSettings } from "./pages/settings";
import { renderTools } from "./pages/tools";
import { renderAI } from "./pages/ai";

const app = document.querySelector<HTMLElement>("#app")!;

type Page = "dashboard" | "devices" | "events" | "settings" | "ai" | "chat" | "media" | "tools";

function route(): Page {
  const hash = location.hash.replace(/^#\//, "") || "dashboard";
  return (["dashboard", "devices", "events", "settings", "ai"].includes(hash) ? hash : "dashboard") as Page;
}

async function render(): Promise<void> {
  const page = route();
  // 顶栏高亮。
  document.querySelectorAll<HTMLElement>("[data-route]").forEach((a) => {
    a.classList.toggle("active", a.dataset.route === page);
  });
  app.innerHTML = "";
  try {
    switch (page) {
      case "dashboard":
        app.append(await renderDashboard());
        break;
      case "chat":
        app.append(await renderChat());
        break;
      case "devices":
        app.append(await renderDevices());
        break;
      case "media":
        app.append(await renderMedia());
        break;
      case "tools":
        app.append(await renderTools());
        break;
      case "events":
        app.append(await renderEvents());
        break;
      case "settings":
        app.append(await renderSettings());
        break;
      case "ai":
        app.append(await renderAI());
        break;
    }
  } catch (err) {
    app.append(empty(`加载失败:${err instanceof Error ? err.message : String(err)}`));
  }
}

window.addEventListener("hashchange", render);
render();

// 顶栏时钟。
setInterval(() => {
  const clock = document.querySelector<HTMLElement>("#clock");
  if (clock) {
    const d = new Date();
    const p = (n: number) => String(n).padStart(2, "0");
    clock.textContent = `${d.getFullYear()}-${p(d.getMonth() + 1)}-${p(d.getDate())} ${p(d.getHours())}:${p(d.getMinutes())}:${p(d.getSeconds())}`;
  }
}, 1000);