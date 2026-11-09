// 逐列比像素：同一段刊本源碼，原版排法（web/classic）與 SVG 版在同一頁、同一比例下截圖，
// 各取每一行（列）的內部（避開界行），逐像素比較。
// 用法：node tools/parity/pixels.mjs <調試端口> <站點根網址> <語料目錄> <輸出目錄> [列數]
import { readFileSync, readdirSync, writeFileSync, mkdirSync } from "node:fs";
import { join, basename } from "node:path";

const [port, site, corpusDir, outDir, ncolArg] = process.argv.slice(2);
const NCOL = Number(ncolArg || 6);
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
await send("Page.navigate", { url: new URL("classic/type/set/", site).href });
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
  // 一：原版排法
  const g1 = await ev(`(async () => {
    document.getElementById("svgbox").style.display = "none";
    const c = __k.toClassic(${JSON.stringify(src)});
    const ed = document.getElementById("編輯器"); ed.value = c.text; ed.dispatchEvent(new Event("input"));
    單卷.排版(); c.free();
    document.getElementById("結果").scrollLeft = 0;
    await document.fonts.ready; await new Promise((r) => setTimeout(r, 300));
    const cols = [...document.querySelectorAll("#jiusilan .jienghangsen")].slice(0, ${NCOL}).map((c) => { const r = c.getBoundingClientRect(); return [r.left, r.top, r.width, r.height]; });
    return { cols, cells: 每行格數 };
  })()`);
  const classicShots = [];
  for (const [l, t, w, h] of g1.cols) classicShots.push(await shot({ x: l + 2, y: t, width: w - 4, height: h }));
  // 二：SVG 版，一字 30px
  const g2 = await ev(`(async () => {
    const box = document.getElementById("svgbox"); box.style.display = "block";
    const r = __k.typeset(${JSON.stringify(src)}, "cells=${g1.cells}\\nformat=scroll\\nengine=svg\\nfont=original");
    box.innerHTML = r.page(0); r.free();
    const svg = box.querySelector("svg");
    const vb = svg.viewBox.baseVal; svg.setAttribute("width", vb.width * 0.3); svg.setAttribute("height", vb.height * 0.3);
    await document.fonts.ready; await new Promise((r) => setTimeout(r, 300));
    const inner = svg.querySelector("rect.rule");
    const m = svg.getScreenCTM();
    const right = (+inner.getAttribute("x") + +inner.getAttribute("width")) * m.a + m.e, top = +inner.getAttribute("y") * m.d + m.f;
    return { right, top, colw: 160 * m.a };
  })()`);
  const svgShots = [];
  g1.cols.forEach(([, , w, h], i) => svgShots.push({ x: g2.right - (i + 1) * g2.colw + 2, y: g2.top, width: w - 4, height: h }));
  for (let i = 0; i < svgShots.length; i++) svgShots[i] = await shot(svgShots[i]);
  classicShots.forEach((b, i) => writeFileSync(join(outDir, `${name}-c${i}-classic.png`), b));
  svgShots.forEach((b, i) => writeFileSync(join(outDir, `${name}-c${i}-svg.png`), b));
  results[name] = g1.cols.length;
}
writeFileSync(join(outDir, "_index.json"), JSON.stringify(results));
ws.close(); process.exit(0);
