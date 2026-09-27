#!/usr/bin/env python3
"""Build a non-distributable, ad-hoc-signed .app for the platform-shipping example."""

from __future__ import annotations

import argparse
import json
import os
import plistlib
import platform
import shutil
import subprocess
import uuid
from pathlib import Path
from typing import NoReturn


ROOT = Path(__file__).resolve().parents[1]
PACKAGE = "mkit-example-platform-shipping"
DEFAULT_APP = ROOT / "target" / "macos" / "MkitPlatformShipping.app"
OUTPUT_ROOT = ROOT / "target" / "macos"
IDENTIFIER = "dev.mkit.example.platform-shipping"


def fail(message: str) -> NoReturn:
    raise SystemExit(f"error: {message}")


def package_metadata() -> tuple[str, str]:
    result = subprocess.run(
        ["cargo", "metadata", "--locked", "--no-deps", "--format-version", "1"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    metadata = json.loads(result.stdout)
    package = next((item for item in metadata["packages"] if item["name"] == PACKAGE), None)
    if package is None:
        fail(f"workspace package {PACKAGE!r} was not found")
    binaries = [target["name"] for target in package["targets"] if "bin" in target["kind"]]
    if len(binaries) != 1:
        fail(f"expected one binary target for {PACKAGE}, found {binaries}")
    return package["version"], binaries[0]


def validate_bundle(
    app: Path,
    executable_name: str,
    expected_version: str,
    *,
    require_signature: bool = True,
) -> None:
    if not app.is_dir() or app.suffix != ".app":
        fail(f"bundle path is not an .app directory: {app}")

    contents = app / "Contents"
    executable = contents / "MacOS" / executable_name
    plist_path = contents / "Info.plist"
    if not executable.is_file():
        fail(f"missing bundle executable: {executable}")
    if not os.access(executable, os.X_OK):
        fail(f"bundle executable is not executable: {executable}")
    if not plist_path.is_file():
        fail(f"missing bundle metadata: {plist_path}")

    with plist_path.open("rb") as plist_file:
        info = plistlib.load(plist_file)
    expected = {
        "CFBundlePackageType": "APPL",
        "CFBundleIdentifier": IDENTIFIER,
        "CFBundleName": "Mkit Platform Shipping",
        "CFBundleExecutable": executable_name,
        "CFBundleShortVersionString": expected_version,
        "CFBundleVersion": expected_version,
        "NSPrincipalClass": "NSApplication",
    }
    mismatches = [
        f"{key}: expected {value!r}, got {info.get(key)!r}"
        for key, value in expected.items()
        if info.get(key) != value
    ]
    if info.get("NSHighResolutionCapable") is not True:
        mismatches.append("NSHighResolutionCapable must be true")
    if mismatches:
        fail("invalid Info.plist: " + "; ".join(mismatches))

    plutil = shutil.which("plutil")
    if plutil:
        subprocess.run([plutil, "-lint", str(plist_path)], check=True)
    if require_signature:
        codesign = shutil.which("codesign")
        if not codesign:
            fail("codesign is required to validate this macOS app bundle")
        subprocess.run(
            [codesign, "--verify", "--deep", "--strict", "--verbose=2", str(app)],
            check=True,
        )
    print(f"Validated development app bundle: {app}")


def build(app: Path, version: str, executable_name: str) -> None:
    if platform.system() != "Darwin":
        fail(".app bundles can only be built on macOS")

    output_root = OUTPUT_ROOT.resolve()
    if app.parent.resolve() != output_root:
        fail(f"output must be a direct child of {output_root}")
    if app.is_symlink():
        fail(f"refusing symlink output path: {app}")
    if app == output_root:
        fail(f"output must be a bundle inside {output_root}")
    output_root.mkdir(parents=True, exist_ok=True)

    if app.exists():
        if not app.is_dir():
            fail(f"refusing to replace non-directory output: {app}")
        # Validate identity and contents before permitting replacement. This
        # prevents --output from becoming a general-purpose recursive delete.
        validate_bundle(app, executable_name, version, require_signature=False)

    subprocess.run(
        ["cargo", "build", "--locked", "--release", "-p", PACKAGE],
        cwd=ROOT,
        check=True,
    )
    binary = ROOT / "target" / "release" / executable_name
    if not binary.is_file():
        fail(f"Cargo did not produce the expected executable: {binary}")

    staged_app = app.parent / f".{app.name}.staging-{uuid.uuid4().hex}.app"
    backup_app = app.parent / f".{app.name}.backup-{uuid.uuid4().hex}"
    try:
        executable = staged_app / "Contents" / "MacOS" / executable_name
        executable.parent.mkdir(parents=True)
        shutil.copy2(binary, executable)
        executable.chmod(executable.stat().st_mode | 0o111)

        info = {
            "CFBundleDevelopmentRegion": "en",
            "CFBundleExecutable": executable_name,
            "CFBundleIdentifier": IDENTIFIER,
            "CFBundleInfoDictionaryVersion": "6.0",
            "CFBundleName": "Mkit Platform Shipping",
            "CFBundlePackageType": "APPL",
            "CFBundleShortVersionString": version,
            "CFBundleVersion": version,
            "NSHighResolutionCapable": True,
            "NSPrincipalClass": "NSApplication",
        }
        plist_path = staged_app / "Contents" / "Info.plist"
        with plist_path.open("wb") as plist_file:
            plistlib.dump(info, plist_file, fmt=plistlib.FMT_XML, sort_keys=True)

        codesign = shutil.which("codesign")
        if not codesign:
            fail("codesign is required to sign the assembled macOS app bundle")
        subprocess.run(
            [
                codesign,
                "--force",
                "--sign",
                "-",
                "--timestamp=none",
                "--identifier",
                IDENTIFIER,
                str(staged_app),
            ],
            check=True,
        )
        validate_bundle(staged_app, executable_name, version)

        if app.exists():
            app.rename(backup_app)
        try:
            staged_app.rename(app)
        except Exception:
            if backup_app.exists():
                backup_app.rename(app)
            raise
        if backup_app.exists():
            shutil.rmtree(backup_app)
    finally:
        if staged_app.exists():
            shutil.rmtree(staged_app)
    print("Signing status: explicitly ad-hoc signed; no Developer ID distribution signature or notarization is applied")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--output",
        type=Path,
        default=DEFAULT_APP,
        help="output .app path under target/macos (default: target/macos/MkitPlatformShipping.app)",
    )
    parser.add_argument("--validate-only", action="store_true", help="validate an existing app bundle without rebuilding it")
    args = parser.parse_args()

    version, executable_name = package_metadata()
    app = args.output.expanduser().absolute()
    if args.validate_only:
        validate_bundle(app, executable_name, version)
    else:
        build(app, version, executable_name)


if __name__ == "__main__":
    main()
