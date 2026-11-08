// 比對經由內嵌工具頁的操作：打開「切葉」面板並確定（冊頁）、右鍵選單的「重排」。
// 用法：node tools/classic/ui.mjs <端口> <頁面網址> <語料文件> <輸出目錄>
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { join } from "node:path";
const [port, url, file, outDir] = process.argv.slice(2);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
let t; for (let i = 0; i < 50; i++) { try { t = await (await fetch(`http://127.0.0.1:${port}/json`)).json(); if (t.length) break; } catch {} await sleep(200); }
const ws = new WebSocket(t.find((x) => x.type === "page").webSocketDebuggerUrl);
await new Promise((r) => ws.addEventListener("open", r));
let id = 0; const w = new Map(); const errs = [];
ws.addEventListener("message", (m) => { const d = JSON.parse(m.data); if (w.has(d.id)) { w.get(d.id)(d); w.delete(d.id); } else if (d.method === "Runtime.exceptionThrown") errs.push(d.params.exceptionDetails.exception?.description?.slice(0, 200)); });
const send = (method, params = {}) => new Promise((r) => { const i = ++id; w.set(i, r); ws.send(JSON.stringify({ id: i, method, params })); });
const ev = async (e) => { const r = await send("Runtime.evaluate", { expression: e, awaitPromise: true, returnByValue: true }); if (r.result.exceptionDetails) throw new Error(JSON.stringify(r.result.exceptionDetails).slice(0, 400)); return r.result.result.value; };
const shot = async (name) => writeFileSync(join(outDir, name + ".png"), Buffer.from((await send("Page.captureScreenshot", { format: "png" })).result.data, "base64"));
mkdirSync(outDir, { recursive: true });
await send("Runtime.enable");
await send("Emulation.setDeviceMetricsOverride", { width: 1600, height: 1000, deviceScaleFactor: 1, mobile: false });
await send("Page.navigate", { url }); await sleep(7000);
await ev(`(async () => { const ed = document.getElementById("編輯器"); ed.value = ${JSON.stringify(readFileSync(file, "utf8"))}; ed.dispatchEvent(new Event("input")); 單卷.排版(); await new Promise(r => setTimeout(r, 400)); return 1; })()`);
// 一：切葉面板 → 確定
await ev(`(async () => {
  document.getElementById("切䈎鈕").click();
  await new Promise(r => setTimeout(r, 300));
  const d = document.getElementById("outPDFwd").contentDocument;
  d.getElementById("每面行數").value = "9"; d.getElementById("版心頂欄字").value = "資治通鑑";
  d.getElementById("okbtn").click();
  await new Promise(r => setTimeout(r, 800));
  return 1; })()`);
writeFileSync(join(outDir, "panel-pages.json"), JSON.stringify({ main: document_safe(await ev(`document.getElementById("冊䈎")?.innerHTML ?? "無"`)), tentou: "", silan: "" }));
await shot("panel-pages");
// 二：右鍵選單裏的字號、重排（經由 iframe 調用主頁）
await ev(`(async () => {
  document.getElementById("切䈎鈕").click(); await new Promise(r => setTimeout(r, 900));
  const m = document.getElementById("ctxmenu").contentWindow;
  m.document.getElementById("重排").click(); await new Promise(r => setTimeout(r, 600));
  document.getElementById("結果").scrollLeft = 0; return 1; })()`);
writeFileSync(join(outDir, "menu-relayout.json"), JSON.stringify({ main: await ev(`document.getElementById("inMain").innerHTML`), tentou: await ev(`document.getElementById("tentou").innerHTML`), silan: await ev(`document.getElementById("jiusilan").innerHTML`) }));
await shot("menu-relayout");
if (errs.length) console.log("頁面錯誤：", errs);
ws.close(); process.exit(0);
function document_safe(x) { return x; }
