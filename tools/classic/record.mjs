// 在对照版（web/ref）里逐个排印语料，记录原版各函数的输入输出，生成移植用的标准答案。
// 用法：node tools/classic/record.mjs <调试端口> <对照版网址> <语料目录> <输出目录>
// 需先启动 Chromium：chrome --headless=new --remote-debugging-port=<端口>
import { readFileSync, readdirSync, writeFileSync, mkdirSync } from "node:fs";
import { join, basename } from "node:path";

const [port, url, corpusDir, outDir] = process.argv.slice(2);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let targets;
for (let i = 0; i < 50; i++) {
  try { targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json(); if (targets.length) break; } catch {}
  await sleep(200);
}
const ws = new WebSocket(targets.find((t) => t.type === "page").webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener("open", r));
let id = 0; const waiting = new Map();
ws.addEventListener("message", (m) => { const d = JSON.parse(m.data); if (waiting.has(d.id)) { waiting.get(d.id)(d); waiting.delete(d.id); } });
const send = (method, params = {}) => new Promise((r) => { const i = ++id; waiting.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const ev = async (expr) => {
  const r = await send("Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true });
  if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 500));
  return r.result.result.value;
};

const WIDTH = 1600, HEIGHT = 1000;
await send("Emulation.setDeviceMetricsOverride", { width: WIDTH, height: HEIGHT, deviceScaleFactor: 1, mobile: false });
await send("Page.navigate", { url });
await sleep(6000);

// 包住原版的全局函数：纯字符串函数记参数与返回值；改页面的函数记前后的 HTML。
await ev(`(() => {
  window.__rec = [];
  const snap = () => ({ main: document.getElementById("inMain")?.innerHTML ?? null, tentou: document.getElementById("tentou")?.innerHTML ?? null });
  const PURE = ["寫眉批","toHTML","數碼","畫表格","分段","分節","引詩文","分段單兒","插圖","尾注","特殊段","渲染段","兼行","旁字","提取旁句","字樣","點字下","陰文和墨圍","圈點抹","構字","縱中橫碼"];
  const DOM = ["點行間","行間標點","單行夾注","度行","寫旁句","雙行夾注","增刪絲欄","對齊眉批"];
  const plain = (a) => (typeof a === "string" || typeof a === "number" || typeof a === "boolean" || a === undefined || a === null) ? a : String(a);
  for (const n of PURE) {
    const f = window[n]; if (typeof f !== "function") continue;
    window[n] = function (...args) {
      const r = f.apply(this, args);
      __rec.push({ fn: n, args: args.map(plain), ret: plain(r), tentou: n === "寫眉批" ? document.getElementById("tentou").innerHTML : undefined });
      return r;
    };
  }
  for (const n of DOM) {
    const f = window[n]; if (typeof f !== "function") continue;
    window[n] = function (...args) {
      const before = snap(); const r = f.apply(this, args);
      __rec.push({ fn: n, args: args.map(plain), before, after: snap() });
      return r;
    };
  }
  const pv = window.punctuateVH;
  window.punctuateVH = function (cls, tf, ctxt) {
    const before = snap(); const r = pv.call(this, cls, tf, ctxt);
    __rec.push(tf ? { fn: "punctuateVH", args: [cls, tf, ctxt], ret: r, 補遺字: 補遺字, 拼字: 拼字 } : { fn: "punctuateVH", args: [cls, tf, ctxt], before, after: snap() });
    return r;
  };
  return 1;
})()`);

const globals = await ev(`({ 調距, 行尾空白長, 每行格數, 正文字號, 系統, 標點, 行寬, 跨註行寬, 瀏覽器 })`);
mkdirSync(outDir, { recursive: true });
writeFileSync(join(outDir, "_globals.json"), JSON.stringify({ viewport: [WIDTH, HEIGHT], ...globals }, null, 1));

for (const f of readdirSync(corpusDir).filter((f) => f.endsWith(".txt")).sort()) {
  const input = readFileSync(join(corpusDir, f), "utf8");
  const out = await ev(`(() => {
    __rec = [];
    const ed = document.getElementById("編輯器");
    ed.value = ${JSON.stringify(input)};
    單卷.排版();
    return { input: ed.value, after: { 調距, 行尾空白長, 每行格數, 正文字號 }, calls: __rec,
      final: { main: document.getElementById("inMain").innerHTML, tentou: document.getElementById("tentou").innerHTML } };
  })()`);
  writeFileSync(join(outDir, basename(f, ".txt") + ".json"), JSON.stringify(out, null, 1));
  console.log(f, out.calls.length, "calls");
}
ws.close(); process.exit(0);
