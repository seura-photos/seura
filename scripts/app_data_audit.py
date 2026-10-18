"""Launches the built app, quits it, and lists every file it wrote outside the
repository: AppData on Windows, ~/Library on macOS, and Documents/Seura Photos
on both (CLAUDE.md rule 2; decision 0013).

Build first, from ui/:
    Windows: npm run tauri -- build --debug --no-bundle
    macOS:   npm run tauri -- build --debug --bundles app
Then, from the repository root:
    python3 scripts/app_data_audit.py [seconds]

Files whose path names the app are listed, and the audit fails if any of them
is outside Documents/Seura Photos. Other new files are only counted per folder,
since other programs write there too; check those by eye when the list changes.
"""

import os
import platform
import subprocess
import sys
import time
from collections import defaultdict
from pathlib import Path

SECONDS = int(sys.argv[1]) if len(sys.argv) > 1 else 15
HOME = Path.home()
# Names the app's files and folders carry: product name, binary, and
# identifier. WebView2's EBWebView folder always sits under one of them.
MARKERS = ("seura photos", "seura-photos", "seuraphotos")
ALLOWED = HOME / "Documents" / "Seura Photos"


def is_running(system):
    if system == "Darwin":
        result = subprocess.run(["pgrep", "-f", "Seura Photos.app/"], capture_output=True)
        return result.returncode == 0
    result = subprocess.run(
        ["tasklist", "/FI", "IMAGENAME eq seura-photos.exe"], capture_output=True, text=True
    )
    return "seura-photos.exe" in result.stdout


def require_running(system):
    if not is_running(system):
        sys.exit("The app isn't running, so the audit would prove nothing.")


def launch_and_quit():
    if platform.system() == "Darwin":
        app = Path("target/debug/bundle/macos/Seura Photos.app")
        subprocess.run(["open", str(app)], check=True)
        time.sleep(SECONDS)
        require_running("Darwin")
        subprocess.run(["osascript", "-e", 'quit app "Seura Photos"'], check=False)
        roots = [HOME / "Library"]
    elif platform.system() == "Windows":
        process = subprocess.Popen([str(Path("target/debug/seura-photos.exe"))])
        time.sleep(SECONDS)
        require_running("Windows")
        # Without /F, taskkill asks the window to close, like a user would.
        subprocess.run(["taskkill", "/IM", "seura-photos.exe"], check=False, capture_output=True)
        try:
            process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            process.kill()
        roots = [
            Path(os.environ["LOCALAPPDATA"]),
            Path(os.environ["APPDATA"]),
            HOME / "AppData" / "LocalLow",
        ]
    else:
        sys.exit("Seura runs on Windows and macOS only.")
    time.sleep(3)
    return roots + [HOME / "Documents" / "Seura Photos"]


def written_since(roots, start):
    for root in roots:
        for folder, _, files in os.walk(root, onerror=lambda _: None):
            for name in files:
                path = Path(folder) / name
                try:
                    info = path.stat()
                except OSError:
                    continue
                if max(info.st_mtime, info.st_ctime) >= start:
                    yield root, path, info.st_size


def main():
    start = time.time() - 1
    roots = launch_and_quit()

    ours = defaultdict(lambda: [0, 0])
    others = defaultdict(int)
    outside = []
    for root, path, size in written_since(roots, start):
        relative = path.relative_to(root)
        if any(marker in str(path).lower() for marker in MARKERS):
            key = root / Path(*relative.parts[: min(3, len(relative.parts))])
            ours[key][0] += 1
            ours[key][1] += size
            if not path.is_relative_to(ALLOWED):
                outside.append(path)
        else:
            others[root / Path(*relative.parts[: min(2, len(relative.parts) - 1) or 1])] += 1

    lines = [f"## What the app wrote ({platform.system()}, {SECONDS} s run)", ""]
    lines += ["Paths that mention Seura:", ""]
    lines += [f"- `{key}`: {n} files, {size // 1024} KB" for key, (n, size) in sorted(ours.items())]
    lines += [] if ours else ["- none"]
    lines += ["", "Other new files (any program), by folder:", ""]
    lines += [f"- `{key}`: {n}" for key, n in sorted(others.items())]
    report = "\n".join(lines)
    print(report)

    summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if summary:
        with open(summary, "a", encoding="utf-8") as file:
            file.write(report + "\n")

    if outside:
        print("\nOutside Documents/Seura Photos (CLAUDE.md rule 2; decision 0013):")
        print("\n".join(f"  {path}" for path in outside))
        sys.exit(1)


if __name__ == "__main__":
    main()
