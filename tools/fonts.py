"""Build the web fonts.

Downloads each OFL font, cuts it into small woff2 slices that each carry a
`unicode-range`, and writes `web/fonts/fonts.css`. The browser then fetches
only the slices for the characters on the page.

Subsetting makes a Modified Version under the OFL, so each slice is renamed
(KB Fangsong, KB Mincho, KB Kai) and ships with the font's license.

Usage: uv run --with fonttools --with brotli tools/fonts.py [--cache DIR]
"""

from __future__ import annotations

import argparse
import io
import shutil
import sys
import urllib.request
import zipfile
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

from fontTools.subset import Options, Subsetter
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

UNIHAN = "https://www.unicode.org/Public/16.0.0/ucd/Unihan.zip"

# Characters per slice. Slices follow character frequency, so a page of
# ordinary text needs the first few slices and little else.
RANKED_SLICE = 120
COMMON_SLICE = 100
RARE_SLICE = 400


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
        # Prefer the regular weight when the archive has several.
        names.sort(key=lambda n: ("regular" not in n.lower(), len(n)))
        return z.read(names[0])


def is_common(cp: int) -> bool:
    """Frequently used Han characters: Big5 level 1 or GB2312 level 1."""
    c = chr(cp)
    try:
        b = c.encode("big5")
        if len(b) == 2 and 0xA440 <= int.from_bytes(b, "big") <= 0xC67E:
            return True
    except UnicodeEncodeError:
        pass
    try:
        b = c.encode("gb2312")
        if len(b) == 2 and 0xB0A1 <= int.from_bytes(b, "big") <= 0xD7F9:
            return True
    except UnicodeEncodeError:
        pass
    return False


def is_han(cp: int) -> bool:
    return 0x3400 <= cp <= 0x9FFF or 0x20000 <= cp <= 0x3FFFF or 0xF900 <= cp <= 0xFAFF


def frequency(cache: Path) -> dict[int, float]:
    """Score Han characters by how often they are used.

    Unihan's kHanyuPinlu gives counts from a frequency dictionary of simplified
    text; each count also goes to the traditional forms of the character.
    kGradeLevel (Hong Kong primary school, traditional) adds a smaller boost.
    """
    with zipfile.ZipFile(io.BytesIO(fetch(UNIHAN, cache))) as z:
        def rows(name: str):
            for line in z.read(name).decode("utf-8").splitlines():
                if line.startswith("U+"):
                    cp, key, value = line.split("\t", 2)
                    yield int(cp[2:], 16), key, value

        score: dict[int, float] = {}
        for cp, key, value in rows("Unihan_Readings.txt"):
            if key == "kHanyuPinlu":
                score[cp] = score.get(cp, 0) + sum(
                    int(v.split("(")[1].rstrip(")")) for v in value.split()
                )
        for cp, key, value in rows("Unihan_Variants.txt"):
            if key == "kTraditionalVariant" and cp in score:
                for v in value.split():
                    t = int(v[2:], 16)
                    if t != cp:
                        score[t] = max(score.get(t, 0), score[cp])
        for cp, key, value in rows("Unihan_DictionaryLikeData.txt"):
            if key == "kGradeLevel":
                score[cp] = score.get(cp, 0) + 2000 / int(value)
    return score


def slices(cps: list[int], score: dict[int, float]) -> list[list[int]]:
    other = [cp for cp in cps if not is_han(cp)]
    han = [cp for cp in cps if is_han(cp)]
    ranked = sorted((cp for cp in han if score.get(cp, 0) > 0), key=lambda cp: -score[cp])
    seen = set(ranked)
    common = [cp for cp in han if cp not in seen and is_common(cp)]
    rare = [cp for cp in han if cp not in seen and not is_common(cp)]
    out = [other] if other else []
    out += [ranked[i : i + RANKED_SLICE] for i in range(0, len(ranked), RANKED_SLICE)]
    out += [common[i : i + COMMON_SLICE] for i in range(0, len(common), COMMON_SLICE)]
    out += [rare[i : i + RARE_SLICE] for i in range(0, len(rare), RARE_SLICE)]
    return [sorted(part) for part in out]


def unicode_range(cps: list[int]) -> str:
    parts = []
    start = prev = cps[0]
    for cp in cps[1:] + [None]:
        if cp is not None and cp == prev + 1:
            prev = cp
            continue
        parts.append(f"U+{start:X}" if start == prev else f"U+{start:X}-{prev:X}")
        if cp is not None:
            start = prev = cp
    return ",".join(parts)


def rename(font: TTFont, family: str) -> None:
    """Replace the font's names, as the OFL asks of modified versions."""
    ps = family.replace(" ", "") + "-Regular"
    names = font["name"]
    for rec in list(names.names):
        if rec.nameID in (1, 3, 4, 6, 16, 17, 18, 21, 22):
            names.removeNames(nameID=rec.nameID)
    for nid, value in ((1, family), (2, "Regular"), (3, ps), (4, family), (6, ps)):
        names.setName(value, nid, 3, 1, 0x409)


def cut(job: tuple) -> None:
    data, part, family, path = job
    font = TTFont(io.BytesIO(data))
    opts = Options()
    opts.flavor = "woff2"
    opts.layout_features = ["*"]
    opts.name_IDs = ["*"]
    opts.notdef_outline = True
    sub = Subsetter(opts)
    sub.populate(unicodes=part)
    sub.subset(font)
    rename(font, family)
    font.flavor = "woff2"
    font.save(path)


def build(spec: dict, cache: Path, score: dict[int, float]) -> str:
    print(f"{spec['family']} ← {spec['source']}", file=sys.stderr)
    data = font_bytes(spec, cache)
    cps = sorted(TTFont(io.BytesIO(data), lazy=True).getBestCmap())
    dest = OUT / spec["id"]
    shutil.rmtree(dest, ignore_errors=True)
    dest.mkdir(parents=True)
    (dest / "OFL.txt").write_bytes(fetch(spec["license"], cache))

    parts = slices(cps, score)
    jobs = [(data, part, spec["family"], dest / f"{i}.woff2") for i, part in enumerate(parts)]
    with ProcessPoolExecutor() as pool:
        list(pool.map(cut, jobs, chunksize=4))
    css = [
        "@font-face{"
        f'font-family:"{spec["family"]}";'
        f'src:url("{spec["id"]}/{i}.woff2") format("woff2");'
        "font-display:swap;"
        f"unicode-range:{unicode_range(part)}"
        "}"
        for i, part in enumerate(parts)
    ]
    size = sum(f.stat().st_size for f in dest.glob("*.woff2"))
    print(f"  {len(parts)} slices, {size / 1e6:.1f} MB, {len(cps)} characters", file=sys.stderr)
    return f"/* {spec['family']}: {spec['source']}, SIL Open Font License 1.1 ({spec['id']}/OFL.txt) */\n" + "\n".join(css)


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--cache", type=Path, default=ROOT / ".cache" / "fonts")
    args = ap.parse_args()
    args.cache.mkdir(parents=True, exist_ok=True)
    OUT.mkdir(parents=True, exist_ok=True)
    score = frequency(args.cache)
    blocks = [build(spec, args.cache, score) for spec in FONTS]
    (OUT / "fonts.css").write_text("\n\n".join(blocks) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
