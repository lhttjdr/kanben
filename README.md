# 刊本

古籍竖排排版引擎。排版全部在 Rust 里完成，编译成 WebAssembly 在浏览器里运行，输出带版框、界行、版心的 SVG 页面。

特点：

- **语法**：LaTeX 风格 `\命令[參數]{內容}`。标记只用 ASCII，正文标点保留原义，去掉标记即原文。
- **解析**：先按通用形式解析，再查命令表；出错带行列号。
- **排版**：按字格计算坐标，结果确定，不依赖浏览器测量，可单元测试。夾注、眉批、分葉都在 Rust 里算好位置。
- **输出**：SVG，每葉一页，文字仍可搜索、复制。
- **字體**：正文明朝（默认）或仿宋，批注楷書，都是开源字體（OFL），切片成网页字体按需加载；算籌、圈點等由引擎直接画出，不靠字体。

语法说明见 [docs/語法.md](docs/語法.md)，示例见 [examples/通鑑.kb](examples/通鑑.kb)。

## 目录

```text
crates/kanben/            核心库（无外部依赖）
  src/syntax/             源码 → 语法树
    mod.rs                块级：设定、标题、段落、环境、段落属性
    inline.rs             通用形式：\命令[參數]{內容}、《》、转义
    commands.rs           命令表：命令名与参数的含义
    ast.rs  span.rs       语法树、位置与诊断
  src/layout/             语法树 → 字格坐标
    flatten.rs            展平成条目：正文 / 夾注 / 旁行 / 眉批 / 擡頭
    token.rs              字与格；標點分类（句讀附着、佔格、白文）
    engine.rs             排版器：折行、夾注平分、低格擡頭、禁則、标记连段
    options.rs            版式选项
  src/render/             字格坐标 → SVG
    page.rs               版面几何：行 → 葉、半葉、版心
    svg.rs                SVG 输出
    ids.rs                IDS 组字
    numerals.rs           葉碼中文数字
  src/bin/kanben.rs       命令行
crates/kanben-wasm/       浏览器接口（wasm-bindgen）
web/                      编辑器页面：左边写源码，右边实时预览
tools/fonts.py            下载开源字體、切片成网页字体（生成 web/fonts）
```

## 使用

工具链由 [mise](https://mise.jdx.dev) 管理（Rust 1.99.1、wasm32 目标、wasm-pack，见 `mise.toml`）：

```sh
mise install          # 安装工具链
mise run test         # 运行全部测试
mise run serve        # 编译 wasm 并在 http://localhost:8765 打开编辑器
mise run example      # 命令行排印示例到 examples/通鑑.html
mise run lint         # 格式与 clippy 检查
mise run fonts        # 生成网页字体到 web/fonts（需要 uv，首次要下载约 50 MB）
```

推送到 `main` 后，GitHub Actions（`.github/workflows/pages.yml`）会检查、测试、编译 wasm，并把 `web/` 发布到 GitHub Pages；示例的排印结果在 `example.html`。

命令行：

```sh
cargo run -p kanben -- 輸入.kb -o 輸出.html --svg 目錄 --font-loader https://lhttjdr.github.io/kanben/fontload.js
```

浏览器：

```js
import init, { typeset, style, fitCells, plainText } from "./pkg/kanben_wasm.js";
await init();
const r = typeset(source, "format=scroll\npunct=grid"); // 第二个参数覆盖 \setup
r.pageCount; r.page(0);                                 // 每页一张 SVG
JSON.parse(r.diagnostics);                              // [{severity, line, col, from, to, message}]
fitCells(source, "", 900, 30);                          // 按高度算每行字數
plainText(selection.toString());                        // 复制文字：直排標點还原
```

SVG 不内嵌样式，页面里引入一次 `style()` 即可；要用开源字體，再调用 `fontload.js` 的 `loadFonts(文字)`。颜色、字体都是 CSS 变量：`--kb-font` `--kb-font-kai` `--kb-ink` `--kb-paper` `--kb-red` 等。

## 字體

| 用途 | 字體 | 网页字体名 |
|---|---|---|
| 正文（`font=mincho`，默认） | [源樣明體](https://github.com/ButTaiwan/genyo-font) | KB Mincho |
| 正文（`font=fangsong`） | [朱雀仿宋](https://github.com/TrionesType/zhuque) | KB Fangsong |
| 旁批、眉批 | [霞鶩文楷](https://github.com/lxgw/LxgwWenKai) | KB Kai |

三种都是 SIL Open Font License 1.1。`tools/fonts.py` 把每种字体切成一字一个的小文件，`web/fontload.js` 只为页面上出现的字登记字体，所以一页只下载用到的那些字（通常几百 KB）。切字属于 OFL 所说的修改版，所以改了字体名，每种字体的授权文件放在 `web/fonts/<名>/OFL.txt`。

## 已知限制

- 佔格模式下直排標點的位置取决于字体。
- 兼行跨过版心（半葉的最后一行）时不会自动避开。
- 还没有实现：表格、插图。
