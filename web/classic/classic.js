// 原版排版器的 wasm 移植：先裝好 wasm 與 engine.js（取代原版的 ldg.js、dlNote.js）、shell.js（取代 main.js）、editor.js（取代 ts.js），
// 再按原來的順序載入其餘腳本，最後載入內嵌的工具頁（它們一載入就會調用主頁的函數）。
import init, * as wasm from "./pkg/kanben_classic_wasm.js";
import { install } from "./engine.js";

// 嵌入模式（?embed）：隱去原版的編輯區，只排印父頁面發來的文本。
const embed = new URLSearchParams(location.search).has("embed");
if (embed) {
  document.documentElement.classList.add("kb-embed");
  const css = document.createElement("style");
  css.textContent =
    ".kb-embed #編輯函, .kb-embed #介欄, .kb-embed #劉海 { display: none !important; }" +
    ".kb-embed #結果函 { width: 100% !important; height: 100% !important; }";
  document.head.appendChild(css);
}

await init();
window.__kanbenWasm = wasm;
install();

function load(src) {
  return new Promise((resolve, reject) => {
    const s = document.createElement("script");
    s.src = src;
    s.async = false;
    s.onload = resolve;
    s.onerror = reject;
    document.body.appendChild(s);
  });
}

await load(new URL("shell.js", import.meta.url).href);
await load(new URL("editor.js", import.meta.url).href);
for (const f of document.querySelectorAll("iframe[data-src]")) f.src = new URL(f.dataset.src, new URL("./", import.meta.url)).href;
// main.js 在 window.onload 裏綁定事件；若頁面早已載入完，補調一次。
if (document.readyState === "complete") window.onload?.();

if (embed) {
  window.addEventListener("message", (e) => {
    if (e.source !== window.parent || e.data?.type !== "kanben-classic") return;
    const ed = document.getElementById("編輯器");
    ed.value = e.data.text;
    ed.dispatchEvent(new Event("input"));
    document.getElementById("bookName").textContent = e.data.book || "自排";
    if (document.getElementById("冊䈎")) window.切䈎(-1);
    window.單卷.排版();
    // 冊頁：書名、卷名、堂號都用源碼裏的（沒寫就空著，不用原版的預設）。
    if (e.data.leaves > 0) {
      document.getElementById("bookName").textContent = e.data.book || "";
      window.切䈎(e.data.leaves, "inMain", { 卷名: e.data.volume || "", 版權: e.data.studio || "" });
    }
    window.parent.postMessage({ type: "kanben-classic-done" }, "*");
  });
  window.parent.postMessage({ type: "kanben-classic-ready" }, "*");
}
