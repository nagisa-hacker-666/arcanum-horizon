#!/usr/bin/env python3
"""Package Arcanum as a source-only `.hnmod`, the file Horizon's registry and
launcher take.

The archive holds `horizon-native.json` and everything under `mixins/arcanum/`:
`.rs` and `.toml` files, the bundle's `data/` folder when it has one, and the
text files a crate folder carries. Dates and modes are fixed, so the same
sources always produce the same bytes and the same SHA-256.

Usage: python3 tools/package.py
"""

import hashlib
import json
import tomllib
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
GAME = "26.3"

# Files a crate folder may carry besides sources, per docs/native-mods.md.
CRATE_TEXT = ("README", "LICENSE", "COPYING", "NOTICE", ".gitignore")


def bundle_files(bundle: Path) -> dict[str, bytes]:
    files: dict[str, bytes] = {}
    for path in sorted(bundle.rglob("*")):
        if not path.is_file():
            continue
        relative = path.relative_to(ROOT).as_posix()
        suffix = path.suffix
        if suffix in (".rs", ".toml"):
            files[relative] = path.read_bytes()
            continue
        if "data" in path.relative_to(bundle).parts:
            files[relative] = path.read_bytes()
            continue
        if path.name.startswith(CRATE_TEXT):
            files[relative] = path.read_bytes()
            continue
        raise SystemExit(f"the registry takes no {relative}: not a source, a data file or a crate text file")
    return files


def package() -> None:
    compat = json.loads((ROOT / "compatibility.json").read_text())
    manifests = sorted((ROOT / "mixins").glob("*/horizon-mixin.toml"))
    if len(manifests) != 1:
        raise SystemExit("a .hnmod holds one bundle: keep one mixins/<id>/horizon-mixin.toml")
    bundle = manifests[0].parent
    mixin = tomllib.loads(manifests[0].read_text())
    mod = mixin["mod"]
    if bundle.name != mod["id"]:
        raise SystemExit(f"rename mixins/{bundle.name}/ to mixins/{mod['id']}/ (the registry finds it by id)")
    if not mod["id"].replace("_", "").isalnum() or not mod["id"].islower():
        raise SystemExit("the registry takes ids of lowercase letters, digits and _ only")
    if mixin.get("compatibility", {}).get("horizon_revision") != compat["horizon_revision"]:
        raise SystemExit("horizon-mixin.toml and compatibility.json pin different Horizon revisions")

    files = bundle_files(bundle)
    if len(files) > 512:
        raise SystemExit(f"{len(files)} files: the registry takes at most 512")
    unpacked = sum(len(content) for content in files.values())
    if unpacked > 64 * 1024 * 1024:
        raise SystemExit(f"{unpacked} bytes unpacked: the registry takes at most 64 MiB")

    manifest = {
        "schema": 1,
        "id": mod["id"],
        "version": mod["version"],
        "game": GAME,
        "horizon": compat["horizon_revision"],
        "mixins": compat["horizon_mixins_revision"],
        "hooks": compat["native_hooks_api"],
    }
    files = {"horizon-native.json": (json.dumps(manifest, indent=2, sort_keys=True) + "\n").encode(), **files}

    out = ROOT / "target" / f"{mod['id']}-{mod['version']}.hnmod"
    out.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(out, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            archive.writestr(info, content)
    print(json.dumps({
        "file": str(out.relative_to(ROOT)),
        "files": len(files),
        "size": out.stat().st_size,
        "sha256": hashlib.sha256(out.read_bytes()).hexdigest(),
    }, indent=2))


if __name__ == "__main__":
    package()
