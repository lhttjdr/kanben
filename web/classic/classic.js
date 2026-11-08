// 原版排版器的 wasm 移植：先裝好 wasm 與 engine.js（取代原版的 ldg.js、dlNote.js）、shell.js（取代 main.js），
// 再按原來的順序載入其餘腳本，最後載入內嵌的工具頁（它們一載入就會調用主頁的函數）。
import init, * as wasm from "./pkg/kanben_classic_wasm.js";
import { install } from "./engine.js";

await init();
window.__kanbenWasm = wasm;
install();

const REF = new URL("../ref/", import.meta.url);
function load(src) {
  return new Promise((resolve, reject) => {
    const s = document.createElement("script");
    s.src = new URL(src, REF).href;
    s.async = false;
    s.onload = resolve;
    s.onerror = reject;
    document.body.appendChild(s);
  });
}

await load(new URL("shell.js", import.meta.url).href);
await load("type/set/ts.js");
for (const f of document.querySelectorAll("iframe[data-src]")) f.src = new URL(f.dataset.src, REF).href;
// main.js 在 window.onload 裏綁定事件；若頁面早已載入完，補調一次。
if (document.readyState === "complete") window.onload?.();
