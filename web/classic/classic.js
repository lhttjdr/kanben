// 原版排版器的 wasm 移植：把頁面上原版的文本處理函數換成 wasm 裏的同名實現。
// 量頁面的幾步（度行、雙行夾注、增刪絲欄、對齊眉批 …）仍用原版的腳本。
import init, * as wasm from "./pkg/kanben_classic_wasm.js";

await init();

/** 原版在頁面上量出的環境值，每次調用時現取（字體加載、改窗口後會變）。 */
function env() {
  const 距 = window.調距 || {};
  return [
    ...["正文", "夾注", "注中注", "注中小字", "敘", "跋"].map((k) => `${k}=${距[k]}`),
    `行尾空白長=${window.行尾空白長}`,
    `系統=${window.系統}`,
  ].join(";");
}

let 抽 = null;

window.寫眉批 = function (cvTXT, ttID) {
  const [body, tentou] = wasm.headNotes(cvTXT).split("\0");
  document.getElementById(ttID).innerHTML = tentou;
  return body;
};

window.toHTML = (cvTXT) => wasm.toHTML(cvTXT, env());

window.點行間 = function (elID) {
  const el = document.getElementById(elID);
  el.innerHTML = wasm.markBody(el.innerHTML);
};

window.行間標點 = function (elID, clr) {
  const el = document.getElementById(elID);
  el.innerHTML = wasm.markInter(el.innerHTML, clr ?? "");
};

window.點字下 = (txt) => wasm.markNote(txt, env());

window.構字 = (ids) => wasm.composeChar(ids);

window.punctuateVH = function (樣式類, tf, ctxt) {
  if (tf) {
    抽?.free();
    抽 = wasm.punctuate(樣式類, ctxt);
    return 抽.html;
  }
  for (const item of document.getElementsByClassName(樣式類)) {
    const xt = wasm.unverticalize(item.innerHTML);
    if (window.調距["夾注"] != -0.25) item.style.letterSpacing = String(window.調距["夾注"] + 0.5) + "em";
    item.innerHTML = xt;
  }
  if (!抽 || !抽.hasPending) return;
  const box = document.getElementById(ctxt);
  box.innerHTML = 抽.restore(box.innerHTML);
};

window.__kanbenClassic = true;
// 頁面加載時原版已用自己的腳本排過一次，換好後重排。
if (window.單卷) window.單卷.排版();
