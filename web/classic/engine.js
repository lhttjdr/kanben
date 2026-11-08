// 排版流程與量頁面的幾步，取代原版的 ldg.js 與 dlNote.js（移植自哂蟹齋排版器，經作者許可）。
// 文本處理、雙行夾注的算式在 wasm 裏；這裏只量頁面、把結果寫回頁面。
// 依賴原版 main.js 的全局量：正文字號、每行格數、行寬、調距、系統、數、contentBlk、adaptM 等。
// 其中 `數` 是 const，不在 window 上，要直接寫名字。
/* global 數 */
import * as wasm from "./pkg/kanben_classic_wasm.js";

const g = window;
const $ = (id) => document.getElementById(id);
const px = (el, prop) => parseFloat(getComputedStyle(el).getPropertyValue(prop));

/** 原版在頁面上量出的環境值，每次調用時現取。 */
function env() {
  const 距 = g.調距 || {};
  return [
    ...["正文", "夾注", "注中注", "注中小字", "敘", "跋"].map((k) => `${k}=${距[k]}`),
    `行尾空白長=${g.行尾空白長}`,
    `系統=${g.系統}`,
  ].join(";");
}

// ───────── 文本處理（wasm） ─────────

let 抽 = null;

function 寫眉批(cvTXT, ttID) {
  const [body, tentou] = wasm.headNotes(cvTXT).split("\0");
  $(ttID).innerHTML = tentou;
  return body;
}

const toHTML = (cvTXT) => wasm.toHTML(cvTXT, env());

function 點行間(elID) {
  const el = $(elID);
  el.innerHTML = wasm.markBody(el.innerHTML);
}

const 點字下 = (txt) => wasm.markNote(txt, env());

function punctuateVH(樣式類, tf, ctxt) {
  if (tf) {
    抽?.free();
    抽 = wasm.punctuate(樣式類, ctxt);
    return 抽.html;
  }
  for (const item of document.getElementsByClassName(樣式類)) {
    const xt = wasm.unverticalize(item.innerHTML);
    if (g.調距["夾注"] != -0.25) item.style.letterSpacing = String(g.調距["夾注"] + 0.5) + "em";
    item.innerHTML = xt;
  }
  if (!抽 || !抽.hasPending) return;
  const box = $(ctxt);
  box.innerHTML = 抽.restore(box.innerHTML);
}

// ───────── 量頁面的幾步 ─────────

/** 側行、側注裏的陰文墨圍再包一層，供樣式定位。 */
function 單行夾注() {
  for (const ct of document.querySelectorAll(".leftLine, .rightLine, .leftLCmt, .rightLCmt")) {
    for (const c of ct.getElementsByTagName("cite")) {
      c.innerHTML = c.classList.contains("mh6grug") ? `<span><span>${c.innerHTML}</span></span>` : `<span>${c.innerHTML}</span>`;
    }
  }
}

/** 段首縮進的格數：父元素（及外層）的上邊距、內距。 */
function 段首縮進(el, 層數) {
  let 壬 = 0;
  let p = el.parentNode;
  for (let i = 0; i < 層數; i++, p = p.parentNode) 壬 += px(p, "margin-top") + px(p, "padding-top");
  return 壬;
}

/** 雙行夾注：量出起始位置，由 wasm 算出排法，寫回頁面。 */
function 雙行夾注(domid) {
  const 甲 = domid === undefined ? document.getElementsByClassName("grebdriuo") : $(domid).getElementsByClassName("grebdriuo");
  for (let i = 0; i < 甲.length; i++) {
    const 注 = 甲[i];
    const 天 = 注.innerHTML;
    const 丙 = wasm.noteLength(天);
    if (丙 <= 2) {
      注.className = "grebdriuo ig_gd";
      注.innerHTML = 點字下(天);
      continue;
    }
    const 己 = document.createElement("span");
    己.className = "beforegd";
    注.parentNode.insertBefore(己, 注);
    const 丁 = Math.round(己.offsetTop / g.正文字號);
    const 壬 = Math.round(段首縮進(注, 1) / g.正文字號);
    const 法 = wasm.notePlan(丙, 丁, 壬, g.每行格數);
    if (法.kind === "sht") {
      注.className = "grebdriuo sht_gd";
      注.style.height = 法.height;
      注.innerHTML = 點字下(天);
    } else {
      注.className = "grebdriuo long_gd";
      注.style.width = 法.width;
      注.style.shapeOutside = 注.style.clipPath = 法.clip;
      const 尾 = 法.tailClip;
      注.innerHTML = `<span><span class="gdHeadflt" style="height:${法.headHeight}"></span><span class="gdTailflt" style="clip-path:${尾};shape-outside:${尾}"></span>${點字下(天)}</span>`;
      if (注.nextSibling !== null) {
        const 後 = document.createElement("span");
        後.className = "sprAftgd";
        後.style.height = 法.afterHeight;
        注.insertAdjacentElement("afterend", 後);
      }
    }
    注.dataset.jdgs = Math.ceil(丙 / 2);
    法.free();
  }
}

/** 改窗口後：雙行夾注按新的行長改成本行或跨行。 */
function 調整雙行夾注(domid = "miun") {
  for (const 前 of $(domid).getElementsByClassName("beforegd")) {
    const 注 = 前.nextElementSibling;
    const 佔 = Number(注.dataset.jdgs);
    const 丁 = Math.round(前.offsetTop / g.正文字號);
    const 壬 = Math.round(段首縮進(注, 1) / g.正文字號);
    const 法 = wasm.noteReplan(佔, 丁, 壬, g.每行格數, 注.classList.contains("long_gd"));
    if (!法) continue;
    if (法.kind === "sht") {
      注.classList.replace("long_gd", "sht_gd");
      const 字距 = 注.style.letterSpacing;
      注.style.width = 注.style.clipPath = 注.style.shapeOutside = "";
      注.style.height = 法.height;
      注.style.letterSpacing = 字距;
      注.firstChild.removeChild(注.getElementsByClassName("gdHeadflt")[0]);
      注.firstChild.removeChild(注.getElementsByClassName("gdTailflt")[0]);
      注.innerHTML = 注.innerHTML.replace(/^<span>|<\/span>$/g, "");
      if (注.nextElementSibling && 注.nextElementSibling.className == "sprAftgd") 注.nextElementSibling.remove();
    } else {
      if (注.classList.contains("sht_gd")) {
        注.classList.replace("sht_gd", "long_gd");
        注.insertAdjacentHTML("afterbegin", '<span class="gdTailflt"></span>');
        注.insertAdjacentHTML("afterbegin", '<span class="gdHeadflt"></span>');
        注.innerHTML = "<span>" + 注.innerHTML + "</span>";
        if (注.nextSibling) {
          const 後 = document.createElement("span");
          後.className = "sprAftgd";
          注.insertAdjacentElement("afterend", 後);
        }
      }
      const 字距 = 注.style.letterSpacing;
      注.style = "";
      注.style.width = 法.width;
      注.style.letterSpacing = 字距;
      注.style.shapeOutside = 注.style.clipPath = 法.clip;
      注.getElementsByClassName("gdHeadflt")[0].style.height = 法.headHeight;
      const 尾 = 注.getElementsByClassName("gdTailflt")[0];
      尾.style.clipPath = 尾.style.shapeOutside = 法.tailClip;
      if (注.nextElementSibling && 注.nextElementSibling.className == "sprAftgd") 注.nextElementSibling.style.height = 法.afterHeight;
    }
    法.free();
  }
}

/** 句段旁注：按所在位置把旁注移到行的上端。 */
function 寫旁句() {
  const 字號比例 = 0.5;
  const 容器 = document.getElementsByClassName("rubyStc");
  for (let i = 0; i < 容器.length; i++) {
    let 壬 = 段首縮進(容器[i], 2);
    const 廓器 = 容器[i].parentNode.parentNode.parentNode;
    if (!廓器.contains($("inMain"))) 壬 += px(廓器, "margin-top") + px(廓器, "padding-top");
    壬 = Math.round(壬 / g.正文字號);
    const 首行縮進 = 容器[i].offsetTop / g.正文字號 / 字號比例;
    const 襯器 = 容器[i].firstElementChild;
    襯器.style.top = String(-首行縮進 + 壬 / 字號比例) + "em";
    襯器.style.textIndent = String(首行縮進 - 壬 / 字號比例) + "em";
    襯器.style.height = String((g.每行格數 - 壬) / 字號比例) + "em";
  }
}

/** 眉批對齊到正文裏它的位置。 */
function 對齊眉批() {
  const 批文 = document.getElementsByClassName("miipei");
  let 批楎;
  try {
    批楎 = (g.當歬卷 < 0 ? $("miug") : $("inMain")).getElementsByClassName("rsvnote");
  } catch {
    批楎 = document.getElementsByClassName("rsvnote");
  }
  const blk = $("content_blk");
  const 右邊 = px(blk, "margin-right") + px(blk, "padding-right");
  for (let i = 0; i < 批楎.length; i++) {
    批楎[i].innerHTML = "";
    const p = 批楎[i].offsetParent;
    const 偏 = p == $("inMain") || p == $("miug") ? 0 : p.offsetLeft;
    批文[i].style.right = String(document.documentElement.scrollWidth - 批楎[i].offsetLeft - 偏 - 右邊 * 2 - g.正文字號 * 3) + "px";
  }
}

/** 尾注：鼠標移到注號上時顯示注文。 */
function shTailCmtHr(n, event) {
  let 框 = $("隨顯校注");
  if (n < 0) {
    if (框 !== null) 框.style.display = "none";
    return;
  }
  if (!event) return;
  const el = $("pCmtA" + String(n));
  if (el === null) return;
  if (框 === null) {
    框 = document.createElement("DL");
    框.id = "隨顯校注";
    document.body.appendChild(框);
  }
  const 改 = (tm) =>
    tm
      .replace(/<span[^<>]+?><\/span>/g, "")
      .replace(/(\s|")gdzhbd(?=\s|")/g, "$1jmhgbd")
      .replace(/(\s|")zzbd(Sm|Md|Tl)(?=\s|")/g, "$1hgbdSys$2")
      .replace(/(\s|")zzbdQt(?=\d\s|\d")/g, "$1hgbdSysQt");
  框.innerHTML = "";
  const 條 = el.childNodes;
  for (let t = 1; t < 條.length; t++) {
    if (條[t].tagName == "DT") 框.innerHTML += "<dt>" + 改(條[t].innerHTML) + "</dt>";
    else if (條[t].tagName == "DD") 框.innerHTML += "<dd>" + 改(條[t].innerHTML) + "</dd>";
  }
  if (框.innerHTML.length > 0) {
    框.style.display = "block";
    框.style.left = (event.clientX > 200 ? String(event.clientX - 100) : "0") + "px";
  }
}

// ───────── 冊頁 ─────────

/** 長卷切成冊頁：每半葉若干行，加版框、絲欄、版心。`半䈎行數 < 0` 時收回長卷。 */
function 切䈎(半䈎行數 = 10, 卷號 = "inMain", { 本書名 = undefined, 卷名 = undefined, 版權 = "哂蟹齋", 魚尾式 = 12 } = {}) {
  const contentBlk = g.contentBlk;
  contentBlk.style.display = "";
  let 冊䈎 = $("冊䈎");
  if (冊䈎 === null) {
    冊䈎 = document.createElement("div");
    冊䈎.id = "冊䈎";
    contentBlk.parentNode.insertBefore(冊䈎, contentBlk.nextSibling);
  } else {
    冊䈎.innerHTML = "";
  }
  if (半䈎行數 < 0) {
    冊䈎.remove();
    setTimeout(() => g.增刪絲欄(), 500);
    return;
  }
  const 諸空段 = document.getElementsByClassName("空行");
  for (const 段 of 諸空段) 段.innerHTML = 段.innerHTML.replace(/<br>/g, "　<br>");
  const 滾者 = location.pathname.endsWith("/type/set/") ? $("結果") : document.documentElement;
  const 卷長 = 滾者.scrollWidth;
  const 卷進度 = 滾者.scrollLeft / (卷長 - 滾者.clientWidth);
  滾者.scrollTo(卷長, 0);
  if (!本書名) 本書名 = $("bookName").textContent.replace(/\s+/g, "");
  if (本書名.length > 7) 本書名 = [本書名.slice(0, Math.ceil(本書名.length / 2)), 本書名.slice(Math.ceil(本書名.length / 2))];
  if (!卷名) 卷名 = $("curSec") ? $("curSec").textContent.replace(/\s+/g, " ") : "";
  const 總行數 = document.getElementsByClassName("jienghangsen").length - 1;
  const 總半䈎數 = Math.ceil(總行數 / 半䈎行數);
  const 總半䈎數補耦 = 總半䈎數 % 2 == 1 ? 總半䈎數 + 1 : 總半䈎數;
  const 卷元素 = $(卷號);
  const 卷類名 = 卷元素.className;
  const 文字上沿 = $("miun").getBoundingClientRect().top + g.正文字號 * 0.1;
  const 首段 = 卷元素.children[0];
  const 入版片段 = document.createRange();
  const 諸䈎片段 = new DocumentFragment();
  入版片段.setStartBefore(首段);
  入版片段.setEndAfter(首段);
  const 版框高 = $("gaeg").style.height;
  const 半䈎版寬 = 半䈎行數 * g.行寬 * g.正文字號;
  let x = 入版片段.getBoundingClientRect().right - 1;
  let 切處所屬;
  let 將刪至 = 首段;
  let 所刪之寬 = 0;
  for (let 敘 = 1; 敘 <= 總半䈎數補耦; 敘++) {
    const prevScrL = 滾者.scrollLeft;
    滾者.scrollBy(-半䈎版寬, 0);
    if (敘 == 總半䈎數) {
      切處所屬 = 卷元素.lastElementChild;
    } else if (敘 < 總半䈎數) {
      x += prevScrL - 半䈎版寬 - 滾者.scrollLeft;
      let textNode;
      if (document.caretPositionFromPoint) {
        textNode = document.caretPositionFromPoint(x, 文字上沿 + 2).offsetNode;
      } else if (document.caretRangeFromPoint) {
        textNode = document.caretRangeFromPoint(x, 文字上沿 + 2).startContainer;
      }
      切處所屬 = textNode.parentElement;
      while (切處所屬.parentElement.id != 卷號) 切處所屬 = 切處所屬.parentElement;
    }
    const 版內 = document.createElement("div");
    版內.className = "冊䈎版內";
    版內.style.height = "calc(" + 版框高 + " + 0.1em)";
    const 天頭 = document.createElement("div");
    天頭.className = "冊䈎天頭";
    const 版內文 = document.createElement("div");
    版內文.className = "版內文";
    if (總半䈎數 == 總半䈎數補耦 || 敘 < 總半䈎數補耦) {
      入版片段.setEndAfter(切處所屬);
      const 裡 = document.createElement("div");
      if (卷類名) 裡.className = 卷類名;
      裡.style.transform = `translateX(calc(${半䈎行數 * (敘 - 1) - 所刪之寬 / g.正文字號 / g.行寬}*var(--行寬)))`;
      裡.appendChild(入版片段.cloneContents());
      版內文.appendChild(裡);
      while (將刪至 != 切處所屬) {
        所刪之寬 += 將刪至.offsetWidth;
        將刪至 = 將刪至.nextElementSibling;
      }
    }
    版內.appendChild(版內文);
    const 絲欄片段 = new DocumentFragment();
    for (let 行 = 1; 行 <= 半䈎行數; 行++) {
      const 新行 = document.createElement("div");
      新行.className = "jienghangsen";
      絲欄片段.appendChild(新行);
    }
    const 半䈎絲欄 = document.createElement("div");
    半䈎絲欄.className = "冊䈎絲欄";
    const 半䈎版框 = document.createElement("div");
    半䈎版框.className = "版框";
    半䈎版框.style.height = 版框高;
    const 版心 = document.createElement("div");
    版心.className = "版心";
    版心.innerHTML =
      typeof 本書名 == "string"
        ? '<div style="flex-basis:' + String(本書名.length) + 'em">' + 本書名 + "</div>"
        : '<div class="版心長書名">' + 本書名[(敘 + 1) % 2] + "</div>";
    版心.innerHTML +=
      '<div><b class="魚尾"></b><span>' + 卷名 + "</span><span>" + 數["讀十萬以內"](Math.ceil(敘 / 2)) + '</span><b class="魚尾"></b></div><div>' + 版權 + "</div>";
    半䈎絲欄.appendChild(絲欄片段);
    敘 % 2 == 1 ? 半䈎絲欄.append(版心) : 半䈎絲欄.prepend(版心);
    半䈎版框.append(半䈎絲欄);
    版內.prepend(半䈎版框);
    const 半䈎 = document.createElement("div");
    半䈎.append(天頭, 版內);
    諸䈎片段.appendChild(半䈎);
    入版片段.setStartBefore(切處所屬);
  }
  冊䈎.appendChild(諸䈎片段);
  document.querySelector(":root").style.setProperty("--版內文寬", `calc(var(--行寬)*${半䈎行數})`);
  contentBlk.style.display = $("right_row").style.display = $("left_row").style.display = "none";
  滾者.scrollTo((滾者.scrollWidth - 滾者.clientWidth) * 卷進度, 0);
  for (const 段 of 諸空段) 段.innerHTML = 段.innerHTML.replace(/　/g, "");
}

/** 長卷、冊頁互換。 */
function 卷䈎互轉() {
  if (g.contentBlk.style.display == "none") {
    切䈎(-9);
    return;
  }
  const 目次卷 = $("miug");
  const 切誰 = 目次卷 && 目次卷.style.display != "none" ? "miug" : "inMain";
  if ($("outPDFpan") === null) {
    切䈎(10, 切誰);
  } else {
    $("outPDFpan").style.display = "flex";
    $("outPDFwd").contentWindow.切誰 = 切誰;
  }
}

// ───────── 排版流程 ─────────

/** 一卷：源文本 → 正文與天頭的 HTML → 量頁面調整。 */
class 哂蟹卷 {
  constructor({ 正文號 = "inMain", 天頭號 = "tentou", 手機頂列數 = 1, 標點色 = "標點色", 鏈題級 = 2, 源 } = {}) {
    this.正文號 = 正文號;
    this.天頭號 = 天頭號;
    this.手機頂列數 = 手機頂列數;
    this.標點色 = 標點色;
    this.源 = 源;
    g.鏈題元素 = "h" + 鏈題級.toString();
  }

  排版() {
    const 天頭凷 = $(this.天頭號);
    const 正文凷 = $(this.正文號);
    const 排文字 = (procT) => {
      procT = 寫眉批(procT, this.天頭號);
      procT = toHTML(procT);
      天頭凷.innerHTML = toHTML(天頭凷.innerHTML);
      正文凷.innerHTML = punctuateVH("grebdriuo", true, procT);
      點行間(this.正文號, this.標點色);
      點行間(this.天頭號);
      單行夾注("dhgd");
      g.度行();
      寫旁句();
      雙行夾注();
      punctuateVH("grebdriuo", false, this.正文號);
      if ($("infos") !== null) $("infos").style.display = "none";
      g.增刪絲欄();
      對齊眉批();
      window.addEventListener("resize", () => {
        對齊眉批();
        g.增刪絲欄();
      });
    };
    g.adaptM(this.手機頂列數);
    if (!this.源) {
      排文字(正文凷.innerHTML);
    } else if (typeof this.源 === "string") {
      fetch(this.源)
        .then((r) => (r.ok ? r.text() : Promise.reject(r.status)))
        .then(排文字);
    } else if (this.源.tagName == "TEXTAREA") {
      排文字(this.源.value);
    } else {
      排文字(this.源.innerHTML.replace(/<\/div>\s*<div>/g, "\n").replace(/<.+?>/g, "").replace(/^[ \t]+/gm, ""));
    }
  }
}

/** 裝上原版其餘腳本要用的全局名字。 */
export function install() {
  Object.assign(g, {
    哂蟹卷,
    切䈎,
    卷䈎互轉,
    寫旁句,
    對齊眉批,
    調整雙行夾注,
    雙行夾注,
    單行夾注,
    shTailCmtHr,
    寫眉批,
    toHTML,
    點行間,
    點字下,
    punctuateVH,
    __kanbenClassic: true,
  });
}
