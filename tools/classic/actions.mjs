// 比對排版以外的動作：切成冊頁、改窗口後重排。
// 用法：node tools/classic/actions.mjs <端口> <頁面網址> <語料文件> <輸出目錄>
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
const size = (h) => send("Emulation.setDeviceMetricsOverride", { width: 1600, height: h, deviceScaleFactor: 1, mobile: false });
const shot = async (name) => writeFileSync(join(outDir, name + ".png"), Buffer.from((await send("Page.captureScreenshot", { format: "png" })).result.data, "base64"));
mkdirSync(outDir, { recursive: true });
await send("Runtime.enable");
await size(1000);
await send("Page.navigate", { url }); await sleep(6000);
const input = readFileSync(file, "utf8");
await ev(`(async () => { const ed = document.getElementById("編輯器"); ed.value = ${JSON.stringify(input)}; 單卷.排版(); await new Promise(r => setTimeout(r, 400)); return 1; })()`);
// 一：改窗口高度後重排
await size(760); await sleep(500);
await ev(`(async () => { 重排(); await new Promise(r => setTimeout(r, 400)); document.getElementById("結果").scrollLeft = 0; return 1; })()`);
const relayout = await ev(`({ main: document.getElementById("inMain").innerHTML, tentou: document.getElementById("tentou").innerHTML, silan: document.getElementById("jiusilan").innerHTML })`);
writeFileSync(join(outDir, "relayout.json"), JSON.stringify(relayout)); await shot("relayout");
// 二：切成冊頁
await size(1000); await sleep(300);
await ev(`(async () => { 重排(); 切䈎(10, "inMain"); await new Promise(r => setTimeout(r, 600)); return 1; })()`);
const pages = await ev(`({ main: document.getElementById("冊䈎").innerHTML, tentou: "", silan: "" })`);
writeFileSync(join(outDir, "pages.json"), JSON.stringify(pages)); await shot("pages");
// 三：收回長卷
await ev(`(async () => { 卷䈎互轉(); await new Promise(r => setTimeout(r, 900)); document.getElementById("結果").scrollLeft = 0; return 1; })()`);
const back = await ev(`({ main: document.getElementById("inMain").innerHTML, tentou: document.getElementById("tentou").innerHTML, silan: document.getElementById("jiusilan").innerHTML })`);
writeFileSync(join(outDir, "back.json"), JSON.stringify(back)); await shot("back");
if (errs.length) console.log("頁面錯誤：", errs);
ws.close(); process.exit(0);
