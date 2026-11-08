// 在指定頁面逐個排印語料，保存排版後的 HTML 與截圖，供逐像素比對。
// 用法：node tools/classic/snapshot.mjs <調試端口> <頁面網址> <語料目錄> <輸出目錄>
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
let id = 0; const waiting = new Map(); const errors = [];
ws.addEventListener("message", (m) => {
  const d = JSON.parse(m.data);
  if (waiting.has(d.id)) { waiting.get(d.id)(d); waiting.delete(d.id); }
  else if (d.method === "Runtime.exceptionThrown") errors.push(d.params.exceptionDetails.exception?.description?.slice(0, 300));
});
const send = (method, params = {}) => new Promise((r) => { const i = ++id; waiting.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const ev = async (expr) => {
  const r = await send("Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true });
  if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 500));
  return r.result.result.value;
};
await send("Runtime.enable");
await send("Emulation.setDeviceMetricsOverride", { width: 1600, height: 1000, deviceScaleFactor: 1, mobile: false });
await send("Page.navigate", { url });
await sleep(6000);
mkdirSync(outDir, { recursive: true });
const ported = await ev(`!!window.__kanbenClassic`);
console.log(url, "wasm 移植：", ported);
for (const f of readdirSync(corpusDir).filter((f) => f.endsWith(".txt")).sort()) {
  const input = readFileSync(join(corpusDir, f), "utf8");
  const dom = await ev(`(async () => {
    const ed = document.getElementById("編輯器");
    ed.value = ${JSON.stringify(input)};
    單卷.排版();
    await new Promise((r) => setTimeout(r, 400));
    document.getElementById("結果").scrollLeft = 0;
    return { main: document.getElementById("inMain").innerHTML, tentou: document.getElementById("tentou").innerHTML,
             silan: document.getElementById("jiusilan")?.innerHTML ?? "" };
  })()`);
  await sleep(200);
  const shot = await send("Page.captureScreenshot", { format: "png" });
  const name = basename(f, ".txt");
  writeFileSync(join(outDir, name + ".json"), JSON.stringify(dom));
  writeFileSync(join(outDir, name + ".png"), Buffer.from(shot.result.data, "base64"));
}
if (errors.length) console.log("頁面錯誤：", errors.slice(0, 5));
ws.close(); process.exit(0);
