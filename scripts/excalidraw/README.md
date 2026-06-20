# Excalidraw diagram pipeline

Self-contained tooling for the hand-drawn (Excalidraw) memory-layout diagrams.
Everything lives in this repo — no `/tmp` or external font dependency.

## Files

- `embed_fonts.py` — post-processes `excalidraw_export` SVG output (see below).
- `Virgil.woff2` — Excalidraw's hand-drawn pencil font (used for prose / labels).
- `Cascadia.woff2` — monospace font (used for code text inside diagrams).

## Authoring → rendering workflow

1. **Author** the diagram as `docs/images/<name>.excalidraw` (Excalidraw JSON).
   Follow the style rules in `CLAUDE.md` → "Excalidraw Diagrams".

2. **Render** to SVG:
   ```bash
   npx excalidraw_export docs/images/<name>.excalidraw
   # produces docs/images/<name>.svg
   ```

3. **Post-process** (embed fonts, fix baselines, widen canvas):
   ```bash
   python3 scripts/excalidraw/embed_fonts.py docs/images/<name>.svg
   ```

4. **Verify visually** — open the SVG and actually LOOK at it. Check arrows land
   on the correct cell, no text clips a box border, fills are hachure (not solid).
   Fix the `.excalidraw` source and re-run steps 2–3 until it's right.

5. **Wire into markdown** — replace the old ```` ```bob ```` block with:
   ```markdown
   ![<alt text>](images/<name>.svg)
   ```

## Why embed_fonts.py is needed

`excalidraw_export` references fonts via remote `https://excalidraw.com/...`
URLs (breaks offline / in mdbook) and omits text baselines (renders `y="NaN"`,
dropping headings onto box borders in rsvg and similar renderers). The script
base64-embeds the fonts, computes baselines from each element's font-size, and
adds a right/bottom margin so edge annotations aren't clipped.
