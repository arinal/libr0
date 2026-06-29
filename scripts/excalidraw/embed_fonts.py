#!/usr/bin/env python3
"""Post-process excalidraw_export SVGs so they render correctly everywhere:
  1. Embed Virgil (hand-drawn pencil) + Cascadia (mono) fonts as base64 data
     URIs, replacing the remote excalidraw.com url() refs (works offline/mdbook).
  2. Fix y="NaN" text baselines: excalidraw_export omits the baseline, so some
     renderers (rsvg, etc.) drop text to y=0 of its group, making headings sit
     on box borders. Replace with a baseline computed from the element font-size.
  3. Widen the viewBox/canvas by a right margin so side notes aren't clipped.

Usage:
    python3 scripts/excalidraw/embed_fonts.py docs/images/foo.svg [more.svg ...]

The Virgil.woff2 and Cascadia.woff2 fonts live next to this script, so the
whole pipeline is self-contained in the repo (no /tmp dependency).
"""
import base64, re, sys, pathlib

# Fonts live alongside this script, regardless of where it's invoked from.
FONT_DIR = pathlib.Path(__file__).resolve().parent
RIGHT_MARGIN = 150  # px added to width so right-edge side notes aren't clipped
BOTTOM_MARGIN = 34  # px added to height so bottom caption isn't clipped

def b64(name):
    return base64.b64encode((FONT_DIR / name).read_bytes()).decode("ascii")

def fix_baselines(svg):
    virgil = b64("Virgil.woff2")
    casc = b64("Cascadia.woff2")
    svg = svg.replace('url("https://excalidraw.com/Virgil.woff2")',
                      f'url("data:font/woff2;base64,{virgil}") format("woff2")')
    svg = svg.replace('url("https://excalidraw.com/Cascadia.woff2")',
                      f'url("data:font/woff2;base64,{casc}") format("woff2")')

    # For each <text ... y="NaN" ... font-size="Npx" ...>, set y to ~0.82*N
    def repl(m):
        full = m.group(0)
        fs = re.search(r'font-size="([\d.]+)px"', full)
        size = float(fs.group(1)) if fs else 14.0
        baseline = round(size * 0.82, 1)
        return full.replace('y="NaN"', f'y="{baseline}"')
    svg = re.sub(r'<text\b[^>]*y="NaN"[^>]*>', repl, svg)
    return svg

def widen(svg):
    # <svg ... viewBox="0 0 W H" width="W" height="H">
    def repl(m):
        vb_w, vb_h, w, h = m.group(1), m.group(2), m.group(3), m.group(4)
        new_w = int(float(w)) + RIGHT_MARGIN
        new_h = int(float(h)) + BOTTOM_MARGIN
        new_vbw = int(float(vb_w)) + RIGHT_MARGIN
        new_vbh = int(float(vb_h)) + BOTTOM_MARGIN
        return (f'viewBox="0 0 {new_vbw} {new_vbh}" width="{new_w}" height="{new_h}"')
    return re.sub(r'viewBox="0 0 ([\d.]+) ([\d.]+)" width="([\d.]+)" height="([\d.]+)"',
                  repl, svg, count=1)

def main(svg_path):
    p = pathlib.Path(svg_path)
    svg = p.read_text()
    svg = fix_baselines(svg)
    svg = widen(svg)
    p.write_text(svg)
    nan_left = svg.count('y="NaN"')
    fonts = svg.count("data:font/woff2;base64")
    print(f"{p.name}: embedded {fonts} font(s), {nan_left} NaN baseline(s) remaining")

if __name__ == "__main__":
    for arg in sys.argv[1:]:
        main(arg)
