// 按需加载网页字體：页面上出现哪些字，就只登记这些字。
//
// 每个字是一个小的 woff2（见 tools/fonts.py），fonts/fonts.json 记录每种字體有哪些字。
// 这里为页面上每个字登记一个只管这个字的 FontFace；浏览器只在真的用这个字體画这个字时才下载。
// 字體没生成（fonts.json 不存在）时什么也不做，页面退回系統字體。

const base = new URL("fonts/", import.meta.url);
let index;
const added = new Set();

/** 把 `4E00-9FA5,3000` 解析成有序的区间表。 */
function parse(cover) {
  return cover.split(",").map((r) => {
    const [a, b = a] = r.split("-");
    return [parseInt(a, 16), parseInt(b, 16)];
  });
}

function covered(ranges, cp) {
  let lo = 0;
  let hi = ranges.length - 1;
  while (lo <= hi) {
    const mid = (lo + hi) >> 1;
    const [a, b] = ranges[mid];
    if (cp < a) hi = mid - 1;
    else if (cp > b) lo = mid + 1;
    else return true;
  }
  return false;
}

async function fonts() {
  index ??= fetch(new URL("fonts.json", base))
    .then((r) => (r.ok ? r.json() : {}))
    .catch(() => ({}))
    .then((json) =>
      Object.entries(json).map(([id, f]) => ({ id, family: f.family, ranges: parse(f.cover) })),
    );
  return index;
}

/** 为 `text` 里的每个字登记字體。可以反复调用，已登记的字不会重复。 */
export async function loadFonts(text) {
  const list = await fonts();
  const cps = new Set();
  for (const c of text) cps.add(c.codePointAt(0));
  for (const { id, family, ranges } of list) {
    for (const cp of cps) {
      const key = `${id}:${cp}`;
      if (cp <= 0x20 || added.has(key) || !covered(ranges, cp)) continue;
      added.add(key);
      const hex = cp.toString(16).toUpperCase();
      const face = new FontFace(family, `url("${new URL(`${id}/${hex}.woff2`, base)}")`, {
        unicodeRange: `U+${hex}`,
        display: "swap",
      });
      document.fonts.add(face);
    }
  }
}
