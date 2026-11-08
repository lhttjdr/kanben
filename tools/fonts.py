"""Build the web fonts.

Downloads each OFL font and cuts it into one tiny woff2 per character, plus
`web/fonts/fonts.json` listing which characters each font has. The page loader
(`web/fontload.js`) then registers only the characters actually on the page,
so a typical page downloads a few hundred kilobytes instead of whole fonts.

Subsetting makes a Modified Version under the OFL, so every file is renamed
(KB Fangsong, KB Mincho, KB Kai) and ships with the font's license.

Usage: uv run --with uharfbuzz --with fonttools --with brotli tools/fonts.py
"""

from __future__ import annotations

import argparse
import io
import json
import os
import shutil
import sys
import urllib.request
import zipfile
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import uharfbuzz as hb
from fontTools.ttLib import TTFont

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "web" / "fonts"

FONTS = [
    {
        "id": "fangsong",
        "family": "KB Fangsong",
        "source": "朱雀仿宋 Zhuque Fangsong v0.212",
        "url": "https://github.com/TrionesType/zhuque/releases/download/v0.212/ZhuqueFangsong-v0.212.zip",
        "member": ".ttf",
        "license": "https://raw.githubusercontent.com/TrionesType/zhuque/v0.212/LICENSE.txt",
    },
    {
        "id": "mincho",
        "family": "KB Mincho",
        "source": "源樣明體 GenYoMin2 TC Regular v2.100",
        "url": "https://github.com/ButTaiwan/genyo-font/raw/v2.100/otf/TC/GenYoMin2TC-R.otf",
        "license": "https://raw.githubusercontent.com/ButTaiwan/genyo-font/v2.100/SIL_Open_Font_License_1.1.txt",
    },
    {
        "id": "kai",
        "family": "KB Kai",
        "source": "霞鶩文楷 LXGW WenKai Regular v1.530",
        "url": "https://github.com/lxgw/LxgwWenKai/releases/download/v1.530/LXGWWenKai-Regular.ttf",
        "license": "https://raw.githubusercontent.com/lxgw/LxgwWenKai/v1.530/OFL.txt",
    },
]

# Tables a single vertical-text glyph does not need. The engine already turns
# punctuation into vertical presentation forms, so layout tables can go.
DROP = ["GSUB", "GPOS", "GDEF", "BASE", "JSTF", "vhea", "vmtx", "VORG", "DSIG", "hdmx", "LTSH", "kern"]


def fetch(url: str, cache: Path) -> bytes:
    path = cache / url.rsplit("/", 1)[-1]
    if not path.exists():
        print(f"  download {url}", file=sys.stderr)
        tmp = path.with_suffix(path.suffix + ".part")
        with urllib.request.urlopen(url, timeout=600) as r, open(tmp, "wb") as f:
            shutil.copyfileobj(r, f)
        tmp.rename(path)
    return path.read_bytes()


def font_bytes(spec: dict, cache: Path) -> bytes:
    data = fetch(spec["url"], cache)
    member = spec.get("member")
    if not member:
        return data
    with zipfile.ZipFile(io.BytesIO(data)) as z:
        names = [n for n in z.namelist() if n.lower().endswith(member)]
        if not names:
            raise SystemExit(f"{spec['url']}: no {member} inside")
        names.sort(key=lambda n: ("regular" not in n.lower(), len(n)))
        return z.read(names[0])


def ranges(cps: list[int]) -> str:
    """Code points as compact hex ranges: `4E00-9FA5,3000`."""
    parts = []
    start = prev = cps[0]
    for cp in cps[1:] + [None]:
        if cp is not None and cp == prev + 1:
            prev = cp
            continue
        parts.append(f"{start:X}" if start == prev else f"{start:X}-{prev:X}")
        if cp is not None:
            start = prev = cp
    return ",".join(parts)


def rename(font: TTFont, family: str) -> None:
    """Replace the font's names, as the OFL asks of modified versions."""
    ps = family.replace(" ", "") + "-Regular"
    names = font["name"]
    names.names = [r for r in names.names if r.nameID not in (1, 2, 3, 4, 6, 16, 17, 18, 21, 22)]
    for nid, value in ((1, family), (2, "Regular"), (3, ps), (4, family), (6, ps)):
        names.setName(value, nid, 3, 1, 0x409)


# Each worker process opens the font once and then cuts many characters.
_face = None
_family = ""
_dest = Path()


def _init(data: bytes, family: str, dest: Path) -> None:
    global _face, _family, _dest
    _face = hb.Face(hb.Blob(data))
    _family, _dest = family, dest


def _cut(cps: list[int]) -> None:
    for cp in cps:
        inp = hb.SubsetInput()
        inp.sets(hb.SubsetInputSets.UNICODE).add(cp)
        drop = inp.sets(hb.SubsetInputSets.DROP_TABLE_TAG)
        for tag in DROP:
            drop.add(int.from_bytes(tag.encode(), "big"))
        inp.flags |= hb.SubsetFlags.NO_HINTING
        font = TTFont(io.BytesIO(hb.subset(_face, inp).blob.data))
        rename(font, _family)
        font.flavor = "woff2"
        font.save(_dest / f"{cp:X}.woff2")


def build(spec: dict, cache: Path) -> dict:
    print(f"{spec['family']} ← {spec['source']}", file=sys.stderr)
    data = font_bytes(spec, cache)
    cps = sorted(cp for cp in TTFont(io.BytesIO(data), lazy=True).getBestCmap() if cp > 0x20)
    dest = OUT / spec["id"]
    shutil.rmtree(dest, ignore_errors=True)
    dest.mkdir(parents=True)
    (dest / "OFL.txt").write_bytes(fetch(spec["license"], cache))

    n = os.cpu_count() or 1
    chunks = [cps[i :: n * 8] for i in range(n * 8)]
    with ProcessPoolExecutor(initializer=_init, initargs=(data, spec["family"], dest)) as pool:
        list(pool.map(_cut, chunks))
    size = sum(f.stat().st_size for f in dest.glob("*.woff2"))
    print(f"  {len(cps)} characters, {size / 1e6:.1f} MB", file=sys.stderr)
    return {
        "family": spec["family"],
        "source": spec["source"],
        "license": f"{spec['id']}/OFL.txt",
        "cover": ranges(cps),
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--cache", type=Path, default=ROOT / ".cache" / "fonts")
    args = ap.parse_args()
    args.cache.mkdir(parents=True, exist_ok=True)
    shutil.rmtree(OUT, ignore_errors=True)
    OUT.mkdir(parents=True)
    index = {spec["id"]: build(spec, args.cache) for spec in FONTS}
    (OUT / "fonts.json").write_text(json.dumps(index, ensure_ascii=False), encoding="utf-8")


if __name__ == "__main__":
    main()
