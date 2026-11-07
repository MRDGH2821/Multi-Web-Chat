#!/usr/bin/env python3
"""Flatten Tauri bundle artifacts into files for a GitHub Release.

Each downloaded workflow artifact must contain installer files
(AppImage, deb, rpm, msi, exe, dmg). macOS `.app` directories are zipped
so the release can store them as single files.
"""

from __future__ import annotations

import argparse
import shutil
import sys
import zipfile
from pathlib import Path

PACKAGE_SUFFIXES = {".appimage", ".deb", ".dmg", ".exe", ".msi", ".rpm"}


def is_inside_app(path: Path) -> bool:
    return any(parent.name.endswith(".app") for parent in path.parents)


def package_files(root: Path) -> list[Path]:
    files = []
    for path in root.rglob("*"):
        if path.is_symlink() or not path.is_file() or is_inside_app(path):
            continue
        if path.suffix.lower() in PACKAGE_SUFFIXES:
            files.append(path)
    return sorted(files)


def app_bundles(root: Path) -> list[Path]:
    bundles = [
        path
        for path in root.rglob("*.app")
        if path.is_dir() and not is_inside_app(path)
    ]
    return sorted(bundles)


def artifact_roots(source: Path) -> list[Path]:
    if not source.is_dir():
        raise SystemExit(f"Missing package directory {source}")
    named = sorted(
        path
        for path in source.iterdir()
        if path.is_dir() and path.name.startswith("multi-web-chat-")
    )
    return named or [source]


def platform_label(root: Path, source: Path) -> str:
    if root == source:
        return "package"
    prefix = "multi-web-chat-"
    if root.name.startswith(prefix):
        return root.name.removeprefix(prefix)
    return root.name


def allocate(name: str, platform: str, used: set[str]) -> str:
    if name not in used:
        used.add(name)
        return name
    candidate = f"{platform}-{name}"
    index = 2
    while candidate in used:
        candidate = f"{platform}-{index}-{name}"
        index += 1
    used.add(candidate)
    return candidate


def zip_app(app: Path, dest_file: Path) -> None:
    with zipfile.ZipFile(dest_file, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for file in sorted(app.rglob("*")):
            if file.is_symlink() or not file.is_file():
                continue
            archive.write(file, arcname=str(Path(app.name) / file.relative_to(app)))


def collect(source: Path, dest: Path) -> list[str]:
    if dest.exists():
        raise SystemExit(f"{dest} already exists")
    dest.mkdir(parents=True)
    used: set[str] = set()
    names: list[str] = []
    for root in artifact_roots(source):
        label = platform_label(root, source)
        before = len(names)
        for file in package_files(root):
            target_name = allocate(file.name, label, used)
            shutil.copy2(file, dest / target_name)
            names.append(target_name)
        for app in app_bundles(root):
            target_name = allocate(f"{app.name}.zip", label, used)
            zip_app(app, dest / target_name)
            names.append(target_name)
        if len(names) == before:
            raise SystemExit(f"No installer packages in {root.name}")
    return names


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source", type=Path, required=True)
    parser.add_argument("--dest", type=Path, required=True)
    args = parser.parse_args()
    names = collect(args.source, args.dest)
    print("\n".join(names))
    return 0


if __name__ == "__main__":
    sys.exit(main())
