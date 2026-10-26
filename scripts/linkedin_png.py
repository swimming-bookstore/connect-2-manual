#!/usr/bin/env python3
"""Capture the four LinkedIn cards from the live manual page.

Uses ~demo (RecordSession). The page draws the 1080 card when ?shot=1 is set.
This script only grabs that card. It does not redraw it.

    trunk serve
    PYTHONPATH=$HOME/demo python3 scripts/linkedin_png.py
"""

from __future__ import annotations

import os
import struct
import sys
import time
import zlib
from pathlib import Path

os.environ.setdefault("DISPLAY", ":0.0")
os.environ.setdefault("XAUTHORITY", str(Path.home() / ".Xauthority"))

sys.path.insert(0, str(Path.home() / "demo"))

from record.session import RecordSession  # noqa: E402
from record import x11cap  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs"
BASE = os.environ.get("MANUAL_URL", "http://127.0.0.1:8081")

# window-size is the outer size. --app has no tab strip, so the page is this tall.
# The desk is laid out at 1280px, then scaled into the card. A narrower window
# makes that 1280px box shrink, which leaves a white strip on the right.
WIN_W, WIN_H = 1400, 1160

PAGES = (
    (f"{BASE}/?shot=1&box=cardano-node-1", "linkedin-cardano.png", "cardano-node-1"),
    (f"{BASE}/?shot=1&box=midnight-node-1", "linkedin-midnight.png", "midnight-node-1"),
    (f"{BASE}/cronos?shot=1&box=cronos-node-1", "linkedin-cronos.png", "cronos-node-1"),
    (f"{BASE}/cronos?shot=1&box=cronos-pos-node-1", "linkedin-cronos-pos.png", "cronos-pos-node-1"),
)


def png(frame: bytes, w: int, h: int) -> bytes:
    raw = bytearray()
    stride = w * 4
    for y in range(h):
        raw.append(0)
        row = frame[y * stride : (y + 1) * stride]
        for x in range(w):
            b, g, r = row[x * 4], row[x * 4 + 1], row[x * 4 + 2]
            raw.extend((r, g, b))

    def chunk(tag: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + tag
            + data
            + struct.pack(">I", zlib.crc32(tag + data) & 0xFFFFFFFF)
        )

    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", w, h, 8, 2, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(bytes(raw), 9))
        + chunk(b"IEND", b"")
    )


def crop(frame: bytes, w: int, h: int, x: int, y: int, cw: int, ch: int) -> tuple[bytes, int, int]:
    x = max(0, min(w - 2, x))
    y = max(0, min(h - 2, y))
    cw = min(w - x, cw)
    ch = min(h - y, ch)
    out = bytearray(cw * ch * 4)
    for row in range(ch):
        src = ((y + row) * w + x) * 4
        dst = row * cw * 4
        out[dst : dst + cw * 4] = frame[src : src + cw * 4]
    return bytes(out), cw, ch


def capture(url: str, name: str, expect: str, index: int) -> None:
    title = f"manual-card-{index}"
    with RecordSession(
        url=url,
        out=Path(f"/tmp/{title}.mp4"),
        title=title,
        cdp_port=9340 + index,
        width=WIN_W,
        height=WIN_H,
        wm_class=f"ManualCard{index}",
        user_data_dir=Path(f"/tmp/{title}-chrome"),
    ) as rec:
        rec.wait_js(
            f"document.querySelector('h1') && document.querySelector('h1').textContent.includes({expect!r})",
            secs=25,
            label=expect,
        )
        rec.eval("document.documentElement.classList.add('shot'); true")
        time.sleep(0.8)
        box = rec.eval(
            """(() => {
              const app = document.querySelector('.app');
              const r = app.getBoundingClientRect();
              return {
                x: r.left, y: r.top, w: r.width, h: r.height,
                dpr: window.devicePixelRatio,
                innerW: window.innerWidth, innerH: window.innerHeight,
                outerW: window.outerWidth, outerH: window.outerHeight,
                h1: document.querySelector('h1').textContent
              };
            })()"""
        )
        if box["h1"] != expect:
            raise SystemExit(f"{name}: page shows {box['h1']!r}, wanted {expect!r}")

        dpy = x11cap.x11.XOpenDisplay(None)
        wid = x11cap.find_by_class(dpy, f"ManualCard{index}") or x11cap.find_window(dpy, title)
        w, h = x11cap.geometry(dpy, wid)
        frame = None
        for _ in range(12):
            frame = x11cap.grab_window(dpy, wid, w, h, False)
            if frame and not x11cap.roughly_blank(frame):
                break
            time.sleep(0.15)
        x11cap.x11.XCloseDisplay(dpy)
        if not frame:
            raise SystemExit(f"{name}: blank grab")

        dpr = float(box["dpr"])
        # --app has no tab strip. The page origin is the window origin.
        # outerHeight includes a title bar that is not in the grab.
        x = round(float(box["x"]) * dpr)
        y = round(float(box["y"]) * dpr)
        cw = round(float(box["w"]) * dpr)
        ch = round(float(box["h"]) * dpr)
        cropped, cw, ch = crop(frame, w, h, x, y, cw, ch)
        if (cw, ch) != (1080, 1080):
            raise SystemExit(f"{name}: card is {cw}x{ch}, not 1080 square")
        path = OUT / name
        path.write_bytes(png(cropped, cw, ch))
        print(path, cw, ch, path.stat().st_size)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for i, (url, name, expect) in enumerate(PAGES):
        capture(url, name, expect, i)


if __name__ == "__main__":
    main()
