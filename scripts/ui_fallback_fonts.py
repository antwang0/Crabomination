"""Build the client's symbol fallback fonts: every non-ASCII character a
client string literal uses that the UI font (Mirano Extended Light) lacks,
subset out of the first Noto font that has it.

The UI font covers 13 of the ~80 symbols the HUD, log and menus use; Bevy's
text stack (parley + fontique) only falls back to fonts it has been handed,
so every other one rendered as a box. The client registers these subsets as
fontique fallbacks (`theme::register_symbol_fallbacks`), and
`theme::tests::every_ui_symbol_has_a_glyph` fails when a new symbol lands in
the source without one — rerun this script then.

Sources, first match wins (all SIL OFL 1.1; `OFL.txt` ships beside them):
  Noto Sans Symbols 2, Noto Sans Symbols, Noto Sans Math — geometric shapes,
      dingbats, arrows, box drawing (Arch: `noto-fonts`, /usr/share/fonts/noto)
  Noto Emoji — monochrome pictographs, so they tint with the text colour like
      every other glyph (https://github.com/google/fonts/tree/main/ofl/notoemoji;
      the variable font, instanced here at weight 400)

Usage:
    pip install fonttools
    python scripts/ui_fallback_fonts.py --emoji NotoEmoji[wght].ttf [--noto-dir DIR]
    python scripts/ui_fallback_fonts.py --list      # what is missing, and from where
"""
import argparse
import re
import sys
from pathlib import Path

from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib import instancer

REPO = Path(__file__).resolve().parent.parent
CLIENT_SRC = REPO / "crabomination_client" / "src"
ASSETS = REPO / "crabomination_client" / "assets" / "fonts"
UI_FONT = ASSETS / "MiranoExtendedFreebie-Light.ttf"
OUT_DIR = ASSETS / "fallback"

# (output stem, file name under --noto-dir, or None for --emoji). Order is
# the fallback order the client registers.
SOURCES = [
    ("NotoSansSymbols2", "NotoSansSymbols2-Regular.ttf"),
    ("NotoSansSymbols", "NotoSansSymbols-Regular.ttf"),
    ("NotoSansMath", "NotoSansMath-Regular.ttf"),
    ("NotoEmoji", None),
]

STRING = re.compile(r'"(?:[^"\\]|\\.)*"')


def used_symbols():
    """Non-ASCII characters in string literals of the client's source,
    comments skipped. Variation selectors are formatting, not glyphs."""
    chars = set()
    for path in CLIENT_SRC.rglob("*.rs"):
        for line in path.read_text(encoding="utf-8").splitlines():
            if line.lstrip().startswith("//"):
                continue
            for lit in STRING.findall(line):
                chars.update(c for c in lit if ord(c) > 127 and not 0xFE00 <= ord(c) <= 0xFE0F)
    return chars


def cmap(font):
    return set(font.getBestCmap())


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--noto-dir", default="/usr/share/fonts/noto")
    ap.add_argument("--emoji", help="Noto Emoji variable font (NotoEmoji[wght].ttf)")
    ap.add_argument("--list", action="store_true", help="report only; write nothing")
    args = ap.parse_args()

    have = cmap(TTFont(UI_FONT))
    missing = sorted(c for c in used_symbols() if ord(c) not in have)
    fonts = {}
    for stem, name in SOURCES:
        path = Path(args.emoji) if name is None else Path(args.noto_dir) / name
        if name is None and not args.emoji:
            continue
        font = TTFont(path)
        if name is None and "fvar" in font:
            font = instancer.instantiateVariableFont(font, {"wght": 400})
        fonts[stem] = font

    assigned = {stem: [] for stem in fonts}
    unmatched = []
    for c in missing:
        stem = next((s for s, f in fonts.items() if ord(c) in cmap(f)), None)
        (assigned[stem] if stem else unmatched).append(c)
    for stem, chars in assigned.items():
        print(f"{stem:18} {len(chars):3}  {''.join(chars)}")
    if unmatched:
        print(f"NO SOURCE          {len(unmatched):3}  {''.join(unmatched)}  (change the literal or add a source)")
    if args.list:
        return 0 if not unmatched else 1
    if not args.emoji:
        sys.exit("--emoji is required to write the fonts (the pictographs come from it)")

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    for old in OUT_DIR.glob("*.ttf"):
        old.unlink()
    opts = subset.Options()
    opts.layout_features = ["*"]
    opts.name_IDs = ["*"]
    opts.notdef_outline = True
    for stem, chars in assigned.items():
        if not chars:
            continue
        sub = subset.Subsetter(opts)
        sub.populate(unicodes=[ord(c) for c in chars])
        font = fonts[stem]
        sub.subset(font)
        out = OUT_DIR / f"{stem}.subset.ttf"
        font.save(out)
        print(f"wrote {out.relative_to(REPO)} ({out.stat().st_size} bytes)")
    return 0 if not unmatched else 1


if __name__ == "__main__":
    sys.exit(main())
