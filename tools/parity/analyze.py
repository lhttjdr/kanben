"""比對原版與 SVG 的字位：按閱讀次序對齊，統計各類字的偏差（單位：字），列出最差的幾處。"""
import json, sys, difflib
from pathlib import Path
from collections import defaultdict

FRAME_PAD = 0.3  # SVG 的 y 從文字頂量，原版從列頂量
d = Path(sys.argv[1])
verbose = "-v" in sys.argv
total = defaultdict(lambda: [0, 0.0, 0.0, 0.0])  # n, sum|dx|, sum|dy|, worst
report = []
for f in sorted(d.glob("*.json")):
    j = json.loads(f.read_text())
    a = [c for c in j["classic"]]
    b = [g for g in j["svg"] if not g["hidden"]]
    sa = [c["ch"] for c in a]; sb = [g["text"] for g in b]
    sm = difflib.SequenceMatcher(None, sa, sb, autojunk=False)
    errs = []
    unmatched = [0, 0]
    for tag, i1, i2, j1, j2 in sm.get_opcodes():
        if tag == "equal":
            for i, k in zip(range(i1, i2), range(j1, j2)):
                c, g = a[i], b[k]
                # 絕對橫坐標（字）：行從右往左排，第 col 行中綫在 -1.6·col。
                dx = (g["x"] - 1.6 * g["col"]) - (c["x"] - 1.6 * c["col"])
                dy = g["y"] + FRAME_PAD - c["y"]
                err = max(abs(dx), abs(dy))
                role = c["role"]
                t = total[role]; t[0] += 1; t[1] += abs(dx); t[2] += abs(dy); t[3] = max(t[3], err)
                errs.append((err, role, c["ch"], c["col"], g["col"], round(c["x"], 2), round(g["x"], 2), round(c["y"], 2), round(g["y"] + FRAME_PAD, 2), round(c["size"], 2), g["size"]))
        else:
            unmatched[0] += i2 - i1; unmatched[1] += j2 - j1
    errs.sort(reverse=True)
    bad = sum(1 for e in errs if e[0] > 0.05)
    report.append((f.stem, len(errs), bad, unmatched, errs[:6]))
print(f"{'語料':18}{'對上':>6}{'偏差>0.05':>10}{'原版多':>7}{'SVG多':>7}")
for name, n, bad, un, worst in report:
    print(f"{name:18}{n:6}{bad:10}{un[0]:7}{un[1]:7}")
    if verbose:
        for e in worst:
            if e[0] > 0.05:
                print(f"    {e[1]:5} {e[2]} 列 原{e[3]}/SVG{e[4]}  x 原{e[5]:+.2f}/SVG{e[6]:+.2f}  y 原{e[7]:.2f}/SVG{e[8]:.2f}  字號 原{e[9]}/SVG{e[10]}")
print()
for role, (n, sx, sy, worst) in sorted(total.items()):
    print(f"{role:6} {n:5} 字  平均|dx| {sx/n:.3f}  平均|dy| {sy/n:.3f}  最大 {worst:.2f}")
