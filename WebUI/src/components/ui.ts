// 轻量 DOM 构建工具:el() 与常用组件。

export type Child = Node | string | null | undefined;

/** 创建元素:el("div", { class: "x", onclick: fn }, child, child...) */
export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attrs: Record<string, unknown> = {},
  ...children: Child[]
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  for (const [key, value] of Object.entries(attrs)) {
    if (value === undefined || value === null) continue;
    if (key === "class") node.className = String(value);
    else if (key.startsWith("on") && typeof value === "function") {
      node.addEventListener(key.slice(2), value as EventListener);
    } else if (key === "dataset") {
      Object.assign(node.dataset, value as Record<string, string>);
    } else {
      node.setAttribute(key, String(value));
    }
  }
  for (const child of children) {
    if (child === null || child === undefined) continue;
    node.append(child);
  }
  return node;
}

/** 圆角卡片。 */
export function card(title: string, ...children: Child[]): HTMLElement {
  return el("section", { class: "card" }, el("h3", {}, title), ...children);
}

/** 状态圆点。 */
export function dot(kind: "online" | "warn" | "alert" | "idle"): HTMLElement {
  const cls = kind === "online" ? "online" : kind === "warn" ? "warn" : kind === "alert" ? "alert" : "";
  return el("span", { class: `dot ${cls}` });
}

/** 状态点 + 文本。 */
export function statusPill(ok: boolean, text: string): HTMLElement {
  return el("span", { class: "flex" }, dot(ok ? "online" : "warn"), el("span", {}, text));
}

/** 空白占位。 */
export function empty(text: string): HTMLElement {
  return el("div", { class: "empty" }, text);
}