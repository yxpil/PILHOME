// 智能助手:日常聊天(AI,自动上下文压缩)。

import { aiChat, ChatMsg } from "../api";
import { card, el } from "../components/ui";

export async function renderChat(): Promise<HTMLElement> {
  const history: ChatMsg[] = [];
  let busy = false;

  const messages = el("div", { class: "chat-log" }, el("div", { class: "chat-msg ai" }, "你好,我是家庭网关助手。可以问我设备状态、分析事件,或让我帮你编排自动化。"));

  const input = el("input", { class: "input", placeholder: "输入消息,回车发送(历史超过 20 条自动压缩上下文)…", style: "flex:1" }) as HTMLInputElement;
  const sendBtn = el("button", { class: "btn" }, "发送");

  async function send() {
    const text = input.value.trim();
    if (!text || busy) return;
    busy = true;
    input.value = "";
    messages.append(el("div", { class: "chat-msg user" }, text));
    const res = await aiChat.chat(text, history).catch(() => null);
    history.push({ role: "user", content: text });
    if (res?.ok) {
      messages.append(el("div", { class: "chat-msg ai" }, res.reply, res.compressed ? el("span", { class: "tag active" }, "已压缩上下文") : null));
      history.push({ role: "assistant", content: res.reply });
    } else {
      const err = el("div", { class: "chat-msg ai warn" }, `请求失败:${res?.error ?? "AI 未配置"}`);
      messages.append(err);
    }
    busy = false;
    messages.scrollTop = messages.scrollHeight;
  }

  input.addEventListener("keydown", (e) => { if (e.key === "Enter") send(); });
  sendBtn.addEventListener("click", send);

  const quick = (label: string, msg: string) => el("button", { class: "btn ghost", onclick: () => { input.value = msg; send(); } }, label);

  return el("div", {},
    el("h1", { class: "page-title" }, "智能助手"),
    el("p", { class: "page-sub" }, "日常聊天 · 事件研判 · 上下文自动压缩"),
    card("对话",
      messages,
      el("div", { class: "flex mt" }, input, sendBtn),
      el("div", { class: "flex mt" },
        quick("设备状态", "帮我看看现在家里有多少设备在线?"),
        quick("分析告警", "请分析最近的安全事件并给出建议"),
        quick("自我审查", "请审查一下系统运行情况"),
      ),
    ),
  );
}