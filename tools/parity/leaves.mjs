// 冊頁比像素：原版「切䈎」切出的半葉與 SVG 冊頁逐頁截圖，以紙邊、外框上沿對齊。
// 用法：node tools/parity/leaves.mjs <調試端口> <站點根網址> <語料目錄> <輸出目錄> [每半葉行數] [頁數]
import { readFileSync, readdirSync, writeFileSync, mkdirSync } from "node:fs";
import { join, basename } from "node:path";

const [port, site, corpusDir, outDir, colsArg, pagesArg] = process.argv.slice(2);
const COLS = Number(colsArg || 9), PAGES = Number(pagesArg || 2);
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
    if (document.getElementById("冊䈎")) 切䈎(-1);
    const ed = document.getElementById("編輯器"); ed.value = c.text; ed.dispatchEvent(new Event("input"));
    document.getElementById("bookName").textContent = c.book;
    單卷.排版();
    await document.fonts.ready; await new Promise((r) => setTimeout(r, 300));
    切䈎(${COLS}, "inMain", { 卷名: c.volume, 版權: c.studio }); c.free();
    await document.fonts.ready; await new Promise((r) => setTimeout(r, 300));
    const panels = [...document.querySelectorAll("#冊䈎 > div")].slice(0, ${PAGES}).map((p) => {
      p.scrollIntoView({ block: "nearest", inline: "center" });
      const r = p.getBoundingClientRect(), f = p.querySelector(".版框").getBoundingClientRect();
      return [r.left, f.top, r.width, f.height];
    });
    return { panels, cells: 每行格數 };
  })()`);
  const shots = [];
  for (let i = 0; i < g1.panels.length; i++) {
    const box = await ev(`(() => { const p = document.querySelectorAll("#冊䈎 > div")[${i}]; p.scrollIntoView({ block: "nearest", inline: "center" });
      const r = p.getBoundingClientRect(), f = p.querySelector(".版框").getBoundingClientRect(); return [r.left, f.top, r.width, f.height]; })()`);
    await sleep(100);
    shots.push([box, await shot({ x: box[0], y: box[1] - 10, width: box[2], height: box[3] + 20 })]);
  }
  for (let i = 0; i < shots.length; i++) {
    const [[, , w, h], a] = shots[i];
    const g2 = await ev(`(async () => {
      const box = document.getElementById("svgbox"); box.style.display = "block";
      const r = __k.typeset(${JSON.stringify(src)}, "cells=${g1.cells}\\ncolumns=${COLS}\\nformat=leaves\\nengine=svg\\nfont=original");
      box.innerHTML = r.page(${i}); r.free();
      const svg = box.querySelector("svg");
      const vb = svg.viewBox.baseVal; svg.setAttribute("width", vb.width * 0.3); svg.setAttribute("height", vb.height * 0.3);
      await document.fonts.ready; await new Promise((r) => setTimeout(r, 300));
      const top = svg.querySelector("rect.rule-fill").getBoundingClientRect(), s = svg.getBoundingClientRect();
      return { left: s.left, top: top.top, width: s.width };
    })()`);
    const b = await shot({ x: g2.left, y: g2.top - 10, width: w, height: h + 20 });
    writeFileSync(join(outDir, `${name}-p${i}-classic.png`), a);
    writeFileSync(join(outDir, `${name}-p${i}-svg.png`), b);
    results[`${name}-p${i}`] = [w, g2.width];
  }

}
writeFileSync(join(outDir, "_index.json"), JSON.stringify(results));
ws.close(); process.exit(0);
