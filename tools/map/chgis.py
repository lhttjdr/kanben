"""從 CHGIS V5 時間序列圖層摘出某一年、某一範圍的府州界與縣治，存成地圖用的 GeoJSON。

數據要自己從 Harvard Dataverse 下載（doi:10.7910/DVN/M7WEFY），解壓到一個目錄：
  v5_time_pref_pgn_utf.*   府州（郡）界
  v5_time_cnty_pts_utf.*   縣治

用法：
  uv run --with pyshp --with pyproj --with shapely python tools/map/chgis.py \\
      目錄 年 西 南 東 北 輸出.geojson [--simplify 0.005]

年份用公元（公元前寫負數）。輸出的要素帶 label、kind（pref / cnty）、beg、end 屬性；
每個府州另有一個 kind 為 pref-label 的點，地圖頁在那裏竪排寫名；郡界畫成虛綫區。
"""

import argparse
import json
from pathlib import Path

import shapefile
from pyproj import CRS, Transformer
import shapely
from shapely.geometry import mapping, shape

CITATION = (
    'CHGIS, Version 5. (c) Fairbank Center for Chinese Studies and the Institute for '
    "Chinese Historical Geography at Fudan University, 2012."
)


def reader(d: Path, stem: str):
    return shapefile.Reader(str(d / stem), encoding="utf-8")


def to_wgs84(d: Path, stem: str):
    crs = CRS.from_wkt((d / f"{stem}.prj").read_text())
    return Transformer.from_crs(crs, "EPSG:4326", always_xy=True).transform


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dir", type=Path)
    ap.add_argument("year", type=int)
    ap.add_argument("bbox", type=float, nargs=4, metavar=("W", "S", "E", "N"))
    ap.add_argument("out", type=Path)
    ap.add_argument("--simplify", type=float, default=0.005, help="化簡容差（度）")
    a = ap.parse_args()
    w, s, e, n = a.bbox
    inside = lambda x, y: w <= x <= e and s <= y <= n
    alive = lambda r: r["BEG_YR"] <= a.year <= r["END_YR"]
    feats = []

    stem = "v5_time_pref_pgn_utf"
    proj = to_wgs84(a.dir, stem)
    r = reader(a.dir, stem)
    for sr in r.iterShapeRecords():
        rec = sr.record
        if not alive(rec) or not inside(rec["X_COORD"], rec["Y_COORD"]):
            continue
        if rec["TYPE_CH"] == "非政区国土":
            continue
        g = shapely.transform(shape(sr.shape.__geo_interface__), lambda x, y: proj(x, y), interleaved=False)
        g = g.simplify(a.simplify, preserve_topology=True)
        props = {
            "label": rec["NAME_FT"],
            "kind": "pref",
            "beg": int(rec["BEG_YR"]),
            "end": int(rec["END_YR"]),
        }
        # 面上不放 label（名字寫在下面的代表點上），原名留在 name。
        poly_props = {k: v for k, v in props.items() if k != "label"} | {"name": props["label"]}
        feats.append({"type": "Feature", "geometry": mapping(g), "properties": poly_props})
        # 名字只寫一處：最大一塊裏的代表點（多塊的郡逐塊寫名會重複）。
        big = max(getattr(g, "geoms", [g]), key=lambda p: p.area)
        feats.append({
            "type": "Feature",
            "geometry": mapping(big.representative_point()),
            "properties": {**props, "kind": "pref-label"},
        })

    stem = "v5_time_cnty_pts_utf"
    r = reader(a.dir, stem)
    for rec in r.iterRecords():
        if alive(rec) and inside(rec["X_COORD"], rec["Y_COORD"]):
            feats.append({
                "type": "Feature",
                "geometry": {"type": "Point", "coordinates": [rec["X_COORD"], rec["Y_COORD"]]},
                "properties": {
                    "label": rec["NAME_FT"],
                    "kind": "cnty",
                    "beg": int(rec["BEG_YR"]),
                    "end": int(rec["END_YR"]),
                },
            })

    def rnd(o):
        if isinstance(o, float):
            return round(o, 4)
        if isinstance(o, (list, tuple)):
            return [rnd(x) for x in o]
        if isinstance(o, dict):
            return {k: rnd(v) for k, v in o.items()}
        return o

    out = {
        "type": "FeatureCollection",
        "attribution": CITATION,
        "changes": (
            f"摘取公元 {a.year} 年存在、位於 {w},{s}–{e},{n} 內的府州界與縣治；"
            f"轉為 WGS84 經緯度，邊界以 {a.simplify}° 容差化簡，坐標取四位小數。"
        ),
        "features": rnd(feats),
    }
    a.out.write_text(json.dumps(out, ensure_ascii=False, separators=(",", ":")) + "\n")
    kinds = [f["properties"]["kind"] for f in feats]  # pref-label 不計
    print(f"{a.out}: 府州 {kinds.count('pref')}，縣 {kinds.count('cnty')}")


if __name__ == "__main__":
    main()
