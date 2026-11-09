// 整版比像素：版框、界行、行號連同正文一起截圖，以外框右上角對齊。
// 用法：node tools/parity/frame.mjs <調試端口> <站點根網址> <語料目錄> <輸出目錄>
import { readFileSync, readdirSync, writeFileSync, mkdirSync } from "node:fs";
import { join, basename } from "node:path";

const [port, site, corpusDir, outDir] = process.argv.slice(2);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let targets;
for (let i = 0; i < 50; i++) { try { targets = await (await fetch(`http://127.0.0.1:${port}/json`)).json(); if (targets.length) break; } catch {} await sleep(200); }
const ws = new WebSocket(targets.find((t) => t.type === "page").webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener("open", r));
let id = 0; const waiting = new Map();
ws.addEventListener("message", (m) => { const d = JSON.parse(m.data); if (waiting.has(d.id)) { waiting.get(d.id)(d); waiting.delete(d.id); } });
const send = (method, params = {}) => new Promise((r) => { const i = ++id; waiting.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const ev = async (expr) => {
  const r = await send("Runtime.evaluate", { expression: expr, awaitPromise: true, returnByValue: true });
  if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 600));
  return r.result.result.value;
};
const shot = async (clip) => Buffer.from((await send("Page.captureScreenshot", { format: "png", clip: { ...clip, scale: 1 } })).result.data, "base64");
await send("Emulation.setDeviceMetricsOverride", { width: 1600, height: 1000, deviceScaleFactor: 1, mobile: false });
await send("Page.navigate", { url: new URL("classic/type/set/?embed", site).href });
await sleep(7000);
await ev(`(async () => {
  window.__k = await import(${JSON.stringify(new URL("pkg/kanben_wasm.js", site).href)}); await __k.default();
  const st = document.createElement("style"); st.textContent = __k.style(); document.head.appendChild(st);
  const box = document.createElement("div"); box.id = "svgbox";
  box.style.cssText = "position:fixed;left:0;top:0;z-index:9999;background:#fff;writing-mode:horizontal-tb";
  document.body.appendChild(box);
  return 1; })()`);
mkdirSync(outDir, { recursive: true });
const results = {};
for (const f of readdirSync(corpusDir).filter((f) => f.endsWith(".kb")).sort()) {
  const src = readFileSync(join(corpusDir, f), "utf8");
  const name = basename(f, ".kb");
  const g1 = await ev(`(async () => {
    document.getElementById("svgbox").style.display = "none";
    const c = __k.toClassic(${JSON.stringify(src)});
    const ed = document.getElementById("編輯器"); ed.value = c.text; ed.dispatchEvent(new Event("input"));
    單卷.排版(); c.free();
    document.getElementById("結果").scrollLeft = 0;
    await document.fonts.ready; await new Promise((r) => setTimeout(r, 300));
    const r = document.getElementById("gaeg").getBoundingClientRect();
    return { box: [r.left, r.top, r.width, r.height], cells: 每行格數 };
  })()`);
  const [l, t, w, h] = g1.box;
  // 截到外框以下 45px（行號），寬取到不超過 1600。
  const W = Math.min(w, 1500), H = h + 45;
  const a = await shot({ x: l + w - W, y: t, width: W, height: H });
  const g2 = await ev(`(async () => {
    const box = document.getElementById("svgbox"); box.style.display = "block";
    const r = __k.typeset(${JSON.stringify(src)}, "cells=${g1.cells}\\nformat=scroll\\nengine=svg\\nfont=original");
    box.innerHTML = r.page(0); r.free();
    const svg = box.querySelector("svg");
    const vb = svg.viewBox.baseVal; svg.setAttribute("width", vb.width * 0.3); svg.setAttribute("height", vb.height * 0.3);
    await document.fonts.ready; await new Promise((r) => setTimeout(r, 300));
    const top = svg.querySelector("rect.rule-fill").getBoundingClientRect();
    return { right: top.right, top: top.top };
  })()`);
  const b = await shot({ x: g2.right - W, y: g2.top, width: W, height: H });
  writeFileSync(join(outDir, `${name}-classic.png`), a);
  writeFileSync(join(outDir, `${name}-svg.png`), b);
  results[name] = [W, H];
}
writeFileSync(join(outDir, "_index.json"), JSON.stringify(results));
ws.close(); process.exit(0);
