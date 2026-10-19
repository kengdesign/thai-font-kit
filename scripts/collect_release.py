"""Collect desktop bundles and checksums. Runs identically on all build hosts."""
import hashlib
import pathlib
import shutil
import sys

source, output = map(pathlib.Path, sys.argv[1:])
output.mkdir(parents=True, exist_ok=True)
files = sorted(p for p in source.rglob("*") if p.is_file() and p.suffix.lower() in {".exe", ".dmg", ".deb", ".appimage"})
if not files:
    raise SystemExit("No installer produced; refusing to publish an empty artifact")
checksums = []
for file in files:
    destination = output / file.name
    if destination.exists():
        raise SystemExit(f"Duplicate installer filename: {file.name}")
    shutil.copy2(file, destination)
    checksums.append(f"{hashlib.sha256(destination.read_bytes()).hexdigest()}  {destination.name}\n")
(output / f"SHA256SUMS-{output.name}.txt").write_text("".join(checksums))
print(f"Collected {len(files)} installers in {output}")
