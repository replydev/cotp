#!/usr/bin/env python3
"""Render the README GIF using real cotp widgets and deterministic demo accounts.

Requires the repository Rust toolchain and ImageMagick (magick) on PATH.
Run from any directory: python3 ci/render_ui_demo.py
"""

import concurrent.futures
import html
import json
from pathlib import Path
import os
import shutil
import subprocess
import tempfile


def main():
    root = Path(__file__).resolve().parents[1]
    if shutil.which("magick") is None:
        raise SystemExit("ImageMagick is required: install it so 'magick' is on PATH.")
    (root / "target").mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="ui-demo-", dir=root / "target") as temp:
        directory = Path(temp)
        subprocess.run(
            ["cargo", "test", "--locked", "interface::tests::record_ui_demo", "--", "--ignored"],
            cwd=root,
            env={**os.environ, "COTP_UI_DEMO_DIR": temp},
            check=True,
        )
        frames = json.loads((directory / "frames.json").read_text())

        def render(frame):
            source = directory / frame["file"]
            svg = source.read_text()
            # Place tutorial captions beneath the application, outside its viewport.
            svg = svg.replace('height="612"', 'height="654"', 1)
            svg = svg.replace('viewBox="0 0 1080 612"', 'viewBox="0 0 1080 654"', 1)
            caption = html.escape(frame["caption"])
            svg = svg.replace(
                "</svg>",
                '<rect x="0" y="612" width="1080" height="42" fill="#142532"/>'
                '<text x="20" y="639" fill="#5eead4" font-family="DejaVu Sans Mono,monospace" '
                f'font-size="16">{caption}</text></svg>',
            )
            source.write_text(svg)
            target = source.with_suffix(".png")
            subprocess.run(["magick", str(source), str(target)], check=True)
            return target

        with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
            images = list(pool.map(render, frames))
        command = ["magick"]
        for frame, image in zip(frames, images):
            command.extend(["-delay", str(frame["delay"]), str(image)])
        output = root / "img" / "cotp_demo.gif"
        command.extend(["-loop", "0", "-layers", "Optimize", str(output)])
        subprocess.run(command, check=True)
        duration = sum(frame["delay"] for frame in frames) / 100
        print(f"Rendered {len(frames)} frames, {duration:.1f}s, {output.stat().st_size / 1024:.0f} KiB: {output}")


if __name__ == "__main__":
    main()
