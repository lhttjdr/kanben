// 同一段刊本源碼：原版排法（web/classic）排出的各字位置 vs SVG 字格排版的各字格位。
// 用法：node tools/parity/collect.mjs <調試端口> <站點根網址> <語料目錄> <輸出目錄>
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
await send("Emulation.setDeviceMetricsOverride", { width: 1600, height: 1000, deviceScaleFactor: 1, mobile: false });
await send("Page.navigate", { url: new URL("classic/type/set/", site).href });
await sleep(7000);
await ev(`(async () => { window.__k = await import(${JSON.stringify(new URL("pkg/kanben_wasm.js", site).href)}); await __k.default(); return 1; })()`);
mkdirSync(outDir, { recursive: true });

for (const f of readdirSync(corpusDir).filter((f) => f.endsWith(".kb")).sort()) {
  const src = readFileSync(join(corpusDir, f), "utf8");
  const out = await ev(`(async () => {
    const src = ${JSON.stringify(src)};
    const c = __k.toClassic(src);
    const ed = document.getElementById("編輯器");
    ed.value = c.text; ed.dispatchEvent(new Event("input"));
    document.getElementById("bookName").textContent = c.book || "自排";
    單卷.排版();
    await new Promise((r) => setTimeout(r, 300));
    const fs = 正文字號;
    const cols = [...document.querySelectorAll("#jiusilan .jienghangsen")].map((c) => c.getBoundingClientRect());
    const top = cols[0].top;
    const kind = (el) => {
      if (el.closest(".jmhgbd, .gdzhbd, .hgbdSysQt1, .hgbdSysQt2, .zzbdQt1, .zzbdQt2, .hgbd2Qt1, .hgbd2Qt2, .zzbd2Qt1, .zzbd2Qt2")) return "Punct";
      if (el.closest(".rubyStc, .rightLine, .leftLine")) return "Side";
      if (el.closest(".grebdriuo, .dhgd, .rightLCmt, .leftLCmt, .twoLCmt")) return "Note";
      if (el.closest("#tentou")) return "Head";
      return "Main";
    };
    const chars = [];
    for (const rootId of ["inMain", "tentou"]) {
      const w = document.createTreeWalker(document.getElementById(rootId), NodeFilter.SHOW_TEXT);
      const range = document.createRange(); let n;
      while ((n = w.nextNode())) {
        const el = n.parentElement;
        if (el.closest(".hideCopy") || getComputedStyle(el).visibility === "hidden") continue;
        for (let i = 0; i < n.length; i++) {
          const ch = n.data[i]; if (!ch.trim()) continue;
          range.setStart(n, i); range.setEnd(n, i + 1);
          const r = range.getBoundingClientRect(); if (!r.width && !r.height) continue;
          const cx = r.left + r.width / 2, cy = r.top + r.height / 2;
          let col = cols.findIndex((c) => cx >= c.left - 0.5 && cx < c.right + 0.5);
          const c0 = cols[0];
          const colW = c0.width;
          if (col < 0) col = Math.round((c0.left + colW / 2 - cx) / colW);
          const cc = c0.left + colW / 2 - col * colW;
          chars.push({ ch, col, x: (cx - cc) / fs, y: (cy - top) / fs, size: parseFloat(getComputedStyle(el).fontSize) / fs, role: kind(el) });
        }
      }
    }
    const ours = JSON.parse(__k.glyphsJson(src, "cells=" + 每行格數 + "\\nformat=scroll\\nengine=svg"));
    const diags = JSON.parse(c.diagnostics).map((d) => d.message);
    c.free();
    return { cells: 每行格數, classic: chars, svg: ours, notes: diags };
  })()`);
  writeFileSync(join(outDir, basename(f, ".kb") + ".json"), JSON.stringify(out));
  console.log(f, "原版", out.classic.length, "字；SVG", out.svg.length, "字；每行", out.cells, "格");
}
ws.close(); process.exit(0);
