// 刊本编辑器：排版全部由 wasm 完成，这里只负责界面。
import init, { typeset, html, style, example, fitCells, plainText } from "./pkg/kanben_wasm.js";

// 语法改过一次，旧草稿用不了，换个键。
const DRAFT_KEY = "kanben:draft:v2";
// 「字數隨窗口」时，一个字大约多少像素高。
const CELL_PX = 30;
// .pages 的上下内边距之和。
const PAGE_PADDING = 48;
const $ = (id) => document.getElementById(id);
const source = $("source");
const pages = $("pages");
const preview = $("preview");
const diagList = $("diags");
const status = $("status");
// 界面选项 → \setup 的键。
const controls = {
  format: $("opt-format"),
  punct: $("opt-punct"),
  "title-mark": $("opt-title"),
  font: $("opt-font"),
  columns: $("opt-columns"),
  cells: $("opt-cells"),
};
const fit = $("opt-fit");

function overrides() {
  return Object.entries(controls)
    .filter(([, el]) => el.value.trim() !== "")
    .map(([k, el]) => `${k}=${el.value.trim()}`)
    .join("\n");
}

/// 页面高度跟着预览区走；字數隨窗口时由 wasm 按这个高度算字數。
function pageHeight() {
  return Math.max(240, preview.clientHeight - PAGE_PADDING);
}

function applyFit() {
  const h = pageHeight();
  document.documentElement.style.setProperty("--page-h", `${h}px`);
  if (!fit.checked) return false;
  const others = overrides().split("\n").filter((l) => !l.startsWith("cells=")).join("\n");
  const cells = String(fitCells(source.value, others, h, CELL_PX));
  if (controls.cells.value === cells) return false;
  controls.cells.value = cells;
  return true;
}

function loadDraft() {
  try {
    return localStorage.getItem(DRAFT_KEY);
  } catch {
    return null;
  }
}

function saveDraft(text) {
  try {
    localStorage.setItem(DRAFT_KEY, text);
  } catch {
    // 无痕模式等情况下存不了，忽略。
  }
}

let first = true;

function render() {
  applyFit();
  const t0 = performance.now();
  const result = typeset(source.value, overrides());
  const ms = performance.now() - t0;

  // 保持离右边（书的开头）的距离不变。
  const fromRight = preview.scrollWidth - preview.clientWidth - preview.scrollLeft;
  const parts = [];
  for (let i = 0; i < result.pageCount; i++) parts.push(result.page(i));
  pages.innerHTML = parts.join("");
  preview.scrollLeft = first ? preview.scrollWidth : preview.scrollWidth - preview.clientWidth - fromRight;
  first = false;

  const diags = JSON.parse(result.diagnostics);
  showDiagnostics(diags);
  const errors = diags.filter((d) => d.severity === "error").length;
  const warnings = diags.length - errors;
  status.textContent =
    `${result.pageCount} 頁 · 排版 ${ms.toFixed(1)} ms` +
    (errors ? ` · ${errors} 處錯誤` : "") +
    (warnings ? ` · ${warnings} 處提示` : "");
  document.title = result.title ? `${result.title} · 刊本` : "刊本排版";
  result.free();
}

function showDiagnostics(diags) {
  diagList.replaceChildren(
    ...diags.map((d) => {
      const li = document.createElement("li");
      li.className = d.severity;
      const pos = document.createElement("span");
      pos.className = "pos";
      pos.textContent = d.line > 0 && d.from + d.to > 0 ? `${d.line}:${d.col}` : "設定";
      li.append(pos, d.message);
      li.addEventListener("click", () => {
        source.focus();
        source.setSelectionRange(d.from, Math.max(d.to, d.from + 1));
      });
      return li;
    }),
  );
}

let timer = 0;
function schedule() {
  clearTimeout(timer);
  timer = setTimeout(() => {
    saveDraft(source.value);
    render();
  }, 120);
}

function download(name, text, type) {
  const url = URL.createObjectURL(new Blob([text], { type }));
  const a = Object.assign(document.createElement("a"), { href: url, download: name });
  a.click();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}

function baseName() {
  const m = source.value.match(/^\s*\\book\{([^}]*)\}/m);
  return (m ? m[1].trim() || "刊本" : "刊本").replace(/[\\/:*?"<>|]/g, "");
}

await init();
$("kb-style").textContent = style();
source.value = loadDraft() ?? example();

source.addEventListener("input", schedule);
for (const el of Object.values(controls)) el.addEventListener("change", render);
fit.addEventListener("change", () => {
  controls.cells.disabled = fit.checked;
  if (!fit.checked) controls.cells.value = "";
  render();
});

// 窗口或分栏大小变化：页面只缩放；开了字數隨窗口且字數变了才重排。
let resizeTimer = 0;
new ResizeObserver(() => {
  clearTimeout(resizeTimer);
  resizeTimer = setTimeout(() => {
    if (applyFit()) render();
  }, 100);
}).observe(preview);

// 从版面复制文字：按阅读顺序，直排字形还原成普通标点。
document.addEventListener("copy", (e) => {
  const sel = getSelection();
  if (!sel || sel.isCollapsed || !preview.contains(sel.anchorNode)) return;
  e.clipboardData.setData("text/plain", plainText(sel.toString()));
  e.preventDefault();
});
$("btn-example").addEventListener("click", () => {
  if (source.value.trim() && source.value !== example() && !confirm("用示例替換當前文本？")) return;
  source.value = example();
  first = true;
  saveDraft(source.value);
  render();
});
$("btn-save").addEventListener("click", () => download(`${baseName()}.kb`, source.value, "text/plain;charset=utf-8"));
$("btn-html").addEventListener("click", () =>
  download(`${baseName()}.html`, html(source.value, overrides(), new URL("fonts/fonts.css", location.href).href), "text/html;charset=utf-8"),
);
$("btn-print").addEventListener("click", () => window.print());
source.addEventListener("keydown", (e) => {
  if (e.key === "Tab") {
    e.preventDefault();
    source.setRangeText("　", source.selectionStart, source.selectionEnd, "end");
    schedule();
  }
});

render();
