"""逐列比較 pixels.mjs 截下的原版與 SVG 版：數出不同的像素，並拼出對照圖。"""
import json, sys
from pathlib import Path
from PIL import Image, ImageChops

d = Path(sys.argv[1])
idx = json.loads((d / "_index.json").read_text())
total_bad = 0
for name, n in idx.items():
    row = []
    for i in range(n):
        a = Image.open(d / f"{name}-c{i}-classic.png").convert("RGB")
        b = Image.open(d / f"{name}-c{i}-svg.png").convert("RGB")
        if a.size != b.size:
            b = b.resize(a.size)
        # 版框位置兩邊算法不同，先在 ±3px 內找最佳對齊，再比字形。去掉上下各 6px（版框綫）。
        w, h = a.size
        core = (4, 6, w - 4, h - 6)
        ac = a.crop(core)
        best = None
        for dx in range(-3, 4):
            for dy in range(-3, 4):
                bc = b.crop((core[0] + dx, core[1] + dy, core[2] + dx, core[3] + dy))
                diff = ImageChops.difference(ac, bc)
                bad = sum(1 for p in diff.get_flattened_data() if max(p) > 40)
                if best is None or bad < best[0]:
                    best = (bad, dx, dy, diff)
        bad, dx, dy, diff = best
        row.append(f"{bad}@{dx},{dy}")
        total_bad += bad
        if bad:
            bc = b.crop((core[0] + dx, core[1] + dy, core[2] + dx, core[3] + dy))
            w, h = ac.size
            pair = Image.new("RGB", (w * 3 + 8, h), "white")
            pair.paste(ac, (0, 0)); pair.paste(bc, (w + 4, 0)); pair.paste(diff.point(lambda v: 255 - min(255, v * 4)), (2 * w + 8, 0))
            pair.save(d / f"{name}-c{i}-pair.png")
    print(f"{name:18} 各列不同的像素：{row}")
print("合計", total_bad)
