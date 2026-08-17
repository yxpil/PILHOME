// PILHOME 智能设备模拟器(WSL / 任意 Node 环境,零依赖)
// 模拟:洗衣机 / 电视盒子 / 饮水机 / 扫地机器人 / 小爱音响 / 机顶盒
// 通过 HTTP 上报到 PILHOME 网关,验证全链路(台账/事件/阈值告警/AI 研判/习惯学习)
//
// 用法: node tools/simulator.js [http://网关地址:8080]

const BASE = process.argv[2] || detectGateway();
const DEVICES = [
  { id: "washer", name: "洗衣机", kind: "actuator", keys: ["power", "cycle", "temperature", "door"] },
  { id: "tv_box", name: "电视盒子", kind: "actuator", keys: ["power", "volume", "playing"] },
  { id: "water_dispenser", name: "饮水机", kind: "sensor", keys: ["water_level", "temperature", "leak"] },
  { id: "vacuum", name: "扫地机器人", kind: "actuator", keys: ["battery", "status", "docked"] },
  { id: "xiaoai", name: "小爱音响", kind: "actuator", keys: ["playing", "volume", "wakeword"] },
  { id: "stb", name: "机顶盒", kind: "actuator", keys: ["power", "channel"] },
];

function detectGateway() {
  try {
    const conf = require("fs").readFileSync("/etc/resolv.conf", "utf8");
    const m = conf.match(/nameserver\s+([\d.]+)/);
    if (m) return `http://${m[1]}:8080`;
  } catch (_) {}
  return "http://127.0.0.1:8080";
}

async function post(path, body) {
  const res = await fetch(BASE + path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  return res.json();
}

function rand(min, max) { return min + Math.random() * (max - min); }
function pick(arr) { return arr[Math.floor(Math.random() * arr.length)]; }

async function register(dev) {
  await post("/api/devices", { id: dev.id, name: dev.name, kind: dev.kind, address: `sim://${dev.id}` });
}

async function report(dev) {
  const key = pick(dev.keys);
  let value, level = "info";
  switch (dev.id) {
    case "washer":
      value = pick(["idle", "washing", "spinning", "done"]);
      if (value === "done") level = "warn";
      break;
    case "tv_box":
      value = pick(["on", "off", "standby"]);
      break;
    case "water_dispenser":
      value = rand(20, 100).toFixed(1);
      if (parseFloat(value) < 25) level = "alert";
      break;
    case "vacuum":
      value = Math.floor(rand(10, 100)).toString();
      if (parseInt(value) < 20) level = "warn";
      break;
    case "xiaoai":
      value = pick(["idle", "playing", "paused"]);
      break;
    default:
      value = pick(["on", "off", "1", "2", "3", "4", "5"]);
  }
  const res = await post(`/api/devices/${dev.id}/event`, { key, value, level });
  console.log(`[${new Date().toISOString()}] ${dev.id} ${key}=${value} (${level}) 置信度=${res.habit_probability?.toFixed(2)} ${res.habit_verdict ?? ""}`);
}

(async () => {
  console.log(`PILHOME 设备模拟器启动,上报地址: ${BASE}`);
  for (const dev of DEVICES) {
    await register(dev);
    console.log(`  已注册: ${dev.id} (${dev.name})`);
  }
  for (const dev of DEVICES) {
    const tick = () => setTimeout(async () => { await report(dev); tick(); }, rand(5000, 12000));
    tick();
  }
  setInterval(async () => {
    try {
      const res = await fetch(BASE + "/api/status").then((r) => r.json());
      console.log(`[汇总] 设备 ${res.devices_total ?? "?"} 台 · 在线 ${res.online ?? "?"} · 事件 ${res.events_total ?? "?"}`);
    } catch (e) { console.log("[汇总] 网关不可达:", e.message); }
  }, 30000);
})();