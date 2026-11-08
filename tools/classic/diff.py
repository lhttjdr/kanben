"""比對兩組快照：HTML 是否逐字相同、截圖有幾個像素不同。"""
import json, sys
from pathlib import Path
from PIL import Image, ImageChops

a, b = Path(sys.argv[1]), Path(sys.argv[2])
bad = 0
for ja in sorted(a.glob("*.json")):
    jb = b / ja.name
    da, db = json.loads(ja.read_text()), json.loads(jb.read_text())
    same = {k: da[k] == db[k] for k in da}
    ia = Image.open(ja.with_suffix(".png")).convert("RGB")
    ib = Image.open(jb.with_suffix(".png")).convert("RGB")
    diff = ImageChops.difference(ia, ib)
    px = sum(1 for p in diff.get_flattened_data() if max(p) > 0) if ia.size == ib.size else -1
    ok = all(same.values()) and px == 0
    bad += not ok
    flag = "✓" if ok else "✗"
    print(f"{flag} {ja.stem:22} HTML {'相同' if all(same.values()) else '不同 ' + ','.join(k for k, v in same.items() if not v):10} 像素差 {px}")
    if not all(same.values()):
        for k, v in same.items():
            if not v:
                x, y = da[k], db[k]
                i = next((i for i, (p, q) in enumerate(zip(x, y)) if p != q), min(len(x), len(y)))
                print(f"     {k} 第 {i} 字起：\n       原版 {x[max(0,i-40):i+80]!r}\n       移植 {y[max(0,i-40):i+80]!r}")
print(f"\n{bad} 項不符" if bad else "\n全部逐字、逐像素相同")
