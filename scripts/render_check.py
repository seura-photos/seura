"""Runs the render check: the built app renders a reference image with each
preview path its web view has (WebGPU, WebGL2) and compares the result with
the wgpu export renderer, then exits (M1 slice 5; decision 0004).

Build first, from ui/:
    Windows: npm run tauri -- build --debug --no-bundle
    macOS:   npm run tauri -- build --debug --bundles app
Then, from the repository root:
    python3 scripts/render_check.py

Fails if any path differs from export by more than the tolerance, if no
path is available, or if the app doesn't finish within the time limit.
"""

import os
import platform
import subprocess
import sys
from pathlib import Path

LIMIT = 180

if platform.system() == "Windows":
    app = Path("target/debug/seura-photos.exe")
elif platform.system() == "Darwin":
    # Inside the bundle, so WebKit sees the app's real identifier.
    app = Path("target/debug/bundle/macos/Seura Photos.app/Contents/MacOS/seura-photos")
else:
    sys.exit("Seura runs on Windows and macOS only.")

try:
    result = subprocess.run(
        [str(app)],
        env={**os.environ, "SEURA_RENDER_CHECK": "1"},
        capture_output=True,
        text=True,
        timeout=LIMIT,
    )
except subprocess.TimeoutExpired as timeout:
    print(timeout.stdout or "", end="")
    sys.exit(f"The render check didn't finish within {LIMIT} seconds.")

lines = [line for line in result.stdout.splitlines() if line.startswith("render check")]
print("\n".join(lines) if lines else result.stdout + result.stderr)
sys.exit(result.returncode)
