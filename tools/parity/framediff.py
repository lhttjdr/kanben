"""整版逐像素比對（frame.mjs 的輸出）：不移位，直接比；寫出差異圖 *-fdiff.png。
用法：python3 tools/parity/framediff.py <輸出目錄>"""
import json, sys
from pathlib import Path
from PIL import Image, ImageChops

d = Path(sys.argv[1])
for name in json.load(open(d / "_index.json")):
    a = Image.open(d / f"{name}-classic.png").convert("RGB")
    b = Image.open(d / f"{name}-svg.png").convert("RGB")
    diff = ImageChops.difference(a, b).convert("L").point(lambda v: 255 if v > 24 else 0)
    n = sum(1 for v in diff.getdata() if v)
    box = diff.getbbox()
    print(f"{name:16} {n:6}  {box}")
    out = Image.new("RGB", (a.width * 3, a.height), "white")
    out.paste(a, (0, 0)); out.paste(b, (a.width, 0)); out.paste(Image.merge("RGB", (diff, Image.new("L", diff.size), Image.new("L", diff.size))), (a.width * 2, 0))
    out.save(d / f"{name}-fdiff.png")
